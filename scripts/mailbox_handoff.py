#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path

from agent_registry import (
    RegistryError,
    current_display_id,
    ensure_mailbox,
    load_registry,
    relative_to_root,
    require_non_empty_str,
    resolve_agent_entry,
    resolve_mailbox_path,
)


ROOT_DIR = Path(__file__).resolve().parent.parent
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))


class MailboxHandoffError(Exception):
    pass


@dataclass(frozen=True)
class TemplateSpec:
    kind: str
    heading: str
    status: str


TEMPLATES: dict[str, TemplateSpec] = {
    "work-continuation": TemplateSpec(
        kind="work-continuation",
        heading="Work Continuation Handoff",
        status="open",
    ),
    "planning-sync": TemplateSpec(
        kind="planning-sync",
        heading="Planning Sync Handoff",
        status="open",
    ),
    "doc-continuation": TemplateSpec(
        kind="doc-continuation",
        heading="Doc Continuation Note",
        status="open",
    ),
    "planning-resolution": TemplateSpec(
        kind="planning-resolution",
        heading="Planning Sync Resolution",
        status="resolved",
    ),
}


def human_timestamp(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%d %H:%M UTC+8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/mailbox_handoff.py",
        description="Create mailbox handoff entries from tracked templates.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser("create", help="create a mailbox entry for an agent mailbox")
    create.add_argument("agent_ref", help="agent_uid or current display_id")
    create.add_argument(
        "template",
        choices=sorted(TEMPLATES),
        help="template kind to render",
    )
    create.add_argument("--scope", required=True, help="scope label for the mailbox entry")
    create.add_argument("--source-agent", help="override the source agent label")
    create.add_argument("--source-handoff", action="append", default=[], help="source handoff label")
    create.add_argument("--files-changed", action="append", default=[], help="path changed in the source work")
    create.add_argument("--files-touched", action="append", default=[], help="path touched during docs work")
    create.add_argument("--behavior-change", action="append", default=[], help="behavior change bullet")
    create.add_argument("--planning-impact", action="append", default=[], help="planning impact bullet")
    create.add_argument("--checklist-impact", action="append", default=[], help="checklist impact bullet")
    create.add_argument("--issue-impact", action="append", default=[], help="issue impact bullet")
    create.add_argument("--verification", action="append", default=[], help="verification command or evidence line")
    create.add_argument("--last-landed-commit", action="append", default=[], help="last landed commit bullet")
    create.add_argument("--current-state", action="append", default=[], help="current state bullet")
    create.add_argument("--next-step", action="append", default=[], help="next suggested step bullet")
    create.add_argument("--blockers", action="append", default=[], help="blocker bullet")
    create.add_argument("--notes", action="append", default=[], help="notes bullet")
    create.add_argument("--evidence", action="append", default=[], help="evidence bullet")
    create.add_argument("--docs-impacted", action="append", default=[], help="docs impacted bullet")
    create.add_argument("--remaining-follow-up", action="append", default=[], help="remaining follow-up bullet")
    create.add_argument("--json", action="store_true", help="emit JSON instead of plain text")
    return parser.parse_args()


def normalize_items(values: list[str], *, default: str | None = None) -> list[str]:
    cleaned = [value.strip() for value in values if value and value.strip()]
    if cleaned:
        return cleaned
    if default is None:
        return []
    return [default]


def list_block(label: str, values: list[str], *, default: str | None = "none") -> list[str]:
    lines = [f"- {label}:"]
    for value in normalize_items(values, default=default):
        lines.append(f"  - {value}")
    return lines


def line_block(label: str, value: str) -> list[str]:
    return [f"- {label}: {value}"]


def render_work_continuation(*, date_text: str, source_agent: str, args: argparse.Namespace) -> str:
    behavior = normalize_items(args.behavior_change, default="none")
    current_state = normalize_items(args.current_state)
    next_step = normalize_items(args.next_step)
    if not current_state:
        raise MailboxHandoffError("work-continuation requires at least one --current-state")
    if not next_step:
        raise MailboxHandoffError("work-continuation requires at least one --next-step")

    lines = [
        "## Work Continuation Handoff",
        "",
        "- Status: open",
        f"- Date: {date_text}",
        f"- Source agent: {source_agent}",
        f"- Scope: {args.scope}",
        *list_block("Files changed", args.files_changed),
        *list_block("Behavior change", behavior, default=None),
        *list_block("Verification", args.verification),
        *list_block("Last landed commit", args.last_landed_commit),
        *list_block("Current state", current_state, default=None),
        *list_block("Next suggested step", next_step, default=None),
        *list_block("Blockers", args.blockers),
    ]
    notes = normalize_items(args.notes, default=None)
    if notes:
        lines.extend(list_block("Notes", notes, default=None))
    return "\n".join(lines)


def render_planning_sync(*, date_text: str, source_agent: str, args: argparse.Namespace) -> str:
    lines = [
        "## Planning Sync Handoff",
        "",
        "- Status: open",
        f"- Date: {date_text}",
        f"- Source agent: {source_agent}",
        f"- Scope: {args.scope}",
        *list_block("Files changed", args.files_changed),
        *list_block("Planning impact", args.planning_impact),
        *list_block("Checklist impact", args.checklist_impact),
        *list_block("Issue impact", args.issue_impact),
        *list_block("Verification", args.verification),
        *list_block("Notes", args.notes),
    ]
    return "\n".join(lines)


def render_doc_continuation(*, date_text: str, source_agent: str, args: argparse.Namespace) -> str:
    current_state = normalize_items(args.current_state)
    next_step = normalize_items(args.next_step)
    if not current_state:
        raise MailboxHandoffError("doc-continuation requires at least one --current-state")
    if not next_step:
        raise MailboxHandoffError("doc-continuation requires at least one --next-step")

    lines = [
        "## Doc Continuation Note",
        "",
        "- Status: open",
        f"- Date: {date_text}",
        f"- Source agent: {source_agent}",
        f"- Scope: {args.scope}",
        *list_block("Current state", current_state, default=None),
        *list_block("Evidence", args.evidence),
        *list_block("Next suggested step", next_step, default=None),
    ]
    notes = normalize_items(args.notes, default=None)
    if notes:
        lines.extend(list_block("Notes", notes, default=None))
    return "\n".join(lines)


def render_planning_resolution(*, date_text: str, source_agent: str, args: argparse.Namespace) -> str:
    source_handoffs = normalize_items(args.source_handoff)
    if not source_handoffs:
        raise MailboxHandoffError("planning-resolution requires at least one --source-handoff")

    lines = [
        "## Planning Sync Resolution",
        "",
        "- Status: resolved",
        f"- Date: {date_text}",
        *list_block("Source handoff", source_handoffs, default=None),
        f"- Scope: {args.scope}",
        *list_block("Files touched", args.files_touched),
        *list_block("Docs impacted", args.docs_impacted),
        *list_block("Planning impact", args.planning_impact),
        *list_block("Checklist impact", args.checklist_impact),
        *list_block("Issue impact", args.issue_impact),
        *list_block("Verification", args.verification),
        *list_block("Remaining follow-up", args.remaining_follow_up),
    ]
    notes = normalize_items(args.notes, default=None)
    if notes:
        lines.extend(list_block("Notes", notes, default=None))
    return "\n".join(lines)


def render_entry(template: str, *, date_text: str, source_agent: str, args: argparse.Namespace) -> str:
    if template == "work-continuation":
        return render_work_continuation(date_text=date_text, source_agent=source_agent, args=args)
    if template == "planning-sync":
        return render_planning_sync(date_text=date_text, source_agent=source_agent, args=args)
    if template == "doc-continuation":
        return render_doc_continuation(date_text=date_text, source_agent=source_agent, args=args)
    if template == "planning-resolution":
        return render_planning_resolution(date_text=date_text, source_agent=source_agent, args=args)
    raise MailboxHandoffError(f"unsupported template: {template}")


def supersede_open_entries(content: str) -> tuple[str, int]:
    lines = content.splitlines()
    superseded = 0
    for index, line in enumerate(lines):
        if line.strip() != "- Status: open":
            continue
        prefix = line[: len(line) - len(line.lstrip())]
        lines[index] = f"{prefix}- Status: superseded"
        superseded += 1
    updated = "\n".join(lines)
    if content.endswith("\n"):
        updated += "\n"
    return updated, superseded


def split_mailbox_content(content: str) -> tuple[str, str]:
    marker = "\n## "
    if content.startswith("## "):
        return "", content.strip()
    if marker not in content:
        return content.strip(), ""
    before, after = content.split(marker, maxsplit=1)
    return before.strip(), f"## {after.strip()}"


def build_updated_mailbox(existing_content: str, entry_text: str, *, supersede_open: bool) -> tuple[str, int]:
    if supersede_open:
        superseded_content, superseded_count = supersede_open_entries(existing_content)
    else:
        superseded_content, superseded_count = existing_content, 0
    preamble, body = split_mailbox_content(superseded_content)

    parts: list[str] = []
    if preamble:
        parts.append(preamble)
    if body:
        parts.append(body)
    parts.append(entry_text.strip())
    return "\n\n".join(parts).rstrip() + "\n", superseded_count


def resolve_mailbox(agent_ref: str) -> tuple[str, str, Path]:
    registry = load_registry(allow_missing=False)
    entry = resolve_agent_entry(registry, agent_ref)
    agent_uid = require_non_empty_str(entry, "agent_uid", agent_ref)
    display_id = current_display_id(entry) or agent_uid
    mailbox_value = require_non_empty_str(entry, "mailbox", agent_uid)
    mailbox_path = resolve_mailbox_path(mailbox_value)
    ensure_mailbox(mailbox_path, title=agent_uid)
    return agent_uid, display_id, mailbox_path


def create_entry(args: argparse.Namespace) -> dict[str, object]:
    try:
        agent_uid, display_id, mailbox_path = resolve_mailbox(args.agent_ref)
    except RegistryError as exc:
        raise MailboxHandoffError(str(exc)) from exc

    template = TEMPLATES[args.template]
    source_agent = args.source_agent.strip() if args.source_agent else display_id
    date_text = human_timestamp()
    entry_text = render_entry(template.kind, date_text=date_text, source_agent=source_agent, args=args)

    if mailbox_path.exists():
        existing_content = mailbox_path.read_text(encoding="utf-8")
    else:
        existing_content = ""

    if not existing_content.strip():
        existing_content = f"# Mailbox for {agent_uid}\n"

    updated_content, superseded_count = build_updated_mailbox(
        existing_content,
        entry_text,
        supersede_open=template.status == "open",
    )
    mailbox_path.write_text(updated_content, encoding="utf-8")

    return {
        "agent_uid": agent_uid,
        "display_id": display_id,
        "mailbox": relative_to_root(mailbox_path),
        "template": template.kind,
        "entry_heading": template.heading,
        "scope": args.scope,
        "status": template.status,
        "date": date_text,
        "source_agent": source_agent,
        "superseded_count": superseded_count,
    }


def emit_result(result: dict[str, object], *, as_json: bool) -> None:
    if as_json:
        print(json.dumps(result, indent=2))
        return
    for key in [
        "agent_uid",
        "display_id",
        "mailbox",
        "template",
        "entry_heading",
        "scope",
        "status",
        "date",
        "source_agent",
        "superseded_count",
    ]:
        print(f"{key}: {result[key]}")


def main() -> int:
    args = parse_args()
    if args.command != "create":
        raise MailboxHandoffError(f"unsupported command: {args.command}")
    result = create_entry(args)
    emit_result(result, as_json=args.json)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except MailboxHandoffError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
