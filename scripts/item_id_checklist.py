#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
AGENT_DIR = ROOT_DIR / ".agent-local" / "agents"
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))
ITEM_ID_COMMENT_RE = re.compile(r"<!--\s*item-id:\s*(?P<item_id>.*?)\s*-->")
CHECKBOX_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+\[(?:X|!|-| )\]\s+(?P<text>.*)$")
LIST_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+(?P<text>.*)$")
HEADING_RE = re.compile(r"^(?P<marks>#{1,6})\s+(?P<text>.+?)\s*$")


class ItemIdChecklistError(Exception):
    pass


def format_timestamp(dt: datetime) -> str:
    return dt.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")


def utc_now() -> str:
    return format_timestamp(datetime.now(timezone.utc))


def relative_to_root(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT_DIR))
    except ValueError:
        return str(path)


def resolve_path(path_value: str) -> Path:
    candidate = Path(path_value)
    if not candidate.is_absolute():
        candidate = ROOT_DIR / candidate
    return candidate


def load_registry() -> dict[str, Any]:
    try:
        payload = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ItemIdChecklistError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise ItemIdChecklistError(f"invalid registry JSON: {exc}") from exc

    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise ItemIdChecklistError("invalid registry: agents must be an array")
    return payload


def resolve_agent_entry(registry: dict[str, Any], identifier: str) -> dict[str, Any]:
    uid_matches = [entry for entry in registry["agents"] if entry.get("agent_uid") == identifier]
    if len(uid_matches) == 1:
        return uid_matches[0]

    display_matches = [entry for entry in registry["agents"] if entry.get("current_display_id") == identifier]
    if len(display_matches) == 1:
        return display_matches[0]

    raise ItemIdChecklistError(f"agent entry not found: {identifier}")


def require_non_empty_str(entry: dict[str, Any], field: str, agent_ref: str) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ItemIdChecklistError(f"agent {agent_ref} is missing required field: {field}")
    return value


def checklist_rel_for(agent_uid: str, source_path: Path) -> str:
    stem = re.sub(r"[^A-Za-z0-9._-]+", "-", source_path.stem).strip("-") or "source"
    return f".agent-local/agents/{agent_uid}/checklists/{stem}-checklist.md"


def agents_bootstrap_checklist_rel(agent_uid: str) -> str:
    return f".agent-local/agents/{agent_uid}/checklists/AGENTS-bootstrap-checklist.md"


def agents_workcycle_checklist_rel(agent_uid: str, batch_num: int) -> str:
    return f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-{batch_num}.md"


def resolve_checklist_path(path_value: str | None, *, agent_uid: str, source_path: Path) -> Path:
    if path_value:
        candidate = resolve_path(path_value)
    else:
        candidate = ROOT_DIR / checklist_rel_for(agent_uid, source_path)

    resolved = candidate.resolve()
    agent_root = (AGENT_DIR / agent_uid).resolve()
    try:
        resolved.relative_to(agent_root)
    except ValueError as exc:
        raise ItemIdChecklistError(
            f"checklist output must live under .agent-local/agents/{agent_uid}/"
        ) from exc
    return resolved


def normalize_item_line(line: str) -> tuple[str, bool]:
    match = ITEM_ID_COMMENT_RE.search(line)
    if match is None:
        return line, False

    comment = match.group(0)
    before_comment = line[: match.start()].rstrip()
    checkbox_match = CHECKBOX_PREFIX_RE.match(before_comment)
    if checkbox_match:
        indent = checkbox_match.group("indent")
        text = checkbox_match.group("text").strip()
    else:
        list_match = LIST_PREFIX_RE.match(before_comment)
        indent = list_match.group("indent") if list_match else ""
        text = list_match.group("text").strip() if list_match else before_comment.strip()
    return f"{indent}- [ ] {text} {comment}", True


def list_item_indent(line: str) -> int | None:
    before_comment = ITEM_ID_COMMENT_RE.sub("", line).rstrip()
    list_match = LIST_PREFIX_RE.match(before_comment)
    if list_match is None:
        return None
    return len(list_match.group("indent"))


