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


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_SCRIPT = ROOT_DIR / "scripts" / "agent_registry.py"
AGENTS_PATH = ROOT_DIR / "AGENTS.md"


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


def checklist_item_line(item_id: str, state: str) -> str:
    return f"<!-- item-id: {item_id} -->", f"- [{state}]"


def set_checklist_item_state(checklist_path: Path, item_id: str, state: str) -> None:
    if not checklist_path.exists():
        raise WorkCycleError(f"missing checklist file: {checklist_path.relative_to(ROOT_DIR)}")

    lines = checklist_path.read_text(encoding="utf-8").splitlines()
    marker, replacement = checklist_item_line(item_id, state)
    for index, line in enumerate(lines):
        if marker not in line:
            continue
        lines[index] = line.replace("- [ ]", replacement, 1)
        lines[index] = lines[index].replace("- [X]", replacement, 1)
        lines[index] = lines[index].replace("- [-]", replacement, 1)
        lines[index] = lines[index].replace("- [!]", replacement, 1)
        checklist_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return
    raise WorkCycleError(f"checklist item not found: {item_id}")


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
    print("checklists_checked:")
    for path in checklist_paths:
        print(f"  - {path.relative_to(ROOT_DIR)}")
    total_unchecked = sum(len(items) for items in unchecked_by_path.values())
    print(f"unchecked_items: {total_unchecked}")
    for path, items in unchecked_by_path.items():
        if not items:
            continue
        print(f"unchecked_in: {path.relative_to(ROOT_DIR)}")
        for item in items:
            print(f"  - {item}")


def main() -> int:
    args = parse_args()
    registry_command = "touch" if args.stage == "begin" else "finish"
    payload = run_registry(registry_command, args.agent_ref)
    agent_uid = payload.get("agent_uid") or args.agent_ref
    display_id = payload.get("display_id")

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
        set_checklist_item_state(workcycle_path, "workflow.touch-work-cycle", "X")
        if checklist_result.get("batch_num") == 1:
            set_checklist_item_state(workcycle_path, "workflow.mailbox-handoff-each-cycle", "-")
        print(f"workcycle_output: {workcycle_output}")
        if "batch_num" in checklist_result:
            print(f"batch_num: {checklist_result['batch_num']}")
    else:
        latest_batch = latest_agents_workcycle_batch_num(agent_uid)
        if latest_batch is None:
            raise WorkCycleError(f"no workcycle checklist found for {agent_uid}")

        workcycle_path = agents_workcycle_checklist_path(agent_uid, latest_batch)
        set_checklist_item_state(workcycle_path, "workflow.finish-work-cycle", "X")
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
        print(build_message(stage, agent=label, scope=args.scope))

        for path in checklist_paths:
            unchecked_by_path[path] = scan_unchecked_items(path)
        emit_checklist_summary(
            checklist_paths=checklist_paths,
            unchecked_by_path=unchecked_by_path,
            bootstrap_batch=bootstrap_batch,
        )
        return 2 if any(unchecked_by_path.values()) else 0

    emit_registry_summary(payload)

    stage = "before" if args.stage == "begin" else "after"
    label = display_id or args.agent_ref
    print(build_message(stage, agent=label, scope=args.scope))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except WorkCycleError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
