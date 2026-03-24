#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
HANDOFF_HEADING = "Work Continuation Handoff"
DATE_PATTERN = "%Y-%m-%d %H:%M UTC+8"
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))
STATUS_PATTERN = re.compile(r"^- Status:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
DATE_FIELD_PATTERN = re.compile(r"^- Date:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SOURCE_AGENT_PATTERN = re.compile(r"^- Source agent:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SOURCE_ROLE_PATTERN = re.compile(r"^- Source role:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SCOPE_PATTERN = re.compile(r"^- Scope:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
NEXT_STEP_PATTERN = re.compile(r"^- Next suggested step:\s*(.*?)(?=^-\s|\Z)", re.MULTILINE | re.DOTALL)


class HandoffScanError(Exception):
    pass


@dataclass
class Section:
    heading: str
    body: str
    order: int


def load_registry() -> dict[str, Any]:
    try:
        payload = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise HandoffScanError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise HandoffScanError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise HandoffScanError("invalid registry: top-level JSON value must be an object")
    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise HandoffScanError("invalid registry: agents must be an array")
    return payload


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def resolve_mailbox_path(mailbox_value: str) -> Path:
    mailbox_path = Path(mailbox_value)
    if not mailbox_path.is_absolute():
        mailbox_path = ROOT_DIR / mailbox_value
    return mailbox_path


def last_display_id(entry: dict[str, Any]) -> str | None:
    history = entry.get("display_history")
    if not isinstance(history, list):
        return None
    for record in reversed(history):
        if not isinstance(record, dict):
            continue
        display_id = record.get("display_id")
        if isinstance(display_id, str) and display_id.strip():
            return display_id
    return None


def section_chunks(text: str) -> list[Section]:
    sections: list[Section] = []
    current_heading = ""
    current_lines: list[str] = []

    for line in text.splitlines():
        if line.startswith("## "):
            if current_heading or current_lines:
                sections.append(
                    Section(
                        heading=current_heading,
                        body="\n".join(current_lines).strip(),
                        order=len(sections),
                    )
                )
            current_heading = line[3:].strip()
            current_lines = []
            continue
        current_lines.append(line)

    if current_heading or current_lines:
        sections.append(
            Section(
                heading=current_heading,
                body="\n".join(current_lines).strip(),
                order=len(sections),
            )
        )
    return sections


def match_group(pattern: re.Pattern[str], text: str) -> str | None:
    match = pattern.search(text)
    if match is None:
        return None
    value = match.group(1).strip()
    return value or None


def parse_taipei_date(value: str | None) -> datetime | None:
    if value is None:
        return None
    try:
        parsed = datetime.strptime(value, DATE_PATTERN)
    except ValueError:
        return None
    return parsed.replace(tzinfo=TAIPEI_TIMEZONE).astimezone(timezone.utc)


def normalize_multiline_field(value: str | None) -> list[str]:
    if value is None:
        return []
    lines: list[str] = []
    for raw_line in value.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if line.startswith("- "):
            lines.append(line[2:].strip())
            continue
        lines.append(line)
    return lines


def extract_open_handoff(path: Path) -> dict[str, Any] | None:
    text = path.read_text(encoding="utf-8")
    candidates: list[tuple[tuple[int, int], dict[str, Any]]] = []

    for section in section_chunks(text):
        if section.heading != HANDOFF_HEADING:
            continue
        status = match_group(STATUS_PATTERN, section.body)
        if status is None or status.lower() != "open":
            continue

        date_text = match_group(DATE_FIELD_PATTERN, section.body)
        parsed_date = parse_taipei_date(date_text)
        source_agent = match_group(SOURCE_AGENT_PATTERN, section.body)
        source_role = match_group(SOURCE_ROLE_PATTERN, section.body)
        scope = match_group(SCOPE_PATTERN, section.body)
        next_step_match = NEXT_STEP_PATTERN.search(section.body)
        next_step_lines = normalize_multiline_field(
            next_step_match.group(1).strip() if next_step_match else None
        )

        record = {
            "status": status,
            "date": date_text,
            "source_agent": source_agent,
            "source_role": source_role,
            "scope": scope,
            "next_suggested_step": next_step_lines,
        }
        sort_key = (
            int(parsed_date.timestamp()) if parsed_date is not None else -1,
            section.order,
        )
        candidates.append((sort_key, record))

    if not candidates:
        return None
    candidates.sort(key=lambda item: item[0], reverse=True)
    return candidates[0][1]


def suggested_takeover(entry: dict[str, Any], handoff: dict[str, Any]) -> dict[str, str]:
    agent_uid = str(entry.get("agent_uid") or "")
    scope = str(handoff.get("scope") or entry.get("scope") or "takeover-scope")
    return {
        "stale_agent_ref": agent_uid,
        "scope": scope,
        "command": f"python3 scripts/agent_registry.py takeover {agent_uid} --scope {scope}",
    }


def scan_inactive_coding_handoffs() -> dict[str, Any]:
    registry = load_registry()
    matching_agents = [
        entry
        for entry in registry["agents"]
        if isinstance(entry, dict)
        and entry.get("role") == "coding"
        and entry.get("status") == "inactive"
    ]

    with_open_handoff: list[dict[str, Any]] = []
    missing_mailboxes: list[dict[str, Any]] = []
    without_open_handoff: list[dict[str, Any]] = []

    for entry in sorted(
        matching_agents,
        key=lambda item: (
            str(item.get("inactive_at") or ""),
            str(item.get("assigned_at") or ""),
            str(item.get("agent_uid") or ""),
        ),
    ):
        mailbox_value = entry.get("mailbox")
        agent_uid = entry.get("agent_uid")
        summary = {
            "agent_uid": agent_uid,
            "display_id": last_display_id(entry),
            "inactive_at": entry.get("inactive_at"),
            "scope": entry.get("scope"),
            "mailbox": mailbox_value,
        }

        if not isinstance(mailbox_value, str) or not mailbox_value.strip():
            missing_mailboxes.append({**summary, "reason": "missing mailbox path"})
            continue

        mailbox_path = resolve_mailbox_path(mailbox_value)
        if not mailbox_path.exists():
            missing_mailboxes.append({**summary, "reason": "mailbox file not found"})
            continue

        handoff = extract_open_handoff(mailbox_path)
        if handoff is None:
            without_open_handoff.append(summary)
            continue

        with_open_handoff.append(
            {
                **summary,
                "mailbox": relative_to_root(mailbox_path),
                "handoff": handoff,
                "suggested_takeover": suggested_takeover(entry, handoff),
            }
        )

    return {
        "status": "ok",
        "inactive_coding_count": len(matching_agents),
        "with_open_handoff_count": len(with_open_handoff),
        "missing_mailbox_count": len(missing_mailboxes),
        "without_open_handoff_count": len(without_open_handoff),
        "handoffs": with_open_handoff,
        "missing_mailboxes": missing_mailboxes,
        "without_open_handoff": without_open_handoff,
    }


def print_human(data: dict[str, Any]) -> None:
    print(f"inactive_coding_agents: {data['inactive_coding_count']}")
    print(f"with_open_handoff: {data['with_open_handoff_count']}")
    print(f"missing_mailbox: {data['missing_mailbox_count']}")
    print(f"without_open_handoff: {data['without_open_handoff_count']}")
    print()

    if data["handoffs"]:
        print("open_handoffs:")
        for entry in data["handoffs"]:
            handoff = entry["handoff"]
            print(
                f"  - {entry['agent_uid']} ({entry.get('display_id') or 'unknown-display'})"
                f" inactive_at={entry.get('inactive_at') or 'unknown'}"
            )
            print(f"    mailbox: {entry['mailbox']}")
            print(f"    scope: {handoff.get('scope') or entry.get('scope') or 'unknown'}")
            print(f"    date: {handoff.get('date') or 'unknown'}")
            print(f"    source_agent: {handoff.get('source_agent') or 'unknown'}")
            suggested = entry.get("suggested_takeover") or {}
            if suggested.get("command"):
                print(f"    takeover: {suggested['command']}")
            next_steps = handoff.get("next_suggested_step") or []
            if next_steps:
                print(f"    next_step: {next_steps[0]}")
        print()

    if data["missing_mailboxes"]:
        print("missing_mailboxes:")
        for entry in data["missing_mailboxes"]:
            print(
                f"  - {entry['agent_uid']} ({entry.get('display_id') or 'unknown-display'}):"
                f" {entry['reason']}"
            )
        print()

    if data["without_open_handoff"]:
        print("without_open_handoff:")
        for entry in data["without_open_handoff"]:
            print(f"  - {entry['agent_uid']} ({entry.get('display_id') or 'unknown-display'})")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/inactive_coding_handoffs.py",
        description="Scan inactive coding agents and report their latest open continuation handoffs.",
    )
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        result = scan_inactive_coding_handoffs()
    except HandoffScanError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print_human(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
