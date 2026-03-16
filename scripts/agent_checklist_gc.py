#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
AGENTS_DIR = ROOT_DIR / ".agent-local" / "agents"
DEFAULT_KEEP_WORKCYCLE_BATCHES = 20


class AgentChecklistGcError(Exception):
    pass


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def load_registry() -> dict[str, Any]:
    try:
        payload = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise AgentChecklistGcError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise AgentChecklistGcError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise AgentChecklistGcError("invalid registry: top-level JSON value must be an object")
    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise AgentChecklistGcError("invalid registry: agents must be an array")
    return payload


def referenced_agent_uids(registry: dict[str, Any]) -> set[str]:
    uids: set[str] = set()
    for entry in registry["agents"]:
        if not isinstance(entry, dict):
            continue
        agent_uid = entry.get("agent_uid")
        if isinstance(agent_uid, str) and agent_uid.strip():
            uids.add(agent_uid)
    return uids


def workcycle_checklists_for(agent_dir: Path) -> list[tuple[int, Path]]:
    checklists_dir = agent_dir / "checklists"
    if not checklists_dir.exists():
        return []
    results: list[tuple[int, Path]] = []
    prefix = "AGENTS-workcycle-checklist-"
    suffix = ".md"
    for path in sorted(checklists_dir.iterdir()):
        if not path.is_file():
            continue
        name = path.name
        if not (name.startswith(prefix) and name.endswith(suffix)):
            continue
        batch_str = name[len(prefix):-len(suffix)]
        if not batch_str.isdigit():
            continue
        results.append((int(batch_str), path))
    return sorted(results, key=lambda item: item[0])


def scan_agent_checklists(*, keep_workcycle_batches: int = DEFAULT_KEEP_WORKCYCLE_BATCHES) -> dict[str, Any]:
    registry = load_registry()
    referenced = referenced_agent_uids(registry)
    referenced_agents: list[dict[str, Any]] = []
    orphaned_agent_dirs: list[dict[str, Any]] = []
    prune_candidates: list[dict[str, Any]] = []

    if not AGENTS_DIR.exists():
        agent_dirs: list[Path] = []
    else:
        agent_dirs = sorted(path for path in AGENTS_DIR.iterdir() if path.is_dir())

    for agent_dir in agent_dirs:
        agent_uid = agent_dir.name
        workcycles = workcycle_checklists_for(agent_dir)
        stale = []
        if len(workcycles) > keep_workcycle_batches:
            stale = [
                {"batch": batch, "path": relative_to_root(path)}
                for batch, path in workcycles[:-keep_workcycle_batches]
            ]

        record = {
            "agent_uid": agent_uid,
            "workcycle_checklists": len(workcycles),
            "stale_workcycle_checklists": len(stale),
        }
        if agent_uid in referenced:
            referenced_agents.append(record)
        else:
            orphaned_agent_dirs.append(record)

        prune_candidates.extend(
            {
                "agent_uid": agent_uid,
                "batch": candidate["batch"],
                "path": candidate["path"],
            }
            for candidate in stale
        )

    return {
        "status": "ok",
        "agents_dir": relative_to_root(AGENTS_DIR),
        "keep_workcycle_batches": keep_workcycle_batches,
        "referenced_agent_count": len(referenced_agents),
        "orphaned_agent_dir_count": len(orphaned_agent_dirs),
        "prune_candidate_count": len(prune_candidates),
        "referenced_agents": referenced_agents,
        "orphaned_agent_dirs": orphaned_agent_dirs,
        "prune_candidates": prune_candidates,
    }


def prune_agent_checklists(*, dry_run: bool, keep_workcycle_batches: int) -> dict[str, Any]:
    scan = scan_agent_checklists(keep_workcycle_batches=keep_workcycle_batches)
    deleted: list[dict[str, Any]] = []
    for record in scan["prune_candidates"]:
        path = ROOT_DIR / record["path"]
        deleted.append(record)
        if dry_run or not path.exists():
            continue
        path.unlink()
    return {
        "status": "ok",
        "dry_run": dry_run,
        "keep_workcycle_batches": keep_workcycle_batches,
        "deleted_count": len(deleted),
        "deleted": deleted,
    }


def print_scan(data: dict[str, Any]) -> None:
    print(f"agents_dir: {data['agents_dir']}")
    print(f"keep_workcycle_batches: {data['keep_workcycle_batches']}")
    print(f"referenced_agent_dirs: {data['referenced_agent_count']}")
    print(f"orphaned_agent_dirs: {data['orphaned_agent_dir_count']}")
    print(f"prune_candidates: {data['prune_candidate_count']}")
    if data["prune_candidates"]:
        print("prune_candidate_paths:")
        for record in data["prune_candidates"]:
            print(f"  - {record['path']} ({record['agent_uid']} batch {record['batch']})")


def print_prune(data: dict[str, Any]) -> None:
    print(f"dry_run: {data['dry_run']}")
    print(f"keep_workcycle_batches: {data['keep_workcycle_batches']}")
    print(f"deleted_checklists: {data['deleted_count']}")
    for record in data["deleted"]:
        print(f"- {record['path']} ({record['agent_uid']} batch {record['batch']})")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_checklist_gc.py",
        description="Inspect and prune stale agent-local workcycle checklist batches.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    scan = subparsers.add_parser("scan", add_help=False)
    scan.add_argument(
        "--keep-workcycle-batches",
        type=int,
        default=DEFAULT_KEEP_WORKCYCLE_BATCHES,
    )
    scan.add_argument("--json", action="store_true")
    scan.add_argument("-h", "--help", action="help")
    scan.set_defaults(func=cmd_scan)

    prune = subparsers.add_parser("prune", add_help=False)
    prune.add_argument("--dry-run", action="store_true")
    prune.add_argument(
        "--keep-workcycle-batches",
        type=int,
        default=DEFAULT_KEEP_WORKCYCLE_BATCHES,
    )
    prune.add_argument("--json", action="store_true")
    prune.add_argument("-h", "--help", action="help")
    prune.set_defaults(func=cmd_prune)
    return parser


def cmd_scan(args: argparse.Namespace) -> int:
    if args.keep_workcycle_batches < 1:
        raise AgentChecklistGcError("--keep-workcycle-batches must be >= 1")
    result = scan_agent_checklists(keep_workcycle_batches=args.keep_workcycle_batches)
    if args.json:
        print(json.dumps(result))
    else:
        print_scan(result)
    return 0


def cmd_prune(args: argparse.Namespace) -> int:
    if args.keep_workcycle_batches < 1:
        raise AgentChecklistGcError("--keep-workcycle-batches must be >= 1")
    result = prune_agent_checklists(
        dry_run=args.dry_run,
        keep_workcycle_batches=args.keep_workcycle_batches,
    )
    if args.json:
        print(json.dumps(result))
    else:
        print_prune(result)
    return 0


def main() -> int:
    args = build_parser().parse_args()
    return args.func(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AgentChecklistGcError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
