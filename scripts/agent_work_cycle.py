#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from agent_timestamp import build_message
from item_id_checklist import (
    agents_bootstrap_checklist_path,
    agents_workcycle_checklist_path,
    latest_agents_workcycle_batch_num,
    materialize_checklist,
)
from item_id_checklist_mark import ItemIdChecklistMarkError, update_checklist_items


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"
AGENTS_PATH = ROOT_DIR / "AGENTS.md"
SHARED_FALLBACK_MAILBOX_LIMIT_BYTES = 1024
SHARED_FALLBACK_MAILBOX_PATHS = [
    ROOT_DIR / ".agent-local" / "coding-to-doc.md",
    ROOT_DIR / ".agent-local" / "doc-to-coding.md",
]
ROLE_OPEN_HANDOFF_HEADINGS = {
    "coding": "Work Continuation Handoff",
    "delivery": "Delivery Continuation Note",
    "doc": "Doc Continuation Note",
}


class WorkCycleError(Exception):
    pass


def run_registry(command: str, agent_ref: str) -> dict[str, str]:
    proc = subprocess.run(
        [sys.executable, str(REGISTRY_SCRIPT), command, agent_ref, "--json"],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or f"{command} failed"
        raise WorkCycleError(message)
    return json.loads(proc.stdout)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_work_cycle.py",
        description="Wrap agent_registry touch/finish with human-facing timestamp lines.",
    )
    parser.add_argument("stage", choices=["begin", "end"], help="begin or end the current work cycle")
    parser.add_argument("agent_ref", help="agent_uid or current display_id")
    parser.add_argument("--scope", help="scope label to append to the timestamp line")
    return parser.parse_args()


def emit_registry_summary(payload: dict[str, str]) -> None:
    if "agent_uid" in payload:
        print(f"agent_uid: {payload['agent_uid']}")
    if "display_id" in payload:
        print(f"display_id: {payload['display_id']}")
    if "role" in payload:
        print(f"role: {payload['role']}")
    if "previous_status" in payload:
        print(f"previous_status: {payload['previous_status']}")
    if "current_status" in payload:
        print(f"current_status: {payload['current_status']}")
    if "last_touched_at" in payload:
        print(f"last_touched_at: {payload['last_touched_at']}")
    if "inactive_at" in payload:
        print(f"inactive_at: {payload['inactive_at']}")


def resolve_agent_mailbox_path(agent_ref: str) -> Path:
    status_payload = run_registry("status", agent_ref)
    agents = status_payload.get("agents")
    if not isinstance(agents, list) or len(agents) != 1:
        raise WorkCycleError(f"unable to resolve mailbox for {agent_ref}")
    mailbox_rel = agents[0].get("mailbox")
    if not isinstance(mailbox_rel, str) or not mailbox_rel.strip():
        raise WorkCycleError(f"agent {agent_ref} is missing mailbox information")
    return ROOT_DIR / mailbox_rel


def resolve_agent_role(agent_ref: str) -> str:
    status_payload = run_registry("status", agent_ref)
    agents = status_payload.get("agents")
    if not isinstance(agents, list) or len(agents) != 1:
        raise WorkCycleError(f"unable to resolve role for {agent_ref}")
    role = agents[0].get("role")
    if not isinstance(role, str) or not role.strip():
        raise WorkCycleError(f"agent {agent_ref} is missing role information")
    return role


def set_checklist_item_states(checklist_path: Path, updates: list[tuple[str, str]]) -> None:
    try:
        update_checklist_items(checklist_path, updates)
    except ItemIdChecklistMarkError as exc:
        raise WorkCycleError(str(exc)) from exc


def scan_unchecked_items(checklist_path: Path) -> list[str]:
    unchecked: list[str] = []
    for line in checklist_path.read_text(encoding="utf-8").splitlines():
        if not line.lstrip().startswith("- [ ] "):
            continue
        unchecked.append(line.strip())
    return unchecked


def emit_checklist_summary(
    *,
    checklist_paths: list[Path],
    unchecked_by_path: dict[Path, list[str]],
    bootstrap_batch: bool,
) -> None:
    print(f"bootstrap_batch: {str(bootstrap_batch).lower()}")
    print(f"checklists_checked: {len(checklist_paths)}")
    total_unchecked = sum(len(items) for items in unchecked_by_path.values())
    print(f"unchecked_items: {total_unchecked}")
    if total_unchecked == 0:
        return
    print("checklist_paths:")
    for path in checklist_paths:
        print(f"  - {path.relative_to(ROOT_DIR)}")
    for path, items in unchecked_by_path.items():
        if not items:
            continue
        print(f"unchecked_in: {path.relative_to(ROOT_DIR)}")
        for item in items:
            print(f"  - {item}")


