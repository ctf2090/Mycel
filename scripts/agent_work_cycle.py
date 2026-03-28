#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from time import perf_counter

from agent_checklist_gc import (
    DEFAULT_KEEP_WORKCYCLE_BATCHES,
    AgentChecklistGcError,
    prune_agent_checklists,
)
from agent_timestamp import build_message
from codex_token_usage_summary import load_latest_usage_snapshot
from mailbox_gc import DEFAULT_DELETE_AGE_DAYS, MailboxGcError, delete_stale_mailboxes
from item_id_checklist import (
    agents_bootstrap_checklist_path,
    agents_workcycle_checklist_path,
    latest_agents_workcycle_batch_num,
    materialize_checklist,
    role_checklist_source_path,
    split_checklist_prefix_for,
    split_workcycle_checklist_path,
)
from item_id_checklist_mark import ItemIdChecklistMarkError, update_checklist_items
from agent_guard import check_agent
from render_next_work_items import NextWorkItemsError, render_payload as render_next_work_items_payload


ROOT_DIR = Path(__file__).resolve().parent.parent
AGENT_LOCAL_DIR = (ROOT_DIR / ".agent-local").resolve()
MAILBOX_DIR = (AGENT_LOCAL_DIR / "mailboxes").resolve()
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"
MAILBOX_HANDOFF_SCRIPT = ROOT_DIR / "scripts" / "mailbox_handoff.py"
CODEX_THREAD_METADATA_SCRIPT = ROOT_DIR / "scripts" / "codex_thread_metadata.py"
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
ROLE_CONTINUATION_TEMPLATES = {
    "coding": "work-continuation",
    "delivery": "delivery-continuation",
    "doc": "doc-continuation",
}
COMPACTION_ABORT_EXIT_CODE = 3

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
    "workflow.reply-with-plan-and-status": (
        "required at the start of every non-bootstrap work cycle batch"
    ),
}

# Registry scope values that are known placeholders and should not be used in
# scope-consistency checks (a placeholder scope never matches a real scope, so
# the check would always fail for newly claimed agents).
PLACEHOLDER_SCOPES: frozenset[str] = frozenset({"pending scope", "", "none", "n/a"})
NON_CYCLE_TRACKED_PATH_PREFIXES: tuple[str, ...] = (".agent-local/", ".git/")
NON_CYCLE_TRACKED_PATH_SUFFIXES: tuple[str, ...] = (".pyc",)


class WorkCycleError(Exception):
    pass


def timed_call(
    phase_timings: dict[str, float] | None,
    key: str,
    func,
    /,
    *args,
    **kwargs,
):
    started_at = perf_counter()
    result = func(*args, **kwargs)
    if phase_timings is not None:
        phase_timings[key] = round(perf_counter() - started_at, 6)
    return result


def emit_phase_timings(phase_timings: dict[str, float] | None) -> None:
    if not phase_timings:
        return
    print("phase_timings_seconds:")
    for key, value in phase_timings.items():
        print(f"  - {key}={value:.3f}")


class WorkCycleArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        if "invalid choice: 'start'" in message:
            message = message + "; did you mean 'begin'?"
        super().error(message)


