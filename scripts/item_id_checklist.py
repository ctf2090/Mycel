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
CHECKBOX_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+\[(?:X|!| )\]\s+(?P<text>.*)$")
LIST_PREFIX_RE = re.compile(r"^(?P<indent>\s*)(?:[-*+]|\d+\.)\s+(?P<text>.*)$")


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
        text = checkbox_match.group("text").strip()
    else:
        list_match = LIST_PREFIX_RE.match(before_comment)
        text = list_match.group("text").strip() if list_match else before_comment.strip()
    return f"- [ ] {text} {comment}", True


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
    normalized_lines: list[str] = []
    item_count = 0
    for line in source_text.splitlines():
        normalized_line, had_item_id = normalize_item_line(line)
        if had_item_id:
            item_count += 1
        normalized_lines.append(normalized_line)

    if item_count == 0:
        raise ItemIdChecklistError(f"source file has no item-id markers: {relative_to_root(source_path)}")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    rendered = [
        "# Agent Item-ID Checklist Copy",
        "",
        f"- Agent UID: `{agent_uid}`",
        f"- Display ID: `{display_id or 'none'}`",
        f"- Source: `{relative_to_root(source_path)}`",
        f"- Generated at: `{utc_now()}`",
        "- This is the agent's personal working copy; update checks here instead of the tracked source file.",
        "- Status meanings: `- [ ]` not checked, `- [X]` checked and completed without problems, `- [!]` checked but problems were found.",
        "- When an item is marked `- [!]`, add an indented subitem immediately below it explaining the problem.",
        "",
        *normalized_lines,
        "",
    ]
    output_path.write_text("\n".join(rendered), encoding="utf-8")

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
