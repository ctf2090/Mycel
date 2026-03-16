#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from agent_timestamp import build_message
from mailbox_gc import DEFAULT_DELETE_AGE_DAYS, MailboxGcError, delete_stale_mailboxes
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

# Items that must almost always be `checked`, not `not-needed`, in a real work
# cycle batch.  If any of these are `[-]` at `end` time the tool reports them
# and returns exit code 2.
#
# Exclusions per batch are handled inside `scan_scrutinized_not_needed_items`:
# • workflow.reply-with-plan-and-status is auto-set to not-needed in batch 1 by
#   the tool itself, so it is excluded from scrutiny on batch 1.
SCRUTINIZED_NOT_NEEDED_ITEMS: dict[str, str] = {
    "workflow.files-changed-summary": (
        "required when source files changed; paste render_files_changed_table.py output verbatim"
    ),
    "workflow.runtime-preflight-before-verification": (
        "required before running cargo test, scripts, or cargo run in the cycle"
    ),
    "workflow.reply-with-plan-and-status": (
        "required at the start of every non-bootstrap work cycle batch"
    ),
}

# Registry scope values that are known placeholders and should not be used in
# scope-consistency checks (a placeholder scope never matches a real scope, so
# the check would always fail for newly claimed agents).
PLACEHOLDER_SCOPES: frozenset[str] = frozenset({"pending scope", "", "none", "n/a"})


class WorkCycleError(Exception):
    pass


class WorkCycleArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        if "invalid choice: 'start'" in message:
            message = message + "; did you mean 'begin'?"
        super().error(message)


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
    argv = sys.argv[1:]
    if argv and argv[0] == "start":
        argv = ["begin", *argv[1:]]
    if argv and "--model-id" in argv:
        agent_ref = argv[1] if len(argv) > 1 else "<agent_ref>"
        stage = argv[0] if argv else "begin"
        raise WorkCycleError(
            "model id is inferred from the agent registry entry created at claim/bootstrap time; "
            "do not pass `--model-id` here. "
            f"Use `python3 scripts/agent_work_cycle.py {stage} {agent_ref}`"
            + (" --scope <scope>`." if stage == "begin" else "`.")
        )
    if argv and argv[0] == "end" and "--batch" in argv:
        batch_index = argv.index("--batch")
        batch_value = None
        if batch_index + 1 < len(argv):
            batch_value = argv[batch_index + 1]
        agent_ref = argv[1] if len(argv) > 1 else "<agent_ref>"
        guessed_batch = f" {batch_value}" if batch_value is not None else ""
        raise WorkCycleError(
            "batch is inferred from the latest workcycle checklist; "
            f"do not pass `--batch{guessed_batch}`. "
            f"Use `python3 scripts/agent_work_cycle.py end {agent_ref}`."
        )

    parser = WorkCycleArgumentParser(
        prog="scripts/agent_work_cycle.py",
        description="Wrap agent_registry touch/finish with human-facing timestamp lines.",
    )
    parser.add_argument("stage", choices=["begin", "end"], help="begin or end the current work cycle")
    parser.add_argument("agent_ref", help="agent_uid or current display_id")
    parser.add_argument("--scope", help="scope label to append to the timestamp line")
    return parser.parse_args(argv)


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


def resolve_agent_model_id(agent_ref: str) -> str | None:
    status_payload = run_registry("status", agent_ref)
    agents = status_payload.get("agents")
    if not isinstance(agents, list) or len(agents) != 1:
        return None
    model_id = agents[0].get("model_id")
    return model_id if isinstance(model_id, str) and model_id.strip() else None


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


def scan_scrutinized_not_needed_items(
    checklist_path: Path, *, batch_num: int
) -> list[tuple[str, str]]:
    """Return (item_id, reason) for scrutinized items marked `not-needed` (`[-]`).

    Scrutinized items are those that should almost always be `checked` during
    real implementation work.  Marking them `not-needed` without genuine cause
    is the main mechanism by which agents silently skip required steps.
    """
    scrutinized = dict(SCRUTINIZED_NOT_NEEDED_ITEMS)
    # Batch 1 is a bootstrap-only cycle: no source files are changed and no
    # tests or scripts are run, so files-changed-summary and
    # runtime-preflight-before-verification are legitimately not-needed.
    # reply-with-plan-and-status is auto-marked not-needed by begin for batch 1.
    if batch_num == 1:
        scrutinized.pop("workflow.reply-with-plan-and-status", None)
        scrutinized.pop("workflow.files-changed-summary", None)
        scrutinized.pop("workflow.runtime-preflight-before-verification", None)

    violations: list[tuple[str, str]] = []
    for line in checklist_path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped.startswith("- [-] "):
            continue
        for item_id, reason in scrutinized.items():
            if f"item-id: {item_id}" in stripped:
                violations.append((item_id, reason))
    return violations


def extract_open_handoff_scope(mailbox_path: Path, *, agent_role: str) -> str | None:
    """Return the scope of the latest open same-role handoff, or None if absent."""
    own_heading = ROLE_OPEN_HANDOFF_HEADINGS.get(agent_role)
    if own_heading is None:
        return None

    current_heading: str | None = None
    in_own = False
    section_open = False
    section_scope: str | None = None
    last_open_scope: str | None = None

    for line in mailbox_path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            if in_own and section_open:
                last_open_scope = section_scope
            current_heading = line[3:].strip()
            in_own = current_heading == own_heading
            section_open = False
            section_scope = None
            continue
        if not in_own:
            continue
        stripped = line.strip()
        if stripped == "- Status: open":
            section_open = True
        elif stripped.startswith("- Scope: "):
            section_scope = stripped[len("- Scope: "):].strip()

    if in_own and section_open:
        last_open_scope = section_scope
    return last_open_scope