def run_registry(command: str, agent_ref: str, *, scope: str | None = None) -> dict[str, str]:
    cmd = [sys.executable, str(REGISTRY_SCRIPT), command, agent_ref]
    if scope:
        cmd.extend(["--scope", scope])
    cmd.append("--json")
    proc = subprocess.run(
        cmd,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or f"{command} failed"
        if (
            command == "touch"
            and "has no active display_id; recover it before touch" in message
        ):
            message = (
                message
                + "; this is a display-slot recovery problem, not by itself "
                + "a compact_context_detected guard block. Recover or claim a fresh agent "
                + "before treating the thread as blocked."
            )
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
    parser.add_argument(
        "stage",
        choices=["begin", "end", "record-paths"],
        help="begin or end the current work cycle, or record owned file paths for the latest active batch",
    )
    parser.add_argument("agent_ref", help="agent_uid or current display_id")
    parser.add_argument("paths", nargs="*", help="repo-relative or absolute file paths to record for record-paths")
    parser.add_argument("--scope", help="scope label to append to the timestamp line")
    parser.add_argument(
        "--phase-timings",
        action="store_true",
        help="emit phase timing diagnostics for begin/end workflow steps",
    )
    parser.add_argument(
        "--blocked-closeout",
        action="store_true",
        help="allow an explicit non-normal closeout when the agent is blocked by agent_guard",
    )
    args = parser.parse_args(argv)
    if args.stage == "record-paths":
        if not args.paths:
            raise WorkCycleError(
                "record-paths requires at least one path. "
                "Use `python3 scripts/agent_work_cycle.py record-paths <agent-ref> <path>...`."
            )
    elif args.paths:
        raise WorkCycleError(
            "unexpected file paths for begin/end. "
            f"Use `python3 scripts/agent_work_cycle.py {args.stage} {args.agent_ref}`."
        )
    return args


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
    mailbox_path = (ROOT_DIR / mailbox_rel).resolve()
    try:
        mailbox_path.relative_to(MAILBOX_DIR)
    except ValueError as exc:
        raise WorkCycleError(
            f"agent {agent_ref} has mailbox outside .agent-local/mailboxes/: {mailbox_rel}"
        ) from exc
    return mailbox_path


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


def resolve_current_codex_metadata() -> tuple[str | None, str | None]:
    if not CODEX_THREAD_METADATA_SCRIPT.exists():
        return (None, None)
    proc = subprocess.run(
        [sys.executable, str(CODEX_THREAD_METADATA_SCRIPT), "--shell", "--cwd", str(ROOT_DIR)],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return (None, None)

    values: dict[str, str] = {}
    for raw_line in proc.stdout.splitlines():
        line = raw_line.strip()
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        try:
            parsed = json.loads(value)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, str) and parsed.strip():
            values[key] = parsed.strip()
    return (values.get("MODEL"), values.get("EFFORT"))


def resolve_current_codex_thread_id_from_metadata() -> str | None:
    if not CODEX_THREAD_METADATA_SCRIPT.exists():
        return None
    proc = subprocess.run(
        [sys.executable, str(CODEX_THREAD_METADATA_SCRIPT), "--shell", "--cwd", str(ROOT_DIR)],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None

    for raw_line in proc.stdout.splitlines():
        line = raw_line.strip()
        if not line.startswith("THREAD_ID="):
            continue
        _, value = line.split("=", 1)
        try:
            parsed = json.loads(value)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, str) and parsed.strip():
            return parsed.strip()
    return None


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


def workcycle_git_state_path(agent_uid: str, batch_num: int) -> Path:
    return (
        ROOT_DIR
        / ".agent-local"
        / "agents"
        / agent_uid
        / "workcycles"
        / f"git-state-{batch_num}.json"
    )


def workcycle_owned_paths_path(agent_uid: str, batch_num: int) -> Path:
    return (
        ROOT_DIR
        / ".agent-local"
        / "agents"
        / agent_uid
        / "workcycles"
        / f"owned-paths-{batch_num}.json"
    )


def workcycle_token_state_path(agent_uid: str, batch_num: int) -> Path:
    return (
        ROOT_DIR
        / ".agent-local"
        / "agents"
        / agent_uid
        / "workcycles"
        / f"token-usage-{batch_num}.json"
    )


def workcycle_end_token_state_path(agent_uid: str, batch_num: int) -> Path:
    return (
        ROOT_DIR
        / ".agent-local"
        / "agents"
        / agent_uid
        / "workcycles"
        / f"token-usage-end-{batch_num}.json"
    )


def run_git(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )


def parse_git_status_paths(output: str) -> list[str]:
    paths: list[str] = []
    for line in output.splitlines():
        if len(line) < 4:
            continue
        path = line[3:]
        if " -> " in path:
            _, path = path.split(" -> ", 1)
        normalized = path.strip()
        if normalized:
            paths.append(normalized)
    return paths


def capture_git_state_snapshot() -> dict[str, object]:
    head_proc = run_git(["rev-parse", "HEAD"])
    status_proc = run_git(["status", "--porcelain=v1", "--untracked-files=all"])
    if head_proc.returncode != 0 or status_proc.returncode != 0:
        return {
            "available": False,
            "head": None,
            "status_paths": [],
        }
    return {
        "available": True,
        "head": head_proc.stdout.strip(),
        "status_paths": parse_git_status_paths(status_proc.stdout),
    }


def store_git_state_snapshot(agent_uid: str, batch_num: int) -> None:
    path = workcycle_git_state_path(agent_uid, batch_num)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(capture_git_state_snapshot(), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def capture_token_usage_snapshot() -> dict[str, object] | None:
    snapshot = load_latest_usage_snapshot(
        cwd=str(ROOT_DIR),
        thread_id=resolve_current_codex_thread_id(),
    )
    if snapshot is None:
        return None
    return snapshot


def resolve_current_codex_thread_id() -> str | None:
    metadata_thread_id = resolve_current_codex_thread_id_from_metadata()
    if metadata_thread_id is not None:
        return metadata_thread_id
    thread_id = os.environ.get("CODEX_THREAD_ID")
    if thread_id is None:
        return None
    normalized = thread_id.strip()
    return normalized or None


def store_token_usage_snapshot(agent_uid: str, batch_num: int) -> dict[str, object] | None:
    snapshot = capture_token_usage_snapshot()
    path = workcycle_token_state_path(agent_uid, batch_num)
    path.parent.mkdir(parents=True, exist_ok=True)
    if snapshot is None:
        path.write_text("null\n", encoding="utf-8")
    else:
        path.write_text(json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return snapshot


def load_token_usage_snapshot(agent_uid: str, batch_num: int) -> dict[str, object] | None:
    path = workcycle_token_state_path(agent_uid, batch_num)
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    return payload if isinstance(payload, dict) else None


def detect_compaction_event(
    snapshot: dict[str, object] | None,
    *,
    after_timestamp: str | None = None,
) -> dict[str, str] | None:
    if snapshot is None:
        return None
    rollout_path_value = snapshot.get("rollout_path")
    if not isinstance(rollout_path_value, str) or not rollout_path_value.strip():
        return None
    rollout_path = Path(rollout_path_value)
    return detect_compaction_event_in_rollout_path(
        rollout_path,
        after_timestamp=after_timestamp,
    )


def detect_compaction_event_in_rollout_path(
    rollout_path: Path,
    *,
    after_timestamp: str | None = None,
) -> dict[str, str] | None:
    if not rollout_path.exists():
        return None

    latest: dict[str, str] | None = None
    for raw_line in rollout_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if entry.get("type") not in {"compaction", "compacted"}:
            continue
        timestamp = entry.get("timestamp")
        if not isinstance(timestamp, str) or not timestamp.strip():
            timestamp = "unknown"
        if after_timestamp and timestamp != "unknown" and timestamp <= after_timestamp:
            continue
        latest = {
            "timestamp": timestamp,
            "rollout_path": str(rollout_path),
        }
    return latest


def create_compaction_handoff(
    agent_ref: str,
    *,
    agent_role: str,
    scope: str,
    detection: dict[str, str],
) -> dict[str, object]:
    template = ROLE_CONTINUATION_TEMPLATES.get(agent_role)
    if template is None:
        raise WorkCycleError(f"unsupported role for compaction handoff: {agent_role}")

    cmd = [
        sys.executable,
        str(MAILBOX_HANDOFF_SCRIPT),
        "create",
        agent_ref,
        template,
        "--scope",
        scope,
        "--current-state",
        "Compact context detected in the current chat thread before work started, so this workcycle was aborted.",
        "--next-step",
        "Open a fresh chat for better performance and continue from this handoff.",
        "--notes",
        (
            "Compaction event detected at "
            f"{detection['timestamp']} in {detection['rollout_path']}."
        ),
        "--json",
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or "mailbox_handoff.py create failed"
        raise WorkCycleError(f"could not create compaction handoff: {message}")
    return json.loads(proc.stdout)


def record_compaction_block(
    agent_ref: str,
    *,
    scope: str,
    detection: dict[str, str],
    handoff_mailbox: str,
) -> None:
    from agent_guard import block_agent

    block_agent(
        agent_ref,
        reason="compact_context_detected",
        detected_at=detection["timestamp"],
        source="agent_work_cycle.begin",
        scope=scope,
        handoff_path=handoff_mailbox,
        rollout_path=detection["rollout_path"],
    )


def store_end_token_usage_snapshot_once(agent_uid: str, batch_num: int) -> dict[str, object] | None:
    path = workcycle_end_token_state_path(agent_uid, batch_num)
    if path.exists():
        return load_end_token_usage_snapshot(agent_uid, batch_num)

    snapshot = capture_token_usage_snapshot()
    path.parent.mkdir(parents=True, exist_ok=True)
    if snapshot is None:
        path.write_text("null\n", encoding="utf-8")
    else:
        path.write_text(json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return snapshot


def load_end_token_usage_snapshot(agent_uid: str, batch_num: int) -> dict[str, object] | None:
    path = workcycle_end_token_state_path(agent_uid, batch_num)
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    return payload if isinstance(payload, dict) else None


def workcycle_next_work_items_spec_path(agent_uid: str, batch_num: int) -> Path:
    return AGENT_LOCAL_DIR / "agents" / agent_uid / "workcycles" / f"next-work-items-{batch_num}.json"


def workcycle_next_work_items_markdown_path(agent_uid: str, batch_num: int) -> Path:
    return AGENT_LOCAL_DIR / "agents" / agent_uid / "workcycles" / f"next-work-items-{batch_num}.md"


def build_next_work_items_payload(
    *,
    agent_role: str,
    compaction_detected: bool,
) -> dict[str, object]:
    return {
        "role": agent_role,
        "compaction_detected": compaction_detected,
    }


def write_next_work_items_outputs(
    *,
    agent_uid: str,
    batch_num: int,
    agent_role: str,
    compaction_detected: bool,
) -> tuple[Path, Path]:
    spec_path = workcycle_next_work_items_spec_path(agent_uid, batch_num)
    markdown_path = workcycle_next_work_items_markdown_path(agent_uid, batch_num)
    spec_path.parent.mkdir(parents=True, exist_ok=True)
    payload = build_next_work_items_payload(
        agent_role=agent_role,
        compaction_detected=compaction_detected,
    )
    spec_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    try:
        rendered = render_next_work_items_payload(payload)
    except NextWorkItemsError as exc:
        raise WorkCycleError(f"failed to render next work items: {exc}") from exc
    markdown_path.write_text(rendered, encoding="utf-8")
    return spec_path, markdown_path


def format_token_count(value: int) -> str:
    return f"{value:,} tok"


def format_ui_token_usage(value: int) -> str:
    if value < 1000:
        return f"{value} tok"
    return f"{round(value / 1000):,}K"


def estimate_cycle_token_spend(
    start_snapshot: dict[str, object] | None, end_snapshot: dict[str, object] | None
) -> int | None:
    if start_snapshot is None or end_snapshot is None:
        return None
    start_thread = start_snapshot.get("thread_id")
    end_thread = end_snapshot.get("thread_id")
    start_total = start_snapshot.get("input_tokens")
    end_total = end_snapshot.get("input_tokens")
    if not (
        isinstance(start_thread, str)
        and isinstance(end_thread, str)
        and start_thread == end_thread
        and isinstance(start_total, int)
        and isinstance(end_total, int)
        and end_total >= start_total
    ):
        return None
    return end_total - start_total


def after_work_token_usage_field(
    start_snapshot: dict[str, object] | None,
    end_snapshot: dict[str, object] | None,
    *,
    bootstrap_batch: bool = False,
) -> str | None:
    usage_part: str | None = None
    spent_part: str | None = None
    pre_boot_part: str | None = None

    estimated = estimate_cycle_token_spend(start_snapshot, end_snapshot)
    if estimated is not None and estimated > 0:
        spent_part = f"+{format_ui_token_usage(estimated)} this cycle est."

    if bootstrap_batch and start_snapshot is not None:
        start_total = start_snapshot.get("input_tokens")
        if isinstance(start_total, int) and start_total > 0:
            pre_boot_part = f"pre-boot +{format_ui_token_usage(start_total)}"

    if end_snapshot is not None:
        input_tokens = end_snapshot.get("input_tokens")
        context_window = end_snapshot.get("model_context_window")
        if isinstance(input_tokens, int) and input_tokens >= 0 and isinstance(context_window, int) and context_window > 0:
            usage_part = (
                f"usage {format_ui_token_usage(input_tokens)}/{format_ui_token_usage(context_window)}"
            )

    parts = [part for part in (usage_part, spent_part, pre_boot_part) if part]
    return " | ".join(parts) if parts else None


def thread_switch_diagnostic(
    start_snapshot: dict[str, object] | None, end_snapshot: dict[str, object] | None
) -> dict[str, str] | None:
    if start_snapshot is None or end_snapshot is None:
        return None
    start_thread = start_snapshot.get("thread_id")
    end_thread = end_snapshot.get("thread_id")
    if not (
        isinstance(start_thread, str)
        and start_thread.strip()
        and isinstance(end_thread, str)
        and end_thread.strip()
        and start_thread != end_thread
    ):
        return None
    return {
        "begin_thread_id": start_thread,
        "end_thread_id": end_thread,
    }


def load_git_state_snapshot(agent_uid: str, batch_num: int) -> dict[str, object] | None:
    path = workcycle_git_state_path(agent_uid, batch_num)
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    return payload if isinstance(payload, dict) else None


def normalize_cycle_tracked_path(raw_path: str) -> str | None:
    candidate = raw_path.strip()
    if not candidate:
        return None
    path_obj = Path(candidate)
    if path_obj.is_absolute():
        try:
            normalized = str(path_obj.resolve().relative_to(ROOT_DIR)).replace("\\", "/")
        except ValueError as exc:
            raise WorkCycleError(f"path is outside the repo root and cannot be recorded: {raw_path}") from exc
    else:
        normalized = str(path_obj).replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    normalized = normalized.strip("/")
    return normalized if is_cycle_tracked_path(normalized) else None


def load_owned_paths_snapshot(agent_uid: str, batch_num: int) -> set[str] | None:
    path = workcycle_owned_paths_path(agent_uid, batch_num)
    if not path.exists():
        return set()
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    if not isinstance(payload, dict):
        return None
    paths_value = payload.get("paths", [])
    if not isinstance(paths_value, list):
        return None
    normalized: set[str] = set()
    for raw_path in paths_value:
        if not isinstance(raw_path, str):
            return None
        normalized_path = normalize_cycle_tracked_path(raw_path)
        if normalized_path:
            normalized.add(normalized_path)
    return normalized


def store_owned_paths_snapshot(agent_uid: str, batch_num: int, paths: set[str]) -> Path:
    path = workcycle_owned_paths_path(agent_uid, batch_num)
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {"paths": sorted(paths)}
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def record_owned_paths(agent_uid: str, batch_num: int, raw_paths: list[str]) -> tuple[Path, set[str]]:
    existing = load_owned_paths_snapshot(agent_uid, batch_num)
    if existing is None:
        raise WorkCycleError(
            f"owned path snapshot is invalid for {agent_uid} batch {batch_num}; repair it before recording more paths"
        )
    updated = set(existing)
    for raw_path in raw_paths:
        normalized = normalize_cycle_tracked_path(raw_path)
        if normalized:
            updated.add(normalized)
    path = store_owned_paths_snapshot(agent_uid, batch_num, updated)
    return (path, updated)


def summarize_missing_recorded_paths(paths: set[str], *, limit: int = 5) -> str:
    ordered = sorted(paths)
    preview = ordered[:limit]
    suffix = "" if len(ordered) <= limit else ", ..."
    return ", ".join(preview) + suffix


def is_cycle_tracked_path(path: str) -> bool:
    normalized = path.strip().replace("\\", "/")
    if not normalized:
        return False
    if normalized.startswith(NON_CYCLE_TRACKED_PATH_PREFIXES):
        return False
    if "/__pycache__/" in f"/{normalized}/":
        return False
    if normalized.endswith(NON_CYCLE_TRACKED_PATH_SUFFIXES):
        return False
    return True


def cycle_has_source_changes(agent_uid: str, batch_num: int) -> bool | None:
    cycle_paths = cycle_tracked_paths(agent_uid, batch_num)
    if cycle_paths is None:
        return None
    return bool(cycle_paths)


def cycle_tracked_paths(agent_uid: str, batch_num: int) -> set[str] | None:
    snapshot = load_git_state_snapshot(agent_uid, batch_num)
    if snapshot is None or snapshot.get("available") is not True:
        return None

    base_head = snapshot.get("head")
    base_status_paths = {
        str(path)
        for path in snapshot.get("status_paths", [])
        if isinstance(path, str) and path.strip()
    }

    current_snapshot = capture_git_state_snapshot()
    if current_snapshot.get("available") is not True:
        return None

    current_head = current_snapshot.get("head")
    current_status_paths = {
        str(path)
        for path in current_snapshot.get("status_paths", [])
        if isinstance(path, str) and path.strip()
    }

    cycle_paths: set[str] = set()
    cycle_paths.update(current_status_paths - base_status_paths)

    if isinstance(base_head, str) and base_head and isinstance(current_head, str) and current_head:
        diff_proc = run_git(["diff", "--name-only", f"{base_head}..{current_head}"])
        if diff_proc.returncode != 0:
            return None
        cycle_paths.update(
            path.strip()
            for path in diff_proc.stdout.splitlines()
            if path.strip()
        )

    return {path for path in cycle_paths if is_cycle_tracked_path(path)}


def cycle_committed_tracked_paths(agent_uid: str, batch_num: int) -> set[str] | None:
    snapshot = load_git_state_snapshot(agent_uid, batch_num)
    if snapshot is None or snapshot.get("available") is not True:
        return None

    base_head = snapshot.get("head")
    current_snapshot = capture_git_state_snapshot()
    if current_snapshot.get("available") is not True:
        return None

    current_head = current_snapshot.get("head")
    if not (isinstance(base_head, str) and base_head and isinstance(current_head, str) and current_head):
        return set()

    diff_proc = run_git(["diff", "--name-only", f"{base_head}..{current_head}"])
    if diff_proc.returncode != 0:
        return None

    return {
        path.strip()
        for path in diff_proc.stdout.splitlines()
        if path.strip() and is_cycle_tracked_path(path.strip())
    }


def cycle_owned_tracked_paths(agent_uid: str, batch_num: int) -> set[str] | None:
    recorded_paths = load_owned_paths_snapshot(agent_uid, batch_num)
    if recorded_paths is None:
        return None

    owned_commits = cycle_owned_commit_refs(agent_uid, batch_num)
    if owned_commits is None:
        return None
    if not owned_commits:
        return recorded_paths

    owned_paths: set[str] = set(recorded_paths)
    for commit_ref in owned_commits:
        diff_proc = run_git(["diff-tree", "--no-commit-id", "--name-only", "-r", commit_ref])
        if diff_proc.returncode != 0:
            return None
        owned_paths.update(
            path.strip()
            for path in diff_proc.stdout.splitlines()
            if path.strip() and is_cycle_tracked_path(path.strip())
        )
    return owned_paths


def cycle_owned_commit_refs(agent_uid: str, batch_num: int) -> list[str] | None:
    snapshot = load_git_state_snapshot(agent_uid, batch_num)
    if snapshot is None or snapshot.get("available") is not True:
        return None

    base_head = snapshot.get("head")
    current_snapshot = capture_git_state_snapshot()
    if current_snapshot.get("available") is not True:
        return None

    current_head = current_snapshot.get("head")
    if not (isinstance(base_head, str) and base_head and isinstance(current_head, str) and current_head):
        return []

    rev_list_proc = run_git(["rev-list", "--reverse", f"{base_head}..{current_head}"])
    if rev_list_proc.returncode != 0:
        return None

    owned: list[str] = []
    for commit_ref in rev_list_proc.stdout.splitlines():
        commit_ref = commit_ref.strip()
        if not commit_ref:
            continue
        body_proc = run_git(["show", "-s", "--format=%B", commit_ref])
        if body_proc.returncode != 0:
            return None
        if f"Agent-Id: {agent_uid}" in body_proc.stdout:
            owned.append(commit_ref)
    return owned


def cycle_source_change_push_status(agent_uid: str, batch_num: int) -> dict[str, str | bool | None]:
    source_changes_present = cycle_has_source_changes(agent_uid, batch_num)
    if source_changes_present is not True:
        return {
            "required": False,
            "ok": True,
            "reason": "no source changes detected in the cycle",
            "remote_head": None,
        }
    owned_source_paths = cycle_owned_tracked_paths(agent_uid, batch_num)
    if owned_source_paths is None:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to determine whether cycle source changes belong to this agent",
            "remote_head": None,
        }
    if not owned_source_paths:
        return {
            "required": False,
            "ok": True,
            "reason": "no cycle-owned tracked-file changes detected; local-only changes do not block closeout",
            "remote_head": None,
        }
    recorded_owned_paths = load_owned_paths_snapshot(agent_uid, batch_num)
    if recorded_owned_paths is None:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to read recorded owned-path entries for this cycle",
            "remote_head": None,
        }
    missing_recorded_paths = owned_source_paths - recorded_owned_paths
    if missing_recorded_paths:
        missing_summary = summarize_missing_recorded_paths(missing_recorded_paths)
        return {
            "required": True,
            "ok": False,
            "reason": (
                "cycle-owned tracked-file changes are missing record-paths entries: "
                f"{missing_summary}; run `python3 scripts/agent_work_cycle.py record-paths {agent_uid} <path>...` "
                "before closeout"
            ),
            "remote_head": None,
        }
    remote_name_proc = run_git(["remote", "get-url", "origin"])
    origin_available = remote_name_proc.returncode == 0

    committed_source_paths = cycle_committed_tracked_paths(agent_uid, batch_num)
    if committed_source_paths is None:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to determine whether cycle file changes were committed",
            "remote_head": None,
        }

    if not committed_source_paths:
        if not origin_available:
            return {
                "required": False,
                "ok": True,
                "reason": "origin/main push verification unavailable; skipping source push guard",
                "remote_head": None,
            }
        return {
            "required": True,
            "ok": False,
            "reason": "cycle-owned tracked-file changes are still uncommitted; commit and push them first",
            "remote_head": None,
        }

    owned_commits = cycle_owned_commit_refs(agent_uid, batch_num)
    if owned_commits is None:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to resolve cycle-owned commits for push verification",
            "remote_head": None,
        }
    if not owned_commits:
        return {
            "required": False,
            "ok": True,
            "reason": "no cycle-owned tracked-file commits detected; foreign local commits do not block closeout",
            "remote_head": None,
        }

    if not origin_available:
        return {
            "required": False,
            "ok": True,
            "reason": "origin/main push verification unavailable; skipping source push guard",
            "remote_head": None,
        }

    current_snapshot = capture_git_state_snapshot()
    if current_snapshot.get("available") is not True:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to capture current git state",
            "remote_head": None,
        }

    current_head = current_snapshot.get("head")
    current_status_paths = {
        str(path)
        for path in current_snapshot.get("status_paths", [])
        if isinstance(path, str) and path.strip()
    }
    current_owned_status_paths = {path for path in current_status_paths if path in owned_source_paths}
    if any(path not in committed_source_paths for path in current_owned_status_paths):
        return {
            "required": True,
            "ok": False,
            "reason": "cycle-owned tracked-file changes are still uncommitted; commit and push them first",
            "remote_head": None,
        }
    if current_owned_status_paths:
        return {
            "required": True,
            "ok": False,
            "reason": "cycle-owned tracked-file changes are still present in the worktree; commit and push them first",
            "remote_head": None,
        }

    if not isinstance(current_head, str) or not current_head:
        return {
            "required": True,
            "ok": False,
            "reason": "unable to resolve HEAD for push verification",
            "remote_head": None,
        }

    remote_proc = run_git(["ls-remote", "--exit-code", "origin", "refs/heads/main"])
    if remote_proc.returncode != 0:
        detail = remote_proc.stderr.strip() or remote_proc.stdout.strip() or "unable to resolve origin/main"
        return {
            "required": True,
            "ok": False,
            "reason": detail,
            "remote_head": None,
        }

    remote_head = remote_proc.stdout.split()[0].strip() if remote_proc.stdout.strip() else ""
    if not remote_head:
        return {
            "required": True,
            "ok": False,
            "reason": "origin/main did not return a remote head sha",
            "remote_head": None,
        }

    push_target = owned_commits[-1]
    ancestor_proc = run_git(["merge-base", "--is-ancestor", push_target, remote_head])
    if ancestor_proc.returncode == 0:
        return {
            "required": True,
            "ok": True,
            "reason": (
                "HEAD is reachable from origin/main"
                if push_target == current_head
                else "latest cycle-owned source commit is reachable from origin/main"
            ),
            "remote_head": remote_head,
        }

    if ancestor_proc.returncode == 1:
        return {
            "required": True,
            "ok": False,
            "reason": (
                "HEAD is not yet reachable from origin/main; push your agent commit first"
                if push_target == current_head
                else "latest cycle-owned source commit is not yet reachable from origin/main; push your agent commit first"
            ),
            "remote_head": remote_head,
        }

    detail = ancestor_proc.stderr.strip() or ancestor_proc.stdout.strip() or "merge-base verification failed"
    return {
        "required": True,
        "ok": False,
        "reason": detail,
        "remote_head": remote_head,
    }


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
    checklist_path: Path, *, batch_num: int, source_changes_present: bool | None = None
) -> list[tuple[str, str]]:
    """Return (item_id, reason) for scrutinized items marked `not-needed` (`[-]`).

    Scrutinized items are those that should almost always be `checked` during
    real implementation work.  Marking them `not-needed` without genuine cause
    is the main mechanism by which agents silently skip required steps.
    """
    scrutinized = dict(SCRUTINIZED_NOT_NEEDED_ITEMS)
    # Batch 1 is a bootstrap-only cycle: no source files are changed, so
    # files-changed-summary is legitimately not-needed.
    # reply-with-plan-and-status is auto-marked not-needed by begin for batch 1.
    if batch_num == 1:
        scrutinized.pop("workflow.reply-with-plan-and-status", None)
        scrutinized.pop("workflow.files-changed-summary", None)
    elif source_changes_present is False:
        scrutinized.pop("workflow.files-changed-summary", None)

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


