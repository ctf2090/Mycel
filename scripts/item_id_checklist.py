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
AGENTS_BOOTSTRAP_TITLE = "New chat bootstrap"
AGENTS_WORKCYCLE_TITLE = "Work Cycle Workflow"
ITEM_ID_COMMENT_RE = re.compile(r"<!--\s*item-id:\s*(?P<item_id>.*?)\s*-->")
CHECKBOX_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+\[(?:X|!|-| )\]\s+(?P<text>.*)$")
LIST_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+(?P<text>.*)$")
HEADING_RE = re.compile(r"^(?P<marks>#{1,6})\s+(?P<text>.+?)\s*$")
CHECKLIST_ITEM_RE = re.compile(
    r"^(?P<indent>\s*)-\s\[(?P<mark>[X!\- ])\]\s.*?(?P<suffix>\s*<!-- item-id: (?P<item_id>.*?) -->\s*)$"
)


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
    source_rel = relative_to_root(source_path)
    if source_rel.startswith("docs/ROLE-CHECKLISTS/") and not stem.startswith("ROLE-"):
        stem = f"ROLE-{stem}"
    return f".agent-local/agents/{agent_uid}/checklists/{stem}-checklist.md"


def agents_bootstrap_checklist_rel(agent_uid: str) -> str:
    return f".agent-local/agents/{agent_uid}/checklists/AGENTS-bootstrap-checklist.md"


def agents_workcycle_checklist_rel(agent_uid: str, batch_num: int) -> str:
    return f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-{batch_num}.md"


def agents_bootstrap_checklist_path(agent_uid: str) -> Path:
    return resolve_path(agents_bootstrap_checklist_rel(agent_uid))


def agents_workcycle_checklist_path(agent_uid: str, batch_num: int) -> Path:
    return resolve_path(agents_workcycle_checklist_rel(agent_uid, batch_num))


def resolve_existing_agent_checklist(path_value: str, *, agent_uid: str) -> Path:
    resolved = resolve_path(path_value).resolve()
    agent_root = (AGENT_DIR / agent_uid).resolve()
    try:
        resolved.relative_to(agent_root)
    except ValueError as exc:
        raise ItemIdChecklistError(
            f"refresh checklist must live under .agent-local/agents/{agent_uid}/"
        ) from exc
    if not resolved.exists():
        raise ItemIdChecklistError(f"refresh checklist not found: {relative_to_root(resolved)}")
    if not resolved.is_file():
        raise ItemIdChecklistError(f"refresh checklist path is not a file: {relative_to_root(resolved)}")
    return resolved


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
            "",
            *body_lines,
            "",
        ]
    )


def collect_existing_item_states(checklist_path: Path) -> dict[str, tuple[str, str | None]]:
    states: dict[str, tuple[str, str | None]] = {}
    lines = checklist_path.read_text(encoding="utf-8").splitlines()
    for index, line in enumerate(lines):
        match = CHECKLIST_ITEM_RE.match(line)
        if match is None:
            continue
        item_id = match.group("item_id").strip()
        mark = match.group("mark")
        indent = match.group("indent")
        problem: str | None = None
        next_index = index + 1
        problem_prefix = f"{indent}  - Problem: "
        if mark == "!" and next_index < len(lines) and lines[next_index].startswith(problem_prefix):
            problem = lines[next_index][len(problem_prefix) :].strip()
        states[item_id] = (mark, problem)
    return states


def apply_existing_item_states(document_text: str, existing_states: dict[str, tuple[str, str | None]]) -> str:
    if not existing_states:
        return document_text

    output_lines: list[str] = []
    for line in document_text.splitlines():
        match = CHECKLIST_ITEM_RE.match(line)
        output_lines.append(line)
        if match is None:
            continue

        item_id = match.group("item_id").strip()
        existing = existing_states.get(item_id)
        if existing is None:
            continue

        mark, problem = existing
        current_mark = match.group("mark")
        indent = match.group("indent")
        if mark != current_mark:
            output_lines[-1] = line.replace(f"[{current_mark}]", f"[{mark}]", 1)
        if mark == "!" and problem:
            output_lines.append(f"{indent}  - Problem: {problem}")

    return "\n".join(output_lines) + "\n"


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


def latest_agents_workcycle_batch_num(agent_uid: str) -> int | None:
    next_batch = next_agents_workcycle_batch_num(agent_uid)
    if next_batch <= 1:
        return None
    return next_batch - 1


def build_agents_section_body(root_lines: list[str], block_lines: list[str]) -> list[str]:
    body = [*root_lines]
    if body and body[-1] != "":
        body.append("")
    body.extend(block_lines)
    return body


