#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"
WORK_CYCLE_SCRIPT = ROOT_DIR / "scripts" / "agent_work_cycle.py"
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
        "planning-sync mailbox scans and scripts/check-plan-refresh.sh until the doc work item actually starts",
    ],
}


class BootstrapError(Exception):
    pass


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


def run_registry_json(*args: str) -> dict[str, Any]:
    return run_json_command([sys.executable, str(REGISTRY_SCRIPT), *args, "--json"])


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


def fast_path_steps_for_role(role: str) -> list[str]:
    steps = list(FAST_PATH_STEPS)
    if role in {"coding", "delivery"}:
        steps.append(
            "check the latest completed CI result for the previous push before implementation or delivery work"
        )
    return steps


def deferred_reads_for_role(role: str) -> list[str]:
    return DEFERRED_READS_COMMON + DEFERRED_READS_BY_ROLE.get(role, [])


def next_actions_for_role(role: str) -> list[str]:
    if role == "coding":
        return [
            "check the latest completed CI result for the previous push before implementation work",
            "defer mailbox scans unless the scope overlaps existing coding work, recovery, or takeover",
        ]
    if role == "delivery":
        return [
            "check the latest completed CI result for the previous push before triaging delivery work",
            "defer broad roadmap/checklist reading unless the active delivery scope needs doc follow-up",
        ]
    if role == "doc":
        return [
            "wait for the concrete doc task before running planning-sync mailbox scans or scripts/check-plan-refresh.sh",
        ]
    return []


def build_result(args: argparse.Namespace) -> dict[str, Any]:
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
    role = claim_payload.get("role")
    if not isinstance(role, str) or not role.strip():
        raise BootstrapError("claim did not return role")

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
        "previous_status": begin_fields.get("previous_status"),
        "current_status": begin_fields.get("current_status"),
        "last_touched_at": begin_fields.get("last_touched_at"),
        "before_work_line": before_work_line,
        "repo_status": repo_status,
        "begin_output": begin_output.strip().splitlines(),
        "startup_mode": "fresh-chat-fast-path",
        "fast_path_steps": fast_path_steps_for_role(role),
        "deferred_reads": deferred_reads_for_role(role),
        "next_actions": next_actions_for_role(role),
        "claimed_agent_label": f"{claim_payload.get('display_id')} ({agent_uid}/{args.model_id})",
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