def emit_push_status_summary(push_status: dict[str, str | bool | None]) -> None:
    required = push_status.get("required") is True
    ok = push_status.get("ok") is True
    reason = str(push_status.get("reason") or "")
    remote_head = push_status.get("remote_head")
    print(f"source_push_required: {str(required).lower()}")
    print(f"source_push_ok: {str(ok).lower()}")
    if remote_head:
        print(f"source_push_remote_head: {remote_head}")
    print(f"source_push_reason: {reason}")


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


def emit_blocked_closeout_summary(guard_result: dict[str, object] | None) -> None:
    blocked = bool(guard_result and guard_result.get("blocked") is True)
    print(f"blocked_closeout: {str(blocked).lower()}")
    if not blocked:
        return
    block = guard_result.get("block")
    if not isinstance(block, dict):
        print("blocked_closeout_reason: unknown")
        return
    reason = block.get("reason")
    detected_at = block.get("detected_at")
    handoff_path = block.get("handoff_path")
    print(f"blocked_closeout_reason: {reason if isinstance(reason, str) and reason else 'unknown'}")
    if isinstance(detected_at, str) and detected_at.strip():
        print(f"blocked_closeout_detected_at: {detected_at}")
    if isinstance(handoff_path, str) and handoff_path.strip():
        print(f"blocked_closeout_handoff: {handoff_path}")


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


