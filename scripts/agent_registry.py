#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
ALLOWED_ROLES = {"coding", "doc"}


class RegistryError(Exception):
    pass


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def load_registry(*, allow_missing: bool = False) -> dict[str, Any]:
    if allow_missing and not REGISTRY_PATH.exists():
        REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
        return {
            "version": 1,
            "updated_at": None,
            "agent_count": 0,
            "agents": [],
        }

    try:
        registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise RegistryError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise RegistryError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(registry, dict):
        raise RegistryError("invalid registry: top-level JSON value must be an object")

    agents = registry.get("agents")
    if not isinstance(agents, list):
        raise RegistryError("invalid registry: agents must be an array")

    agent_count = registry.get("agent_count")
    if agent_count != len(agents):
        raise RegistryError(
            f"invalid registry: agent_count={agent_count!r} does not match agents length {len(agents)}"
        )

    for entry in agents:
        if not isinstance(entry, dict):
            raise RegistryError("invalid registry: agent entry must be an object")

    return registry


def save_registry(registry: dict[str, Any]) -> None:
    REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
    REGISTRY_PATH.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")


def resolve_mailbox_path(mailbox_value: str) -> Path:
    mailbox_path = Path(mailbox_value)
    if not mailbox_path.is_absolute():
        mailbox_path = ROOT_DIR / mailbox_path
    return mailbox_path


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def find_agent_entry(registry: dict[str, Any], agent_id: str) -> dict[str, Any]:
    matches = [entry for entry in registry["agents"] if entry.get("id") == agent_id]
    if not matches:
        raise RegistryError(f"agent entry not found: {agent_id}")
    if len(matches) > 1:
        raise RegistryError(f"invalid registry: duplicate agent id {agent_id}")
    return matches[0]


def next_agent_id(registry: dict[str, Any], role: str) -> str:
    max_suffix = 0
    for entry in registry["agents"]:
        agent_id = entry.get("id")
        if isinstance(agent_id, str) and agent_id.startswith(f"{role}-"):
            suffix = agent_id[len(role) + 1 :]
            if suffix.isdigit():
                max_suffix = max(max_suffix, int(suffix))
    return f"{role}-{max_suffix + 1}"


def choose_auto_role(registry: dict[str, Any]) -> str:
    active_counts = {role: 0 for role in ALLOWED_ROLES}
    for entry in registry["agents"]:
        entry_role = entry.get("role")
        entry_status = entry.get("status")
        if entry_role in ALLOWED_ROLES and entry_status == "active":
            active_counts[entry_role] += 1

    if active_counts["coding"] == 0:
        return "coding"
    if active_counts["doc"] == 0:
        return "doc"
    return "coding"


def require_non_empty_str(entry: dict[str, Any], field: str, agent_id: str) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"agent {agent_id} is missing required field: {field}")
    return value


def print_json(data: dict[str, Any]) -> None:
    print(json.dumps(data))