def include_parent_list_items(lines: list[str], item_index: int, selected_indices: set[int]) -> None:
    threshold = list_item_indent(lines[item_index])
    if threshold is None:
        return

    for parent_index in range(item_index - 1, -1, -1):
        line = lines[parent_index]
        if HEADING_RE.match(line):
            break
        if not line.strip():
            break

        parent_indent = list_item_indent(line)
        if parent_indent is None or parent_indent >= threshold:
            continue

        selected_indices.add(parent_index)
        threshold = parent_indent


def collect_relevant_lines(lines: list[str]) -> tuple[list[str], int]:
    selected_indices: set[int] = set()
    heading_stack: list[tuple[int, int]] = []
    root_heading_index: int | None = None
    item_count = 0

    for index, line in enumerate(lines):
        heading_match = HEADING_RE.match(line)
        if heading_match is not None:
            level = len(heading_match.group("marks"))
            if level == 1 and root_heading_index is None:
                root_heading_index = index
            heading_stack = [(existing_level, heading_index) for existing_level, heading_index in heading_stack if existing_level < level]
            heading_stack.append((level, index))
            continue

        _, had_item_id = normalize_item_line(line)
        if not had_item_id:
            continue

        item_count += 1
        selected_indices.add(index)
        if root_heading_index is not None:
            selected_indices.add(root_heading_index)
        if heading_stack:
            selected_indices.add(heading_stack[-1][1])
        include_parent_list_items(lines, index, selected_indices)

    rendered_lines: list[str] = []
    previous_was_heading = False
    for index, line in enumerate(lines):
        if index not in selected_indices:
            continue

        normalized_line, had_item_id = normalize_item_line(line)
        output_line = normalized_line if had_item_id else line.rstrip()
        is_heading = HEADING_RE.match(output_line) is not None

        if is_heading and rendered_lines and rendered_lines[-1] != "":
            rendered_lines.append("")
        elif not is_heading and previous_was_heading and rendered_lines and rendered_lines[-1] != "":
            rendered_lines.append("")

        rendered_lines.append(output_line)
        previous_was_heading = is_heading

    return rendered_lines, item_count


def split_heading_blocks(lines: list[str]) -> tuple[list[str], list[tuple[str, list[str]]]]:
    root_lines: list[str] = []
    blocks: list[tuple[str, list[str]]] = []
    current_title: str | None = None
    current_lines: list[str] = []

    for line in lines:
        heading_match = HEADING_RE.match(line)
        if heading_match is None:
            if current_title is None:
                root_lines.append(line)
            else:
                current_lines.append(line)
            continue

        level = len(heading_match.group("marks"))
        heading_text = heading_match.group("text").strip()
        if level == 1:
            if current_title is not None:
                blocks.append((current_title, current_lines))
                current_title = None
                current_lines = []
            root_lines.append(line)
            continue

        if current_title is not None:
            blocks.append((current_title, current_lines))
        current_title = heading_text
        current_lines = [line]

    if current_title is not None:
        blocks.append((current_title, current_lines))

    return root_lines, blocks


def render_checklist_document(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    body_lines: list[str],
    generated_at: str,
) -> str:
    return "\n".join(
        [
            "# Agent Item-ID Checklist Copy",
            "",
            f"- Agent UID: `{agent_uid}`",
            f"- Display ID: `{display_id or 'none'}`",
            f"- Source: `{relative_to_root(source_path)}`",
            f"- Generated at: `{generated_at}`",
            "- This is the agent's personal working copy; update checks here instead of the tracked source file.",
            "- Status meanings: `- [ ]` not checked, `- [X]` checked and completed without problems, `- [-]` not needed for this work cycle, `- [!]` checked but problems were found.",
            "- When an item is marked `- [!]`, add an indented subitem immediately below it explaining the problem.",
            "",
            *body_lines,
            "",
        ]
    )


def next_agents_workcycle_batch_num(agent_uid: str) -> int:
    checklists_dir = (AGENT_DIR / agent_uid / "checklists").resolve()
    if not checklists_dir.exists():
        return 1

    pattern = re.compile(r"^AGENTS-workcycle-checklist-(?P<batch>\d+)\.md$")
    max_batch = 0
    for path in checklists_dir.iterdir():
        match = pattern.match(path.name)
        if match is None:
            continue
        max_batch = max(max_batch, int(match.group("batch")))
    return max_batch + 1