def emit_checklist_gc_summary(result: dict[str, object] | None, *, error: str | None = None) -> None:
    if error is not None:
        print("agent_checklist_gc_status: error")
        print(f"agent_checklist_gc_error: {error}")
        return
    if result is None:
        return
    print("agent_checklist_gc_status: ok")
    print(f"agent_checklist_gc_keep_workcycle_batches: {result['keep_workcycle_batches']}")
    print(f"agent_checklist_gc_deleted: {result['deleted_count']}")
    deleted = result.get("deleted")
    if isinstance(deleted, list) and deleted:
        print("agent_checklist_gc_deleted_paths:")
        for record in deleted:
            if isinstance(record, dict):
                print(
                    f"  - {record['path']} ({record['agent_uid']} batch {record['batch']})"
                )


def main() -> int:
    args = parse_args()
    phase_timings: dict[str, float] | None = {} if args.phase_timings else None
    if args.stage == "record-paths":
        status_payload = timed_call(phase_timings, "resolve_agent_status", run_registry, "status", args.agent_ref)
        agents = status_payload.get("agents")
        if not isinstance(agents, list) or len(agents) != 1:
            raise WorkCycleError(f"unable to resolve agent state for {args.agent_ref}")
        agent = agents[0]
        agent_uid = agent.get("agent_uid") or args.agent_ref
        display_id = agent.get("current_display_id")
        status = agent.get("status")
        if status != "active":
            raise WorkCycleError(
                f"record-paths requires an active work cycle; agent {args.agent_ref} is currently {status or 'unknown'}"
            )
        latest_batch = latest_agents_workcycle_batch_num(agent_uid)
        if latest_batch is None:
            raise WorkCycleError(f"no work cycle batch exists yet for {agent_uid}; run begin first")
        stored_path, owned_paths = timed_call(
            phase_timings, "record_owned_paths", record_owned_paths, agent_uid, latest_batch, args.paths
        )
        print(f"agent_uid: {agent_uid}")
        if isinstance(display_id, str) and display_id.strip():
            print(f"display_id: {display_id}")
        print(f"batch_num: {latest_batch}")
        print(f"owned_paths_snapshot: {stored_path.relative_to(ROOT_DIR)}")
        print(f"recorded_paths: {len(owned_paths)}")
        if owned_paths:
            print("owned_path_entries:")
            for path in sorted(owned_paths):
                print(f"  - {path}")
        emit_phase_timings(phase_timings)
        return 0

    registry_command = "touch" if args.stage == "begin" else "finish"
    registry_scope = args.scope if args.stage == "begin" else None
    payload = timed_call(
        phase_timings, "registry_transition", run_registry, registry_command, args.agent_ref, scope=registry_scope
    )
    agent_uid = payload.get("agent_uid") or args.agent_ref
    display_id = payload.get("display_id")
    agent_role = timed_call(phase_timings, "resolve_agent_role", resolve_agent_role, agent_uid)
    model_id = timed_call(phase_timings, "resolve_agent_model_id", resolve_agent_model_id, agent_uid)
    current_model, current_effort = timed_call(
        phase_timings, "resolve_codex_metadata", resolve_current_codex_metadata
    )
    label_model = current_model or model_id
    guard_result = timed_call(phase_timings, "agent_guard_check", check_agent, agent_uid)

    checklist_paths: list[Path] = []
    unchecked_by_path: dict[Path, list[str]] = {}
    bootstrap_batch = False

    if args.stage == "begin":
        checklist_result = timed_call(
            phase_timings,
            "materialize_workcycle_checklist",
            materialize_checklist,
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
        timed_call(phase_timings, "mark_begin_checklist_defaults", set_checklist_item_states, workcycle_path, updates)
        batch_num = checklist_result.get("batch_num")
        if isinstance(batch_num, int):
            started_at = perf_counter()
            store_git_state_snapshot(agent_uid, batch_num)
            store_owned_paths_snapshot(agent_uid, batch_num, set())
            begin_token_snapshot = store_token_usage_snapshot(agent_uid, batch_num)
            if phase_timings is not None:
                phase_timings["store_begin_snapshots"] = round(perf_counter() - started_at, 6)
        else:
            begin_token_snapshot = None
        compaction = timed_call(
            phase_timings, "detect_begin_compaction", detect_compaction_event, begin_token_snapshot
        )
        print(f"workcycle_output: {workcycle_output}")
        role_source = role_checklist_source_path(agent_role)
        role_prefix = split_checklist_prefix_for(role_source)
        if role_prefix is not None and role_source.exists():
            role_checklist_result = timed_call(
                phase_timings,
                "materialize_role_workcycle_checklist",
                materialize_checklist,
                agent_uid=agent_uid,
                display_id=display_id,
                source_path=role_source,
                output_path=split_workcycle_checklist_path(agent_uid, role_prefix, 1),
                section="workcycle",
            )
            role_workcycle_output = role_checklist_result.get("output")
            if isinstance(role_workcycle_output, str):
                print(f"role_workcycle_output: {role_workcycle_output}")
        if "batch_num" in checklist_result:
            print(f"batch_num: {checklist_result['batch_num']}")
        if compaction is not None:
            handoff = timed_call(
                phase_timings,
                "create_compaction_handoff",
                create_compaction_handoff,
                agent_uid,
                agent_role=agent_role,
                scope=args.scope or "compact-context-abort",
                detection=compaction,
            )
            timed_call(
                phase_timings,
                "record_compaction_block",
                record_compaction_block,
                agent_uid,
                scope=args.scope or "compact-context-abort",
                detection=compaction,
                handoff_mailbox=handoff["mailbox"],
            )
            finish_payload = timed_call(
                phase_timings, "compaction_finish_transition", run_registry, "finish", agent_uid
            )
            emit_registry_summary(finish_payload)
            print("compact_context_detected: true")
            print(f"compaction_timestamp: {compaction['timestamp']}")
            print(f"compaction_rollout_path: {compaction['rollout_path']}")
            print(f"handoff_created: {handoff['mailbox']}")
            print(
                "alert: compact context detected, we better open a new chat for better performance, and handoff is ready."
            )
            emit_phase_timings(phase_timings)
            return COMPACTION_ABORT_EXIT_CODE
        closeout_command = f"python3 scripts/agent_work_cycle.py end {agent_uid}"
        if args.phase_timings:
            closeout_command += " --phase-timings"
        print(f"closeout_command: {closeout_command}")
    else:
        if guard_result.get("blocked") is True and not args.blocked_closeout:
            raise WorkCycleError(
                "agent execution is blocked; use "
                f"`python3 scripts/agent_work_cycle.py end {agent_uid} --blocked-closeout` "
                "to close out this thread explicitly without treating it as a normal completion."
            )
        if args.blocked_closeout and guard_result.get("blocked") is not True:
            raise WorkCycleError(
                "blocked closeout requested, but the agent is not currently blocked by agent_guard; "
                "blocked closeout is only valid when `agent_guard.py check` reports `blocked: true`; "
                f"use `python3 scripts/agent_work_cycle.py end {agent_uid}` instead."
            )

        latest_batch = latest_agents_workcycle_batch_num(agent_uid)
        if latest_batch is None:
            raise WorkCycleError(f"no workcycle checklist found for {agent_uid}")

        workcycle_path = agents_workcycle_checklist_path(agent_uid, latest_batch)
        timed_call(
            phase_timings,
            "mark_finish_workcycle_item",
            set_checklist_item_states,
            workcycle_path,
            [("workflow.finish-work-cycle", "checked")],
        )
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
        started_at = perf_counter()
        start_token_snapshot = load_token_usage_snapshot(agent_uid, latest_batch)
        end_token_snapshot = store_end_token_usage_snapshot_once(agent_uid, latest_batch)
        thread_switch = thread_switch_diagnostic(
            start_token_snapshot, end_token_snapshot
        )
        after_timestamp = (
            str(start_token_snapshot.get("timestamp"))
            if isinstance(start_token_snapshot, dict)
            and isinstance(start_token_snapshot.get("timestamp"), str)
            else None
        )
        start_rollout_path = (
            Path(str(start_token_snapshot.get("rollout_path")))
            if isinstance(start_token_snapshot, dict)
            and isinstance(start_token_snapshot.get("rollout_path"), str)
            and str(start_token_snapshot.get("rollout_path")).strip()
            else None
        )
        if start_rollout_path is not None:
            end_compaction = detect_compaction_event_in_rollout_path(
                start_rollout_path,
                after_timestamp=after_timestamp,
            )
        else:
            end_compaction = detect_compaction_event(
                end_token_snapshot,
                after_timestamp=after_timestamp,
            )
        if phase_timings is not None:
            phase_timings["token_snapshot_diagnostics"] = round(perf_counter() - started_at, 6)
        next_work_items_spec, next_work_items_markdown = write_next_work_items_outputs(
            agent_uid=agent_uid,
            batch_num=latest_batch,
            agent_role=agent_role,
            compaction_detected=end_compaction is not None,
        )
        print(
            build_message(
                stage,
                agent=label,
                agent_uid=agent_uid,
                model_id=label_model,
                reasoning_effort=current_effort,
                scope=args.scope,
                token_usage=after_work_token_usage_field(
                    start_token_snapshot,
                    end_token_snapshot,
                    bootstrap_batch=bootstrap_batch,
                ),
                status_note="compaction detected" if end_compaction is not None else None,
            )
        )
        if end_compaction is not None:
            print("compact_context_detected_before_after_work: true")
            print(f"compaction_timestamp: {end_compaction['timestamp']}")
            print(f"compaction_rollout_path: {end_compaction['rollout_path']}")
            print(
                "alert: compact context detected before after-work closeout; open a fresh chat before continuing."
            )
        if thread_switch is not None:
            print("thread_switch_detected_before_after_work: true")
            print(f"begin_thread_id: {thread_switch['begin_thread_id']}")
            print(f"end_thread_id: {thread_switch['end_thread_id']}")
            print(
                "warning: begin/end Codex thread ids differ during after-work closeout; diagnostics may span a thread switch."
            )
        print(f"next_work_items_spec: {next_work_items_spec.relative_to(ROOT_DIR)}")
        print(f"next_work_items_markdown: {next_work_items_markdown.relative_to(ROOT_DIR)}")
        print(
            "next_work_items_render_command: "
            f"python3 scripts/render_next_work_items.py {next_work_items_spec.relative_to(ROOT_DIR)}"
        )
        print(
            "next_work_items_paste_rule: paste the rendered Markdown verbatim after the After work line; "
            "edit the JSON spec first if custom options are needed, then rerun the render command."
        )

        started_at = perf_counter()
        for path in checklist_paths:
            unchecked_by_path[path] = scan_unchecked_items(path)
        if phase_timings is not None:
            phase_timings["scan_checklists"] = round(perf_counter() - started_at, 6)

        # Scrutinize not-needed markings on high-value required items.
        not_needed_violations: list[tuple[str, str]] = []
        started_at = perf_counter()
        owned_source_paths = cycle_owned_tracked_paths(agent_uid, latest_batch)
        source_changes_present = None if owned_source_paths is None else bool(owned_source_paths)
        push_status = cycle_source_change_push_status(agent_uid, latest_batch)
        for path in checklist_paths:
            not_needed_violations.extend(
                scan_scrutinized_not_needed_items(
                    path,
                    batch_num=latest_batch,
                    source_changes_present=source_changes_present,
                )
            )
        if phase_timings is not None:
            phase_timings["push_and_not_needed_checks"] = round(perf_counter() - started_at, 6)

        started_at = perf_counter()
        mailbox_path = resolve_agent_mailbox_path(agent_uid)
        open_handoff_lines = scan_open_handoffs(mailbox_path, agent_role=agent_role)

        # Scope consistency: registry scope vs open handoff scope.
        registry_scope = resolve_agent_scope(agent_uid)
        handoff_scope = extract_open_handoff_scope(mailbox_path, agent_role=agent_role)
        if phase_timings is not None:
            phase_timings["mailbox_and_scope_scan"] = round(perf_counter() - started_at, 6)

        emit_checklist_summary(
            checklist_paths=checklist_paths,
            unchecked_by_path=unchecked_by_path,
            bootstrap_batch=bootstrap_batch,
        )
        emit_not_needed_scrutiny_summary(not_needed_violations)
        emit_push_status_summary(push_status)
        emit_scope_consistency_summary(registry_scope=registry_scope, handoff_scope=handoff_scope)
        emit_mailbox_summary(mailbox_path, open_handoff_lines)
        emit_blocked_closeout_summary(guard_result if args.blocked_closeout else None)
        shared_fallback_records = timed_call(
            phase_timings, "shared_fallback_scan", scan_shared_fallback_mailboxes
        )
        emit_shared_fallback_summary(shared_fallback_records)
        mailbox_gc_result: dict[str, object] | None = None
        mailbox_gc_error: str | None = None
        try:
            mailbox_gc_result = timed_call(
                phase_timings,
                "mailbox_gc",
                delete_stale_mailboxes,
                dry_run=False, min_age_days=DEFAULT_DELETE_AGE_DAYS
            )
        except MailboxGcError as exc:
            mailbox_gc_error = str(exc)
        emit_mailbox_gc_summary(mailbox_gc_result, error=mailbox_gc_error)
        checklist_gc_result: dict[str, object] | None = None
        checklist_gc_error: str | None = None
        try:
            checklist_gc_result = timed_call(
                phase_timings,
                "agent_checklist_gc",
                prune_agent_checklists,
                dry_run=False,
                keep_workcycle_batches=DEFAULT_KEEP_WORKCYCLE_BATCHES,
            )
        except AgentChecklistGcError as exc:
            checklist_gc_error = str(exc)
        emit_checklist_gc_summary(checklist_gc_result, error=checklist_gc_error)
        emit_phase_timings(phase_timings)

        same_role_open_count = len(open_handoff_lines["same_role"])
        other_role_open_count = len(open_handoff_lines["other_role"])
        mailbox_pending = other_role_open_count > 1
        if not bootstrap_batch:
            mailbox_pending = mailbox_pending or same_role_open_count != 1
        shared_fallback_pending = any(record["over_limit"] for record in shared_fallback_records)
        not_needed_pending = len(not_needed_violations) > 0
        push_pending = push_status.get("required") is True and push_status.get("ok") is not True
        if args.blocked_closeout:
            return 2 if mailbox_pending or shared_fallback_pending else 0
        return (
            2
            if any(unchecked_by_path.values())
            or mailbox_pending
            or shared_fallback_pending
            or not_needed_pending
            or push_pending
            else 0
        )

    emit_registry_summary(payload)

    stage = "before" if args.stage == "begin" else "after"
    label = display_id or args.agent_ref
    print(
        build_message(
            stage,
            agent=label,
            agent_uid=agent_uid,
            model_id=label_model,
            reasoning_effort=current_effort,
            scope=args.scope,
        )
    )
    emit_phase_timings(phase_timings)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except WorkCycleError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
