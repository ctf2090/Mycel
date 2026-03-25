#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from item_id_checklist_mark import ItemIdChecklistMarkError, update_checklist_items


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"
WORK_CYCLE_SCRIPT = ROOT_DIR / "scripts" / "agent_work_cycle.py"
CODEX_THREAD_METADATA_SCRIPT = ROOT_DIR / "scripts" / "codex_thread_metadata.py"
FAST_PATH_STEPS = [
    "scan the repo root with ls",
    "read AGENTS-LOCAL.md if it exists, then read .agent-local/dev-setup-status.md",
    "read docs/ROLE-CHECKLISTS/README.md, docs/AGENT-REGISTRY.md, and .agent-local/agents.json",
    "run scripts/agent_bootstrap.py <role> --model-id <model_id> or scripts/agent_bootstrap.py auto --model-id <model_id>",
]
DEFERRED_READS_COMMON = [
    "ROADMAP.md and other broad planning docs",
    "full registry dumps beyond confirming active peers and the claimed agent state",
    "broad markdown sweeps outside the task area",
]
DEFERRED_READS_BY_ROLE = {
    "coding": [
        "full mailbox scans unless the chat is resuming, taking over, or working an overlapping coding scope",
    ],
    "delivery": [
        "broad roadmap/checklist sweeps and mailbox scans unrelated to the active CI/process scope",
    ],
    "doc": [
        "planning-sync mailbox scans and scripts/check-plan-refresh.py until the doc work item actually starts",
    ],
}
ROLE_HANDOFF_HEADINGS = {
    "coding": "Work Continuation Handoff",
    "delivery": "Delivery Continuation Note",
    "doc": "Doc Continuation Note",
}
BOOTSTRAP_PREFLIGHT_REQUIREMENTS = ["python3", "git", "rg", "sed"]
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))
DATE_PATTERN = "%Y-%m-%d %H:%M UTC+8"
STATUS_PATTERN = re.compile(r"^- Status:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
DATE_FIELD_PATTERN = re.compile(r"^- Date:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SCOPE_PATTERN = re.compile(r"^- Scope:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SOURCE_AGENT_PATTERN = re.compile(r"^- Source agent:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
SOURCE_ROLE_PATTERN = re.compile(r"^- Source role:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
NEXT_STEP_PATTERN = re.compile(r"^- Next suggested step:\s*(.*?)(?=^-\s|\Z)", re.MULTILINE | re.DOTALL)
COMPACTION_ABORT_MARKERS = (
    "Compact context detected in the current chat thread before work started",
    "Compaction event detected at ",
)
LATEST_CI_GH_FIELDS = [
    "databaseId",
    "status",
    "conclusion",
    "workflowName",
    "displayTitle",
    "headSha",
    "updatedAt",
]


class Section:
    def __init__(self, heading: str, body: str, order: int) -> None:
        self.heading = heading
        self.body = body
        self.order = order


class BootstrapError(Exception):
    pass


def read_text_if_exists(path: Path) -> str | None:
    if not path.exists():
        return None
    return path.read_text(encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_bootstrap.py",
        description="Claim, start, and begin a fresh agent work cycle in one command.",
    )
    parser.add_argument(
        "role",
        nargs="?",
        default="auto",
        choices=["auto", "coding", "delivery", "doc"],
        help="agent role to claim; defaults to auto",
    )
    parser.add_argument("--scope", default="pending scope", help="scope label for the new agent")
    parser.add_argument("--assigned-by", default="user", help="registry assigned_by value")
    parser.add_argument("--json", action="store_true", help="emit a combined JSON payload")
    parser.add_argument("--model-id", required=True, dest="model_id", help="model identifier to record in the registry and include in timestamp lines")
    parser.add_argument(
        "--concise",
        action="store_true",
        help="emit a shorter text summary suited for relaying the bootstrap result to the user",
    )
    return parser.parse_args()


def run_command(command: list[str]) -> str:
    proc = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or "command failed"
        raise BootstrapError(message)
    return proc.stdout


def run_json_command(command: list[str]) -> dict[str, Any]:
    stdout = run_command(command)
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise BootstrapError(f"expected JSON output from {' '.join(command)}") from exc
    if not isinstance(payload, dict):
        raise BootstrapError(f"expected JSON object output from {' '.join(command)}")
    return payload


def run_json_array_command(command: list[str]) -> list[dict[str, Any]]:
    stdout = run_command(command)
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise BootstrapError(f"expected JSON output from {' '.join(command)}") from exc
    if not isinstance(payload, list):
        raise BootstrapError(f"expected JSON array output from {' '.join(command)}")
    return [item for item in payload if isinstance(item, dict)]


def run_registry_json(*args: str) -> dict[str, Any]:
    return run_json_command([sys.executable, str(REGISTRY_SCRIPT), *args, "--json"])


def run_bootstrap_runtime_preflight() -> bool:
    command = [
        sys.executable,
        str(ROOT_DIR / "scripts" / "check-runtime-preflight.py"),
        *[item for requirement in BOOTSTRAP_PREFLIGHT_REQUIREMENTS for item in ("--require", requirement)],
    ]
    proc = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or "bootstrap runtime preflight failed"
        raise BootstrapError(message)
    return True


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


def load_registry() -> dict[str, Any]:
    try:
        return json.loads((ROOT_DIR / ".agent-local" / "agents.json").read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise BootstrapError("missing registry file after bootstrap claim") from exc
    except json.JSONDecodeError as exc:
        raise BootstrapError("invalid registry JSON after bootstrap claim") from exc


def resolve_repo_path(path_value: str) -> Path:
    path = Path(path_value)
    if not path.is_absolute():
        path = ROOT_DIR / path
    return path


def set_checklist_item_states(checklist_path: Path, updates: list[tuple[str, str]]) -> None:
    if not checklist_path.exists() or not updates:
        return
    try:
        update_checklist_items(checklist_path, updates)
    except ItemIdChecklistMarkError as exc:
        raise BootstrapError(str(exc)) from exc


def parse_key_value_lines(output: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in output.splitlines():
        if not line or line.startswith("[") or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key] = value
    return parsed


def find_timestamp_line(output: str) -> str | None:
    for line in output.splitlines():
        if line.startswith("[") and "] " in line:
            return line
    return None


def current_or_last_display_id(entry: dict[str, Any]) -> str | None:
    current = entry.get("current_display_id")
    if isinstance(current, str) and current.strip():
        return current
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


def resolve_mailbox_path(mailbox_value: str) -> Path:
    path = Path(mailbox_value)
    if not path.is_absolute():
        path = ROOT_DIR / path
    return path


def section_chunks(text: str) -> list[Section]:
    sections: list[Section] = []
    current_heading = ""
    current_lines: list[str] = []
    for line in text.splitlines():
        if line.startswith("## "):
            if current_heading or current_lines:
                sections.append(Section(current_heading, "\n".join(current_lines).strip(), len(sections)))
            current_heading = line[3:].strip()
            current_lines = []
            continue
        current_lines.append(line)
    if current_heading or current_lines:
        sections.append(Section(current_heading, "\n".join(current_lines).strip(), len(sections)))
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
    return parsed.replace(tzinfo=TAIPEI_TIMEZONE)


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
        else:
            lines.append(line)
    return lines


def extract_latest_open_handoff(mailbox_path: Path, *, role: str) -> dict[str, Any] | None:
    heading = ROLE_HANDOFF_HEADINGS.get(role)
    if heading is None or not mailbox_path.exists():
        return None
    text = mailbox_path.read_text(encoding="utf-8")
    candidates: list[tuple[tuple[int, int], dict[str, Any]]] = []
    for section in section_chunks(text):
        if section.heading != heading:
            continue
        status = match_group(STATUS_PATTERN, section.body)
        if status is None or status.lower() != "open":
            continue
        date_text = match_group(DATE_FIELD_PATTERN, section.body)
        parsed_date = parse_taipei_date(date_text)
        next_step_match = NEXT_STEP_PATTERN.search(section.body)
        next_steps = normalize_multiline_field(next_step_match.group(1).strip() if next_step_match else None)
        record = {
            "heading": section.heading,
            "status": status,
            "date": date_text,
            "scope": match_group(SCOPE_PATTERN, section.body),
            "source_agent": match_group(SOURCE_AGENT_PATTERN, section.body),
            "source_role": match_group(SOURCE_ROLE_PATTERN, section.body),
            "next_suggested_step": next_steps,
            "body": section.body,
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


def is_compaction_abort_handoff(handoff: dict[str, Any]) -> bool:
    body = handoff.get("body")
    if not isinstance(body, str) or not body.strip():
        return False
    return any(marker in body for marker in COMPACTION_ABORT_MARKERS)


def latest_same_role_handoff(registry: dict[str, Any], *, role: str, current_agent_uid: str) -> dict[str, Any] | None:
    agents = registry.get("agents")
    if not isinstance(agents, list):
        return None
    candidates: list[tuple[tuple[int, str, str], dict[str, Any]]] = []
    for entry in agents:
        if not isinstance(entry, dict):
            continue
        if entry.get("role") != role:
            continue
        if entry.get("status") == "active":
            continue
        agent_uid = entry.get("agent_uid")
        if not isinstance(agent_uid, str) or not agent_uid.strip() or agent_uid == current_agent_uid:
            continue
        mailbox_value = entry.get("mailbox")
        if not isinstance(mailbox_value, str) or not mailbox_value.strip():
            continue
        handoff = extract_latest_open_handoff(resolve_mailbox_path(mailbox_value), role=role)
        if handoff is None:
            continue
        if is_compaction_abort_handoff(handoff):
            continue
        display_id = current_or_last_display_id(entry) or agent_uid
        date_text = handoff.get("date")
        parsed_date = parse_taipei_date(date_text if isinstance(date_text, str) else None)
        sort_key = (
            int(parsed_date.timestamp()) if parsed_date is not None else -1,
            str(entry.get("last_touched_at") or entry.get("inactive_at") or ""),
            agent_uid,
        )
        candidates.append(
            (
                sort_key,
                {
                    "agent_uid": agent_uid,
                    "display_id": display_id,
                    "mailbox": mailbox_value,
                    "handoff": handoff,
                },
            )
        )
    if not candidates:
        return None
    candidates.sort(key=lambda item: item[0], reverse=True)
    return candidates[0][1]


def fast_path_steps_for_role(role: str) -> list[str]:
    steps = list(FAST_PATH_STEPS)
    if role in {"coding", "delivery"}:
        steps.append(
            "check the latest completed CI result for the previous push before implementation or delivery work"
        )
    return steps


def deferred_reads_for_role(role: str) -> list[str]:
    return DEFERRED_READS_COMMON + DEFERRED_READS_BY_ROLE.get(role, [])


def lookup_latest_completed_ci(role: str) -> dict[str, Any] | None:
    if role not in {"coding", "delivery"}:
        return None
    fields = ",".join(LATEST_CI_GH_FIELDS)
    command = [
        "gh",
        "run",
        "list",
        "--branch",
        "main",
        "--limit",
        "5",
        "--json",
        fields,
    ]
    try:
        runs = run_json_array_command(command)
    except BootstrapError as exc:
        return {
            "checked": False,
            "status": "unavailable",
            "message": str(exc),
        }

    for run in runs:
        if run.get("status") != "completed":
            continue
        return {
            "checked": True,
            "status": "completed",
            "databaseId": run.get("databaseId"),
            "workflowName": run.get("workflowName"),
            "displayTitle": run.get("displayTitle"),
            "conclusion": run.get("conclusion"),
            "headSha": run.get("headSha"),
            "updatedAt": run.get("updatedAt"),
        }

    return {
        "checked": False,
        "status": "missing",
        "message": "no completed GitHub Actions runs found on main",
    }


def next_actions_for_role(role: str, latest_ci: dict[str, Any] | None) -> list[str]:
    if role == "coding":
        latest_ci_status = latest_ci.get("status") if isinstance(latest_ci, dict) else None
        ci_action = (
            "re-run the latest completed CI lookup before implementation work because bootstrap could not confirm it"
            if latest_ci_status in {"unavailable", "missing"}
            else "use the latest completed CI result above as the baseline before choosing the next implementation slice"
        )
        return [
            ci_action,
            "defer mailbox scans unless the scope overlaps existing coding work, recovery, or takeover",
        ]
    if role == "delivery":
        latest_ci_status = latest_ci.get("status") if isinstance(latest_ci, dict) else None
        ci_action = (
            "re-run the latest completed CI lookup before delivery follow-up because bootstrap could not confirm it"
            if latest_ci_status in {"unavailable", "missing"}
            else "use the latest completed CI result above as the baseline before triaging delivery work"
        )
        return [
            ci_action,
            "defer broad roadmap/checklist reading unless the active delivery scope needs doc follow-up",
        ]
    if role == "doc":
        return [
            "wait for the concrete doc task before running planning-sync mailbox scans or scripts/check-plan-refresh.py",
            "review open Dependabot pull requests first to assess dependency-update doc or checklist impact",
            "review open human-authored product pull requests before choosing the first doc follow-up item",
        ]
    return []


def handoff_review_action(role: str, same_role_handoff: dict[str, Any] | None) -> str | None:
    if same_role_handoff is None:
        return None
    display_id = same_role_handoff.get("display_id") or same_role_handoff.get("agent_uid") or "same-role agent"
    source_role = same_role_handoff.get("role") or role or "unknown"
    handoff = same_role_handoff.get("handoff")
    if not isinstance(handoff, dict):
        return None
    scope = handoff.get("scope") or "the latest same-role scope"
    next_steps = handoff.get("next_suggested_step")
    next_step = next_steps[0] if isinstance(next_steps, list) and next_steps else None
    if isinstance(next_step, str) and next_step.strip():
        return (
            f"review the latest same-role handoff from {display_id} "
            f"(role={source_role}) for scope {scope} and consider this follow-up first: {next_step.strip()}"
        )
    return f"review the latest same-role handoff from {display_id} (role={source_role}) for scope {scope} before choosing the first work item"


def persist_same_role_handoff_review(bootstrap_checklist_path: Path, same_role_handoff: dict[str, Any] | None) -> bool:
    if same_role_handoff is None or not bootstrap_checklist_path.exists():
        return False
    handoff = same_role_handoff.get("handoff")
    if not isinstance(handoff, dict):
        return False
    display_id = same_role_handoff.get("display_id") or same_role_handoff.get("agent_uid") or "unknown"
    scope = handoff.get("scope") or "unknown"
    date_text = handoff.get("date") or "unknown"
    source_agent = handoff.get("source_agent") or "unknown"
    source_role = handoff.get("source_role") or same_role_handoff.get("role") or "unknown"
    next_steps = handoff.get("next_suggested_step")
    next_steps_list = [step for step in next_steps if isinstance(step, str) and step.strip()] if isinstance(next_steps, list) else []

    marker = "## Latest Same-Role Handoff Review"
    original = bootstrap_checklist_path.read_text(encoding="utf-8")
    trimmed = original.rstrip()
    if marker in trimmed:
        trimmed = trimmed.split(marker, 1)[0].rstrip()

    lines = [
        trimmed,
        "",
        marker,
        "",
        f"- Reviewed agent: `{display_id}`",
        f"- Handoff source agent: `{source_agent}`",
        f"- Handoff source role: `{source_role}`",
        f"- Handoff date: `{date_text}`",
        f"- Handoff scope: `{scope}`",
    ]
    if next_steps_list:
        lines.append("- Handoff next suggested step:")
        for step in next_steps_list:
            lines.append(f"  - {step}")
    else:
        lines.append("- Handoff next suggested step:")
        lines.append("  - none recorded")
    bootstrap_checklist_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return True


def scan_root_layout() -> list[str]:
    return sorted(entry.name for entry in ROOT_DIR.iterdir())


def dev_setup_status_updates() -> list[tuple[str, str]]:
    status_path = ROOT_DIR / ".agent-local" / "dev-setup-status.md"
    text = read_text_if_exists(status_path)
    if text is None:
        return [
            ("bootstrap.read-dev-setup-status", "not-needed"),
            ("bootstrap.skip-dev-setup-when-ready", "not-needed"),
        ]

    updates: list[tuple[str, str]] = [("bootstrap.read-dev-setup-status", "checked")]
    if "- Status: ready" in text:
        updates.extend(
            [
                ("bootstrap.skip-dev-setup-when-ready", "checked"),
                ("bootstrap.refresh-dev-setup-when-needed", "not-needed"),
                ("bootstrap.dev-setup-template", "not-needed"),
            ]
        )
    return updates


def record_bootstrap_checklist_progress(
    bootstrap_checklist_path: Path,
    *,
    role_arg: str,
    same_role_handoff: dict[str, Any] | None,
    runtime_preflight_ok: bool,
) -> None:
    updates: list[tuple[str, str]] = [("bootstrap.repo-layout", "checked")]

    if read_text_if_exists(ROOT_DIR / "docs" / "ROLE-CHECKLISTS" / "README.md") is not None:
        updates.append(("bootstrap.read-role-checklists", "checked"))

    registry_doc = read_text_if_exists(ROOT_DIR / "docs" / "AGENT-REGISTRY.md")
    registry_json = read_text_if_exists(ROOT_DIR / ".agent-local" / "agents.json")
    if registry_doc is not None and registry_json is not None:
        updates.append(("bootstrap.read-agent-registry", "checked"))

    updates.extend(dev_setup_status_updates())
    updates.append(
        (
            "bootstrap.runtime-preflight",
            "checked" if runtime_preflight_ok else "not-needed",
        )
    )
    updates.append(("bootstrap.claim-fresh-agent-for-new-chat", "checked"))
    updates.append(
        (
            "bootstrap.no-confirm-after-role-read",
            "checked" if role_arg != "auto" else "not-needed",
        )
    )
    updates.append(
        (
            "bootstrap.claim-auto",
            "checked" if role_arg == "auto" else "not-needed",
        )
    )
    updates.append(
        (
            "bootstrap.review-latest-same-role-handoff",
            "checked" if same_role_handoff is not None else "not-needed",
        )
    )
    set_checklist_item_states(bootstrap_checklist_path, updates)


def build_result(args: argparse.Namespace) -> dict[str, Any]:
    scan_root_layout()
    read_text_if_exists(ROOT_DIR / "docs" / "ROLE-CHECKLISTS" / "README.md")
    read_text_if_exists(ROOT_DIR / "docs" / "AGENT-REGISTRY.md")
    runtime_preflight_ok = run_bootstrap_runtime_preflight()
    claim_payload = run_registry_json(
        "claim", args.role, "--scope", args.scope, "--assigned-by", args.assigned_by, "--model-id", args.model_id
    )
    agent_uid = claim_payload.get("agent_uid")
    if not isinstance(agent_uid, str) or not agent_uid.strip():
        raise BootstrapError("claim did not return agent_uid")

    start_payload = run_registry_json("start", agent_uid)
    begin_output = run_command(
        [sys.executable, str(WORK_CYCLE_SCRIPT), "begin", agent_uid, "--scope", args.scope]
    )
    begin_fields = parse_key_value_lines(begin_output)
    before_work_line = find_timestamp_line(begin_output)
    repo_status = run_command(["git", "status", "-sb"]).splitlines()
    registry = load_registry()
    role = claim_payload.get("role")
    if not isinstance(role, str) or not role.strip():
        raise BootstrapError("claim did not return role")
    current_model, current_effort = resolve_current_codex_metadata()
    claimed_model = current_model or args.model_id

    latest_completed_ci = lookup_latest_completed_ci(role)
    same_role_handoff = latest_same_role_handoff(registry, role=role, current_agent_uid=agent_uid)
    next_actions = next_actions_for_role(role, latest_completed_ci)
    handoff_action = handoff_review_action(role, same_role_handoff)
    if handoff_action is not None:
        next_actions.append(handoff_action)
    bootstrap_output = start_payload.get("bootstrap_output")
    persisted_handoff_review = False
    if isinstance(bootstrap_output, str) and bootstrap_output.strip():
        bootstrap_checklist_path = resolve_repo_path(bootstrap_output)
        persisted_handoff_review = persist_same_role_handoff_review(
            bootstrap_checklist_path,
            same_role_handoff,
        )
        record_bootstrap_checklist_progress(
            bootstrap_checklist_path,
            role_arg=args.role,
            same_role_handoff=same_role_handoff,
            runtime_preflight_ok=runtime_preflight_ok,
        )

    result: dict[str, Any] = {
        "agent_uid": agent_uid,
        "display_id": claim_payload.get("display_id"),
        "role": role,
        "scope": claim_payload.get("scope"),
        "assigned_by": claim_payload.get("assigned_by"),
        "assigned_at": claim_payload.get("assigned_at"),
        "mailbox": claim_payload.get("mailbox"),
        "mailbox_link": start_payload.get("mailbox_link"),
        "confirmed_at": start_payload.get("confirmed_at"),
        "bootstrap_output": start_payload.get("bootstrap_output"),
        "bootstrap_created": start_payload.get("bootstrap_created"),
        "workcycle_output": begin_fields.get("workcycle_output"),
        "batch_num": begin_fields.get("batch_num"),
        "closeout_command": begin_fields.get("closeout_command"),
        "previous_status": begin_fields.get("previous_status"),
        "current_status": begin_fields.get("current_status"),
        "last_touched_at": begin_fields.get("last_touched_at"),
        "before_work_line": before_work_line,
        "repo_status": repo_status,
        "begin_output": begin_output.strip().splitlines(),
        "startup_mode": "fresh-chat-fast-path",
        "fast_path_steps": fast_path_steps_for_role(role),
        "deferred_reads": deferred_reads_for_role(role),
        "latest_completed_ci": latest_completed_ci,
        "next_actions": next_actions,
        "latest_same_role_handoff": same_role_handoff,
        "latest_same_role_handoff_persisted": persisted_handoff_review,
        "claimed_agent_label": (
            f"{claim_payload.get('display_id')} "
            f"({agent_uid}/{claimed_model}"
            f"{'/' + current_effort if current_effort else ''})"
        ),
    }
    return result


def print_concise_text_result(result: dict[str, Any]) -> None:
    claimed_agent_label = result.get("claimed_agent_label")
    if claimed_agent_label:
        print(f"claimed_agent: {claimed_agent_label}")
    for key in ["role", "scope", "startup_mode"]:
        value = result.get(key)
        if value is None:
            continue
        print(f"{key}: {value}")

    before_work_line = result.get("before_work_line")
    if before_work_line:
        print(before_work_line)

    print("repo_status:")
    for line in result.get("repo_status", []):
        print(f"  {line}")

    latest_completed_ci = result.get("latest_completed_ci")
    if isinstance(latest_completed_ci, dict):
        print("latest_completed_ci:")
        if latest_completed_ci.get("status") == "completed":
            for key in ["workflowName", "displayTitle", "conclusion", "headSha", "updatedAt", "databaseId"]:
                value = latest_completed_ci.get(key)
                if value is not None:
                    print(f"  {key}: {value}")
        else:
            print(f"  status: {latest_completed_ci.get('status')}")
            message = latest_completed_ci.get("message")
            if message is not None:
                print(f"  message: {message}")

    closeout_command = result.get("closeout_command")
    if closeout_command:
        print(f"closeout_command: {closeout_command}")

    next_actions = result.get("next_actions", [])
    if next_actions:
        print("next_actions:")
        for action in next_actions:
            print(f"  - {action}")

    deferred_reads = result.get("deferred_reads", [])
    if deferred_reads:
        print("deferred_reads:")
        for item in deferred_reads:
            print(f"  - {item}")


def print_text_result(result: dict[str, Any], *, concise: bool = False) -> None:
    if concise:
        print_concise_text_result(result)
        return

    ordered_keys = [
        "agent_uid",
        "display_id",
        "role",
        "scope",
        "assigned_by",
        "assigned_at",
        "mailbox",
        "mailbox_link",
        "confirmed_at",
        "bootstrap_output",
        "bootstrap_created",
        "workcycle_output",
        "batch_num",
        "closeout_command",
        "previous_status",
        "current_status",
        "last_touched_at",
        "startup_mode",
    ]
    for key in ordered_keys:
        value = result.get(key)
        if value is None:
            continue
        print(f"{key}: {value}")

    before_work_line = result.get("before_work_line")
    if before_work_line:
        print(before_work_line)

    print("repo_status:")
    for line in result.get("repo_status", []):
        print(f"  {line}")

    latest_completed_ci = result.get("latest_completed_ci")
    if isinstance(latest_completed_ci, dict):
        print("latest_completed_ci:")
        for key in ["status", "workflowName", "displayTitle", "conclusion", "headSha", "updatedAt", "databaseId", "message"]:
            value = latest_completed_ci.get(key)
            if value is None:
                continue
            print(f"  {key}: {value}")

    fast_path_steps = result.get("fast_path_steps", [])
    if fast_path_steps:
        print("fast_path_steps:")
        for index, step in enumerate(fast_path_steps, start=1):
            print(f"  {index}. {step}")

    next_actions = result.get("next_actions", [])
    if next_actions:
        print("next_actions:")
        for action in next_actions:
            print(f"  - {action}")

    deferred_reads = result.get("deferred_reads", [])
    if deferred_reads:
        print("deferred_reads:")
        for item in deferred_reads:
            print(f"  - {item}")


def main() -> int:
    args = parse_args()
    result = build_result(args)
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print_text_result(result, concise=args.concise)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except BootstrapError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