def print_claim(data: dict[str, Any]) -> None:
    print(f"agent claimed: {data['agent_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"assigned_by: {data['assigned_by']}")
    print(f"assigned_at: {data['assigned_at']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"next: scripts/agent_registry.py start {data['agent_id']}")


def print_start(data: dict[str, Any]) -> None:
    print(f"agent confirmed: {data['agent_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"confirmed_at: {data['confirmed_at']}")


def print_stop(data: dict[str, Any]) -> None:
    print(f"agent stopped: {data['agent_id']}")
    print(f"previous_status: {data['previous_status']}")
    print(f"current_status: {data['current_status']}")
    print(f"updated_at: {data['updated_at']}")


def print_status(data: dict[str, Any]) -> None:
    print(f"registry: {data['registry_path']}")
    print(f"updated_at: {data['updated_at']}")
    print(f"agents: {data['agent_count']}")
    for entry in data["agents"]:
        print(f"id: {entry['id']}")
        print(f"  role: {entry['role']}")
        print(f"  status: {entry['status']}")
        print(f"  scope: {entry['scope']}")
        print(f"  assigned_by: {entry['assigned_by']}")
        print(f"  assigned_at: {entry['assigned_at']}")
        print(f"  confirmed_by_agent: {entry['confirmed_by_agent']}")
        print(f"  confirmed_at: {entry['confirmed_at']}")
        print(f"  mailbox: {entry['mailbox']}")
        print(f"  mailbox_exists: {entry['mailbox_exists']}")
        files = entry.get("files", [])
        if files:
            print("  files:")
            for path in files:
                print(f"    - {path}")
        else:
            print("  files: []")


def print_resume_check(data: dict[str, Any]) -> None:
    print(f"agent_id: {data['agent_id']}")
    print(f"role: {data['role']}")
    print(f"current_status: {data['current_status']}")
    print(f"scope: {data['scope']}")
    print(f"confirmed_by_agent: {data['confirmed_by_agent']}")
    print(f"confirmed_at: {data['confirmed_at']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"safe_to_resume: {data['safe_to_resume']}")
    print(f"reason: {data['reason']}")


def print_recover(data: dict[str, Any]) -> None:
    print(f"stale_agent: {data['stale_agent_id']}")
    print(f"stale_status_before_recovery: {data['stale_status']}")
    print(f"stale_mailbox: {data['stale_mailbox']}")
    print(f"replacement_agent: {data['replacement_agent_id']}")
    print(f"role: {data['replacement_role']}")
    print(f"scope: {data['replacement_scope']}")
    print(f"replacement_mailbox: {data['replacement_mailbox']}")
    print(f"assigned_by: {data['assigned_by']}")
    print(f"updated_at: {data['updated_at']}")
    print(f"takeover_note: {data['takeover_note']}")
    print("next: read the stale mailbox before resuming tracked work")


def cmd_claim(args: argparse.Namespace) -> int:
    if args.role != "auto" and args.role not in ALLOWED_ROLES:
        raise RegistryError(f"unsupported role: {args.role}")
    if not args.assigned_by.strip():
        raise RegistryError("assigned_by must not be empty")
    if not args.scope.strip():
        raise RegistryError("scope must not be empty")

    registry = load_registry(allow_missing=True)
    role = choose_auto_role(registry) if args.role == "auto" else args.role

    new_id = next_agent_id(registry, role)
    mailbox_rel = f".agent-local/{new_id}.md"
    mailbox_path = resolve_mailbox_path(mailbox_rel)
    mailbox_path.parent.mkdir(parents=True, exist_ok=True)
    if not mailbox_path.exists():
        mailbox_path.write_text(f"# Mailbox for {new_id}\n\n", encoding="utf-8")

    now = utc_now()
    registry["agents"].append(
        {
            "id": new_id,
            "role": role,
            "assigned_by": args.assigned_by,
            "assigned_at": now,
            "confirmed_by_agent": False,
            "confirmed_at": None,
            "status": "paused",
            "scope": args.scope,
            "files": [],
            "mailbox": mailbox_rel,
        }
    )
    registry["agent_count"] = len(registry["agents"])
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_id": new_id,
        "role": role,
        "scope": args.scope,
        "assigned_by": args.assigned_by,
        "assigned_at": now,
        "mailbox": mailbox_rel,
    }
    if args.json:
        print_json(result)
    else:
        print_claim(result)
    return 0


def cmd_start(args: argparse.Namespace) -> int:
    registry = load_registry()
    entry = find_agent_entry(registry, args.agent_id)

    for field in ["role", "assigned_by", "assigned_at", "scope", "mailbox"]:
        require_non_empty_str(entry, field, args.agent_id)

    status = require_non_empty_str(entry, "status", args.agent_id)
    if status == "done":
        raise RegistryError(f"agent {args.agent_id} cannot start because status is done")
    if status == "blocked":
        raise RegistryError(f"agent {args.agent_id} cannot start because status is blocked")

    files = entry.get("files")
    if not isinstance(files, list):
        raise RegistryError(f"agent {args.agent_id} is missing required field: files")

    now = utc_now()
    entry["confirmed_by_agent"] = True
    entry["confirmed_at"] = now
    entry["status"] = "active"
    registry["updated_at"] = now

    mailbox_path = resolve_mailbox_path(entry["mailbox"])
    mailbox_path.parent.mkdir(parents=True, exist_ok=True)
    if not mailbox_path.exists():
        mailbox_path.write_text(f"# Mailbox for {args.agent_id}\n\n", encoding="utf-8")

    save_registry(registry)

    result = {
        "status": "ok",
        "agent_id": args.agent_id,
        "role": entry["role"],
        "scope": entry["scope"],
        "mailbox": relative_to_root(mailbox_path),
        "confirmed_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_start(result)
    return 0


def cmd_stop(args: argparse.Namespace) -> int:
    if args.status not in {"paused", "done"}:
        raise RegistryError(f"unsupported stop status: {args.status}")

    registry = load_registry()
    entry = find_agent_entry(registry, args.agent_id)
    current_status = require_non_empty_str(entry, "status", args.agent_id)

    now = utc_now()
    entry["status"] = args.status
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_id": args.agent_id,
        "previous_status": current_status,
        "current_status": args.status,
        "updated_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_stop(result)
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    registry = load_registry()
    selected = registry["agents"]

    if args.agent_id:
        selected = [find_agent_entry(registry, args.agent_id)]

    normalized = []
    for entry in selected:
        mailbox_value = entry.get("mailbox")
        mailbox_exists = False
        mailbox_display = None
        if isinstance(mailbox_value, str) and mailbox_value.strip():
            mailbox_path = resolve_mailbox_path(mailbox_value)
            mailbox_exists = mailbox_path.exists()
            mailbox_display = relative_to_root(mailbox_path)

        normalized.append(
            {
                "id": entry.get("id"),
                "role": entry.get("role"),
                "status": entry.get("status"),
                "scope": entry.get("scope"),
                "assigned_by": entry.get("assigned_by"),
                "assigned_at": entry.get("assigned_at"),
                "confirmed_by_agent": entry.get("confirmed_by_agent", False),
                "confirmed_at": entry.get("confirmed_at"),
                "files": entry.get("files", []),
                "mailbox": mailbox_display,
                "mailbox_exists": mailbox_exists,
            }
        )

    result = {
        "status": "ok",
        "registry_path": str(REGISTRY_PATH),
        "updated_at": registry.get("updated_at"),
        "agent_count": len(normalized) if args.agent_id else len(registry["agents"]),
        "agents": normalized,
    }
    if args.json:
        print_json(result)
    else:
        print_status(result)
    return 0


def cmd_resume_check(args: argparse.Namespace) -> int:
    registry = load_registry()
    entry = find_agent_entry(registry, args.agent_id)

    role = require_non_empty_str(entry, "role", args.agent_id)
    status = require_non_empty_str(entry, "status", args.agent_id)
    scope = require_non_empty_str(entry, "scope", args.agent_id)
    mailbox_value = require_non_empty_str(entry, "mailbox", args.agent_id)
    confirmed_by_agent = entry.get("confirmed_by_agent", False)
    confirmed_at = entry.get("confirmed_at")

    mailbox_path = resolve_mailbox_path(mailbox_value)
    reason = "agent is still active and confirmed"
    safe_to_resume = True
    exit_code = 0

    if status != "active":
        safe_to_resume = False
        reason = f"agent status is {status}; do not resume tracked work under {args.agent_id}"
        exit_code = 2
    elif confirmed_by_agent is not True or not isinstance(confirmed_at, str) or not confirmed_at.strip():
        safe_to_resume = False
        reason = f"agent {args.agent_id} is not fully confirmed; do not resume tracked work"
        exit_code = 2

    result = {
        "status": "ok" if safe_to_resume else "stop",
        "agent_id": args.agent_id,
        "role": role,
        "current_status": status,
        "scope": scope,
        "confirmed_by_agent": bool(confirmed_by_agent),
        "confirmed_at": confirmed_at,
        "mailbox": relative_to_root(mailbox_path),
        "safe_to_resume": safe_to_resume,
        "reason": reason,
    }
    if args.json:
        print_json(result)
    else:
        print_resume_check(result)
    return exit_code


def cmd_recover(args: argparse.Namespace) -> int:
    if not args.assigned_by.strip():
        raise RegistryError("assigned_by must not be empty")

    registry = load_registry()
    stale_entry = find_agent_entry(registry, args.stale_agent_id)

    role = require_non_empty_str(stale_entry, "role", args.stale_agent_id)
    if role not in ALLOWED_ROLES:
        raise RegistryError(f"unsupported role in stale entry: {role}")

    scope = args.scope or require_non_empty_str(stale_entry, "scope", args.stale_agent_id)
    stale_mailbox_value = require_non_empty_str(stale_entry, "mailbox", args.stale_agent_id)
    stale_status = require_non_empty_str(stale_entry, "status", args.stale_agent_id)
    if stale_status == "done":
        raise RegistryError(f"agent {args.stale_agent_id} cannot be recovered because status is done")

    stale_mailbox_path = resolve_mailbox_path(stale_mailbox_value)
    new_agent_id = next_agent_id(registry, role)
    new_mailbox_rel = f".agent-local/{new_agent_id}.md"
    new_mailbox_path = resolve_mailbox_path(new_mailbox_rel)
    new_mailbox_path.parent.mkdir(parents=True, exist_ok=True)
    if not new_mailbox_path.exists():
        new_mailbox_path.write_text(f"# Mailbox for {new_agent_id}\n\n", encoding="utf-8")

    now = utc_now()
    stale_entry["status"] = "paused"
    registry["agents"].append(
        {
            "id": new_agent_id,
            "role": role,
            "assigned_by": args.assigned_by,
            "assigned_at": now,
            "confirmed_by_agent": True,
            "confirmed_at": now,
            "status": "active",
            "scope": scope,
            "files": [],
            "mailbox": new_mailbox_rel,
        }
    )
    registry["agent_count"] = len(registry["agents"])
    registry["updated_at"] = now
    save_registry(registry)

    takeover_note = f"taking over from {args.stale_agent_id} after interrupted chat"
    existing_mailbox = new_mailbox_path.read_text(encoding="utf-8")
    if takeover_note not in existing_mailbox:
        new_mailbox_path.write_text(existing_mailbox + f"- {takeover_note}\n", encoding="utf-8")

    result = {
        "status": "ok",
        "stale_agent_id": args.stale_agent_id,
        "stale_status": stale_status,
        "stale_mailbox": relative_to_root(stale_mailbox_path),
        "replacement_agent_id": new_agent_id,
        "replacement_role": role,
        "replacement_scope": scope,
        "replacement_mailbox": new_mailbox_rel,
        "assigned_by": args.assigned_by,
        "updated_at": now,
        "takeover_note": takeover_note,
    }
    if args.json:
        print_json(result)
    else:
        print_recover(result)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_registry.py",
        description="Manage the local agent registry.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    claim = subparsers.add_parser("claim", add_help=False)
    claim.add_argument("role")
    claim.add_argument("--scope", default="pending scope")
    claim.add_argument("--assigned-by", default="user")
    claim.add_argument("--json", action="store_true")
    claim.add_argument("-h", "--help", action="help")
    claim.set_defaults(func=cmd_claim)

    start = subparsers.add_parser("start", add_help=False)
    start.add_argument("agent_id")
    start.add_argument("--json", action="store_true")
    start.add_argument("-h", "--help", action="help")
    start.set_defaults(func=cmd_start)

    stop = subparsers.add_parser("stop", add_help=False)
    stop.add_argument("agent_id")
    stop.add_argument("--status", default="paused")
    stop.add_argument("--json", action="store_true")
    stop.add_argument("-h", "--help", action="help")
    stop.set_defaults(func=cmd_stop)

    status = subparsers.add_parser("status", add_help=False)
    status.add_argument("agent_id", nargs="?")
    status.add_argument("--json", action="store_true")
    status.add_argument("-h", "--help", action="help")
    status.set_defaults(func=cmd_status)

    resume_check = subparsers.add_parser("resume-check", add_help=False)
    resume_check.add_argument("agent_id")
    resume_check.add_argument("--json", action="store_true")
    resume_check.add_argument("-h", "--help", action="help")
    resume_check.set_defaults(func=cmd_resume_check)

    recover = subparsers.add_parser("recover", add_help=False)
    recover.add_argument("stale_agent_id")
    recover.add_argument("--scope", default="")
    recover.add_argument("--assigned-by", default="user")
    recover.add_argument("--json", action="store_true")
    recover.add_argument("-h", "--help", action="help")
    recover.set_defaults(func=cmd_recover)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except RegistryError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