def write_agents_section_checklist(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    output_path: Path,
    body_lines: list[str],
    generated_at: str,
    existing_states: dict[str, tuple[str, str | None]] | None = None,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    rendered = render_checklist_document(
        agent_uid=agent_uid,
        display_id=display_id,
        source_path=source_path,
        body_lines=body_lines,
        generated_at=generated_at,
    )
    output_path.write_text(apply_existing_item_states(rendered, existing_states or {}), encoding="utf-8")


def materialize_agents_checklists(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    normalized_lines: list[str],
    item_count: int,
    section: str,
    refresh_path: Path | None = None,
) -> dict[str, Any]:
    root_lines, source_blocks = split_heading_blocks(normalized_lines)
    source_block_map = {title: lines for title, lines in source_blocks}
    bootstrap_block = source_block_map.get(AGENTS_BOOTSTRAP_TITLE)
    workcycle_block = source_block_map.get(AGENTS_WORKCYCLE_TITLE)

    if bootstrap_block is None or workcycle_block is None:
        raise ItemIdChecklistError("AGENTS.md checklist generation requires both bootstrap and workcycle sections")

    generated_at = utc_now()
    result = {
        "agent_uid": agent_uid,
        "display_id": display_id,
        "source": relative_to_root(source_path),
        "item_count": item_count,
        "section": section,
    }

    if section in {"all", "bootstrap"}:
        bootstrap_path = refresh_path if section == "bootstrap" and refresh_path is not None else agents_bootstrap_checklist_path(agent_uid)
        write_agents_section_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            output_path=bootstrap_path,
            body_lines=build_agents_section_body(root_lines, bootstrap_block),
            generated_at=generated_at,
            existing_states=collect_existing_item_states(refresh_path) if refresh_path is not None and section == "bootstrap" else None,
        )
        if section == "bootstrap":
            result["output"] = relative_to_root(bootstrap_path)
            return result
        result["bootstrap_output"] = relative_to_root(bootstrap_path)

    if section in {"all", "workcycle"}:
        if section == "workcycle" and refresh_path is not None:
            workcycle_path = refresh_path
            batch_match = re.search(r"AGENTS-workcycle-checklist-(?P<batch>\d+)\.md$", workcycle_path.name)
            if batch_match is None:
                raise ItemIdChecklistError("refresh path for AGENTS workcycle must target AGENTS-workcycle-checklist-<n>.md")
            batch_num = int(batch_match.group("batch"))
        else:
            batch_num = next_agents_workcycle_batch_num(agent_uid)
            workcycle_path = agents_workcycle_checklist_path(agent_uid, batch_num)
        write_agents_section_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            output_path=workcycle_path,
            body_lines=build_agents_section_body(root_lines, workcycle_block),
            generated_at=generated_at,
            existing_states=collect_existing_item_states(refresh_path) if refresh_path is not None and section == "workcycle" else None,
        )
        if section == "workcycle":
            result["output"] = relative_to_root(workcycle_path)
            result["batch_num"] = batch_num
            return result
        result["workcycle_output"] = relative_to_root(workcycle_path)
        result["batch_num"] = batch_num

    return result


def materialize_checklist(
    *,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
    output_path: Path,
    section: str = "all",
    refresh_path: Path | None = None,
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
            section=section,
            refresh_path=refresh_path,
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    rendered = render_checklist_document(
        agent_uid=agent_uid,
        display_id=display_id,
        source_path=source_path,
        body_lines=normalized_lines,
        generated_at=utc_now(),
    )
    output_path.write_text(
        apply_existing_item_states(rendered, collect_existing_item_states(refresh_path) if refresh_path is not None else {}),
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
    if "section" in data:
        print(f"section: {data['section']}")
    if "bootstrap_output" in data and "workcycle_output" in data:
        print(f"bootstrap_output: {data['bootstrap_output']}")
        print(f"workcycle_output: {data['workcycle_output']}")
        print(f"batch_num: {data['batch_num']}")
    elif "batch_num" in data:
        print(f"output: {data['output']}")
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
    parser.add_argument("--section", choices=["all", "bootstrap", "workcycle"], default="all")
    parser.add_argument(
        "--refresh",
        default="",
        help="rewrite an existing agent-local checklist in place and preserve item states by item-id",
    )
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
        refresh_path = None
        if args.refresh:
            refresh_path = resolve_existing_agent_checklist(args.refresh, agent_uid=agent_uid)
        if args.output and args.refresh:
            raise ItemIdChecklistError("--output cannot be combined with --refresh; refresh rewrites the existing checklist in place")
        if source_path.name == "AGENTS.md" and args.output:
            raise ItemIdChecklistError("AGENTS.md checklist generation manages its own bootstrap/workcycle filenames; omit --output")
        if source_path.name == "AGENTS.md" and refresh_path is not None:
            if args.section == "all":
                raise ItemIdChecklistError("AGENTS.md refresh requires --section bootstrap or --section workcycle")
            if args.section == "bootstrap" and refresh_path.name != "AGENTS-bootstrap-checklist.md":
                raise ItemIdChecklistError("AGENTS.md bootstrap refresh must target AGENTS-bootstrap-checklist.md")
            if args.section == "workcycle" and not re.search(r"AGENTS-workcycle-checklist-\d+\.md$", refresh_path.name):
                raise ItemIdChecklistError("AGENTS.md workcycle refresh must target AGENTS-workcycle-checklist-<n>.md")
        output_path = refresh_path or resolve_checklist_path(args.output or None, agent_uid=agent_uid, source_path=source_path)
        result = materialize_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
            output_path=output_path,
            section=args.section,
            refresh_path=refresh_path,
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