def resolve_agent_scope(agent_ref: str) -> str | None:
    """Return the scope field from the registry for the given agent, or None."""
    status_payload = run_registry("status", agent_ref)
    agents = status_payload.get("agents")
    if not isinstance(agents, list) or len(agents) != 1:
        return None
    scope = agents[0].get("scope")
    return scope if isinstance(scope, str) else None


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


def emit_not_needed_scrutiny_summary(violations: list[tuple[str, str]]) -> None:
    print(f"scrutinized_not_needed_violations: {len(violations)}")
    if not violations:
        return
    print("scrutinized_not_needed_items:")
    for item_id, reason in violations:
        print(f"  - {item_id}: {reason}")


def emit_scope_consistency_summary(
    *, registry_scope: str | None, handoff_scope: str | None
) -> None:
    registry_display = registry_scope or "(not set)"
    handoff_display = handoff_scope or "(not found)"
    print(f"registry_scope: {registry_display}")
    print(f"handoff_scope: {handoff_display}")
    if (
        registry_scope
        and registry_scope.lower() not in PLACEHOLDER_SCOPES
        and handoff_scope
        and handoff_scope.lower() not in PLACEHOLDER_SCOPES
        and registry_scope != handoff_scope
    ):
        print("scope_consistency: MISMATCH — registry scope differs from open handoff scope")
    else:
        print("scope_consistency: ok")


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


def emit_mailbox_gc_summary(result: dict[str, object] | None, *, error: str | None = None) -> None:
    if error is not None:
        print("mailbox_gc_status: error")
        print(f"mailbox_gc_error: {error}")
        return
    if result is None:
        return
    print("mailbox_gc_status: ok")
    print(f"mailbox_gc_min_age_days: {result['min_age_days']}")
    print(f"mailbox_gc_deleted: {result['deleted_count']}")
    deleted = result.get("deleted")
    if isinstance(deleted, list) and deleted:
        print("mailbox_gc_deleted_paths:")
        for record in deleted:
            if isinstance(record, dict):
                print(f"  - {record['path']} ({record['age_days']} days)")


def main() -> int:
    args = parse_args()
    registry_command = "touch" if args.stage == "begin" else "finish"
    payload = run_registry(registry_command, args.agent_ref)
    agent_uid = payload.get("agent_uid") or args.agent_ref
    display_id = payload.get("display_id")
    agent_role = resolve_agent_role(agent_uid)
    model_id = resolve_agent_model_id(agent_uid)

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
            updates.append(("workflow.install-needed-tools", "not-needed"))
            updates.append(("workflow.reply-with-plan-and-status", "not-needed"))
        set_checklist_item_states(workcycle_path, updates)
        print(f"workcycle_output: {workcycle_output}")
        if "batch_num" in checklist_result:
            print(f"batch_num: {checklist_result['batch_num']}")
        print(f"closeout_command: python3 scripts/agent_work_cycle.py end {agent_uid}")
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
        print(build_message(stage, agent=label, agent_uid=agent_uid, model_id=model_id, scope=args.scope))

        for path in checklist_paths:
            unchecked_by_path[path] = scan_unchecked_items(path)

        # Scrutinize not-needed markings on high-value required items.
        not_needed_violations: list[tuple[str, str]] = []
        for path in checklist_paths:
            not_needed_violations.extend(
                scan_scrutinized_not_needed_items(path, batch_num=latest_batch)
            )

        mailbox_path = resolve_agent_mailbox_path(agent_uid)
        open_handoff_lines = scan_open_handoffs(mailbox_path, agent_role=agent_role)

        # Scope consistency: registry scope vs open handoff scope.
        registry_scope = resolve_agent_scope(agent_uid)
        handoff_scope = extract_open_handoff_scope(mailbox_path, agent_role=agent_role)

        emit_checklist_summary(
            checklist_paths=checklist_paths,
            unchecked_by_path=unchecked_by_path,
            bootstrap_batch=bootstrap_batch,
        )
        emit_not_needed_scrutiny_summary(not_needed_violations)
        emit_scope_consistency_summary(registry_scope=registry_scope, handoff_scope=handoff_scope)
        emit_mailbox_summary(mailbox_path, open_handoff_lines)
        shared_fallback_records = scan_shared_fallback_mailboxes()
        emit_shared_fallback_summary(shared_fallback_records)
        mailbox_gc_result: dict[str, object] | None = None
        mailbox_gc_error: str | None = None
        try:
            mailbox_gc_result = delete_stale_mailboxes(
                dry_run=False, min_age_days=DEFAULT_DELETE_AGE_DAYS
            )
        except MailboxGcError as exc:
            mailbox_gc_error = str(exc)
        emit_mailbox_gc_summary(mailbox_gc_result, error=mailbox_gc_error)

        same_role_open_count = len(open_handoff_lines["same_role"])
        other_role_open_count = len(open_handoff_lines["other_role"])
        mailbox_pending = other_role_open_count > 1
        if not bootstrap_batch:
            mailbox_pending = mailbox_pending or same_role_open_count != 1
        shared_fallback_pending = any(record["over_limit"] for record in shared_fallback_records)
        not_needed_pending = len(not_needed_violations) > 0
        return (
            2
            if any(unchecked_by_path.values())
            or mailbox_pending
            or shared_fallback_pending
            or not_needed_pending
            else 0
        )

    emit_registry_summary(payload)

    stage = "before" if args.stage == "begin" else "after"
    label = display_id or args.agent_ref
    print(build_message(stage, agent=label, agent_uid=agent_uid, model_id=model_id, scope=args.scope))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except WorkCycleError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
