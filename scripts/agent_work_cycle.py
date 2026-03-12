#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from agent_timestamp import build_message


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"


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


def main() -> int:
    args = parse_args()
    registry_command = "touch" if args.stage == "begin" else "finish"
    payload = run_registry(registry_command, args.agent_ref)
    emit_registry_summary(payload)

    stage = "before" if args.stage == "begin" else "after"
    label = payload.get("display_id") or args.agent_ref
    print(build_message(stage, agent=label, scope=args.scope))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except WorkCycleError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