def scan_open_handoffs(mailbox_path: Path, *, agent_role: str) -> dict[str, list[int]]:
    if not mailbox_path.exists():
        raise WorkCycleError(f"missing mailbox file: {mailbox_path.relative_to(ROOT_DIR)}")

    own_heading = ROLE_OPEN_HANDOFF_HEADINGS.get(agent_role)
    if own_heading is None:
        raise WorkCycleError(f"unsupported agent role for mailbox validation: {agent_role}")

    same_role_open_lines: list[int] = []
    other_role_open_lines: list[int] = []
    current_heading: str | None = None
    for index, line in enumerate(mailbox_path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.startswith("## "):
            current_heading = line[3:].strip()
            continue
        if line.strip() == "- Status: open":
            if current_heading == own_heading:
                same_role_open_lines.append(index)
            else:
                other_role_open_lines.append(index)
    return {"same_role": same_role_open_lines, "other_role": other_role_open_lines}


def emit_mailbox_summary(mailbox_path: Path, open_handoff_lines: dict[str, list[int]]) -> None:
    same_role_open_lines = open_handoff_lines["same_role"]
    other_role_open_lines = open_handoff_lines["other_role"]
    all_open_lines = same_role_open_lines + other_role_open_lines
    print(f"mailbox: {mailbox_path.relative_to(ROOT_DIR)}")
    print(f"open_handoffs: {len(all_open_lines)}")
    print(f"same_role_open_handoffs: {len(same_role_open_lines)}")
    print(f"other_role_open_handoffs: {len(other_role_open_lines)}")
    if same_role_open_lines:
        print("same_role_open_handoff_lines:")
        for line_no in same_role_open_lines:
            print(f"  - {line_no}")
    if other_role_open_lines:
        print("other_role_open_handoff_lines:")
        for line_no in other_role_open_lines:
            print(f"  - {line_no}")


def scan_shared_fallback_mailboxes() -> list[dict[str, int | str | bool]]:
    results: list[dict[str, int | str | bool]] = []
    for path in SHARED_FALLBACK_MAILBOX_PATHS:
        if not path.exists():
            continue
        size_bytes = path.stat().st_size
        results.append(
            {
                "path": str(path.relative_to(ROOT_DIR)),
                "size_bytes": size_bytes,
                "over_limit": size_bytes > SHARED_FALLBACK_MAILBOX_LIMIT_BYTES,
            }
        )
    return results


def emit_shared_fallback_summary(records: list[dict[str, int | str | bool]]) -> None:
    oversized = [record for record in records if record["over_limit"]]
    print(f"shared_fallback_mailboxes_checked: {len(records)}")
    print(f"shared_fallback_mailbox_limit_bytes: {SHARED_FALLBACK_MAILBOX_LIMIT_BYTES}")
    print(f"oversized_shared_fallback_mailboxes: {len(oversized)}")
    if not oversized:
        return
    print("oversized_shared_fallback_mailbox_paths:")
    for record in oversized:
        print(f"  - {record['path']} ({record['size_bytes']} bytes)")


def main() -> int:
    args = parse_args()
    registry_command = "touch" if args.stage == "begin" else "finish"
    payload = run_registry(registry_command, args.agent_ref)
    agent_uid = payload.get("agent_uid") or args.agent_ref
    display_id = payload.get("display_id")
    agent_role = resolve_agent_role(agent_uid)

    checklist_paths: list[Path] = []
    unchecked_by_path: dict[Path, list[str]] = {}
    bootstrap_batch = False

    if args.stage == "begin":
        checklist_result = materialize_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=AGENTS_PATH,
            output_path=agents_bootstrap_checklist_path(agent_uid),
            section="workcycle",
        )
        workcycle_output = checklist_result.get("output")
        if not isinstance(workcycle_output, str):
            raise WorkCycleError("workcycle checklist generation did not return an output path")
        workcycle_path = ROOT_DIR / workcycle_output
        updates: list[tuple[str, str]] = [("workflow.touch-work-cycle", "checked")]
        if checklist_result.get("batch_num") == 1:
            updates.append(("workflow.mailbox-handoff-each-cycle", "not-needed"))
        set_checklist_item_states(workcycle_path, updates)
        print(f"workcycle_output: {workcycle_output}")
        if "batch_num" in checklist_result:
            print(f"batch_num: {checklist_result['batch_num']}")
    else:
        latest_batch = latest_agents_workcycle_batch_num(agent_uid)
        if latest_batch is None:
            raise WorkCycleError(f"no workcycle checklist found for {agent_uid}")

        workcycle_path = agents_workcycle_checklist_path(agent_uid, latest_batch)
        set_checklist_item_states(workcycle_path, [("workflow.finish-work-cycle", "checked")])
        checklist_paths.append(workcycle_path)

        bootstrap_path = agents_bootstrap_checklist_path(agent_uid)
        bootstrap_batch = latest_batch == 1
        if bootstrap_batch:
            if not bootstrap_path.exists():
                raise WorkCycleError(
                    f"missing bootstrap checklist file: {bootstrap_path.relative_to(ROOT_DIR)}"
                )
            checklist_paths.insert(0, bootstrap_path)

        stage = "after"
        label = display_id or args.agent_ref
        emit_registry_summary(payload)
        print(build_message(stage, agent=label, agent_uid=agent_uid, scope=args.scope))

        for path in checklist_paths:
            unchecked_by_path[path] = scan_unchecked_items(path)
        mailbox_path = resolve_agent_mailbox_path(agent_uid)
        open_handoff_lines = scan_open_handoffs(mailbox_path, agent_role=agent_role)
        emit_checklist_summary(
            checklist_paths=checklist_paths,
            unchecked_by_path=unchecked_by_path,
            bootstrap_batch=bootstrap_batch,
        )
        emit_mailbox_summary(mailbox_path, open_handoff_lines)
        shared_fallback_records = scan_shared_fallback_mailboxes()
        emit_shared_fallback_summary(shared_fallback_records)
        same_role_open_count = len(open_handoff_lines["same_role"])
        other_role_open_count = len(open_handoff_lines["other_role"])
        mailbox_pending = other_role_open_count > 1
        if not bootstrap_batch:
            mailbox_pending = mailbox_pending or same_role_open_count != 1
        shared_fallback_pending = any(record["over_limit"] for record in shared_fallback_records)
        return 2 if any(unchecked_by_path.values()) or mailbox_pending or shared_fallback_pending else 0

    emit_registry_summary(payload)

    stage = "before" if args.stage == "begin" else "after"
    label = display_id or args.agent_ref
    print(build_message(stage, agent=label, agent_uid=agent_uid, scope=args.scope))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except WorkCycleError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