def materialize_agents_checklists(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    normalized_lines: list[str],
    item_count: int,
) -> dict[str, Any]:
    root_lines, source_blocks = split_heading_blocks(normalized_lines)
    source_block_map = {title: lines for title, lines in source_blocks}
    bootstrap_block = source_block_map.get("New chat bootstrap")
    workcycle_block = source_block_map.get("Work Cycle Workflow")

    if bootstrap_block is None or workcycle_block is None:
        raise ItemIdChecklistError("AGENTS.md checklist generation requires both bootstrap and workcycle sections")

    generated_at = utc_now()
    bootstrap_path = resolve_path(agents_bootstrap_checklist_rel(agent_uid))
    bootstrap_path.parent.mkdir(parents=True, exist_ok=True)
    bootstrap_body = [*root_lines]
    if bootstrap_body and bootstrap_body[-1] != "":
        bootstrap_body.append("")
    bootstrap_body.extend(bootstrap_block)
    bootstrap_path.write_text(
        render_checklist_document(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            body_lines=bootstrap_body,
            generated_at=generated_at,
        ),
        encoding="utf-8",
    )

    batch_num = next_agents_workcycle_batch_num(agent_uid)
    workcycle_path = resolve_path(agents_workcycle_checklist_rel(agent_uid, batch_num))
    workcycle_path.parent.mkdir(parents=True, exist_ok=True)
    workcycle_body = [*root_lines]
    if workcycle_body and workcycle_body[-1] != "":
        workcycle_body.append("")
    workcycle_body.extend(workcycle_block)
    workcycle_path.write_text(
        render_checklist_document(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            body_lines=workcycle_body,
            generated_at=generated_at,
        ),
        encoding="utf-8",
    )

    return {
        "agent_uid": agent_uid,
        "display_id": display_id,
        "source": relative_to_root(source_path),
        "bootstrap_output": relative_to_root(bootstrap_path),
        "workcycle_output": relative_to_root(workcycle_path),
        "batch_num": batch_num,
        "item_count": item_count,
    }


def materialize_checklist(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    output_path: Path,
) -> dict[str, Any]:
    if not source_path.exists():
        raise ItemIdChecklistError(f"source file not found: {relative_to_root(source_path)}")
    if not source_path.is_file():
        raise ItemIdChecklistError(f"source path is not a file: {relative_to_root(source_path)}")

    source_text = source_path.read_text(encoding="utf-8")
    normalized_lines, item_count = collect_relevant_lines(source_text.splitlines())

    if item_count == 0:
        raise ItemIdChecklistError(f"source file has no item-id markers: {relative_to_root(source_path)}")

    if source_path.name == "AGENTS.md":
        return materialize_agents_checklists(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            normalized_lines=normalized_lines,
            item_count=item_count,
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_checklist_document(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            body_lines=normalized_lines,
            generated_at=utc_now(),
        ),
        encoding="utf-8",
    )

    return {
        "agent_uid": agent_uid,
        "display_id": display_id,
        "source": relative_to_root(source_path),
        "output": relative_to_root(output_path),
        "item_count": item_count,
    }


def print_human(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"source: {data['source']}")
    if "bootstrap_output" in data and "workcycle_output" in data:
        print(f"bootstrap_output: {data['bootstrap_output']}")
        print(f"workcycle_output: {data['workcycle_output']}")
        print(f"batch_num: {data['batch_num']}")
    else:
        print(f"output: {data['output']}")
    print(f"item_count: {data['item_count']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/item_id_checklist.py",
        description="Create an agent-local checkbox checklist copy from an item-id annotated Markdown file.",
    )
    parser.add_argument("agent_ref")
    parser.add_argument("source_md")
    parser.add_argument("--output", default="")
    parser.add_argument("--json", action="store_true")
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        registry = load_registry()
        entry = resolve_agent_entry(registry, args.agent_ref)
        agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
        display_id = entry.get("current_display_id")
        if not isinstance(display_id, str) or not display_id.strip():
            display_id = None
        source_path = resolve_path(args.source_md)
        if source_path.name == "AGENTS.md" and args.output:
            raise ItemIdChecklistError("AGENTS.md checklist generation manages its own bootstrap/workcycle filenames; omit --output")
        output_path = resolve_checklist_path(args.output or None, agent_uid=agent_uid, source_path=source_path)
        result = materialize_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            output_path=output_path,
        )
    except ItemIdChecklistError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps({"status": "ok", **result}))
    else:
        print_human(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
