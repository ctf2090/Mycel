#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

try:
    from scripts.agent_registry import (
        RegistryError,
        current_display_id,
        load_registry,
        resolve_agent_entry,
    )
except ImportError:  # pragma: no cover - direct script execution path
    from agent_registry import (
        RegistryError,
        current_display_id,
        load_registry,
        resolve_agent_entry,
    )


ROOT_DIR = Path(__file__).resolve().parent.parent
BLOCK_STATE_PATH = ROOT_DIR / ".agent-local" / "runtime" / "agent-blocks.json"
STATE_VERSION = 1
EXIT_ALLOWED = 0
EXIT_BLOCKED = 10
EXIT_STATE_ERROR = 12


class AgentGuardError(Exception):
    pass


def empty_state() -> dict[str, Any]:
    return {
        "version": STATE_VERSION,
        "blocks": {},
    }


def load_block_state(*, allow_missing: bool = True) -> dict[str, Any]:
    if allow_missing and not BLOCK_STATE_PATH.exists():
        return empty_state()
    try:
        payload = json.loads(BLOCK_STATE_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        if allow_missing:
            return empty_state()
        raise AgentGuardError(f"missing guard state: {BLOCK_STATE_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise AgentGuardError(f"invalid guard state JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise AgentGuardError("invalid guard state: top-level JSON value must be an object")
    version = payload.get("version")
    if version != STATE_VERSION:
        raise AgentGuardError(
            f"invalid guard state: version={version!r} does not match expected {STATE_VERSION}"
        )
    blocks = payload.get("blocks")
    if not isinstance(blocks, dict):
        raise AgentGuardError("invalid guard state: blocks must be an object")
    return payload


def save_block_state(payload: dict[str, Any]) -> None:
    BLOCK_STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w",
        encoding="utf-8",
        dir=str(BLOCK_STATE_PATH.parent),
        delete=False,
    ) as handle:
        json.dump(payload, handle, indent=2)
        handle.write("\n")
        temp_path = Path(handle.name)
    temp_path.replace(BLOCK_STATE_PATH)


def resolve_agent(agent_ref: str) -> dict[str, str]:
    try:
        registry = load_registry()
        entry = resolve_agent_entry(registry, agent_ref)
    except RegistryError as exc:
        if "missing registry file:" in str(exc):
            fallback = agent_ref.strip()
            if not fallback:
                raise AgentGuardError("agent_ref must be non-empty") from exc
            return {
                "agent_uid": fallback,
                "display_id": fallback,
            }
        raise AgentGuardError(str(exc)) from exc

    agent_uid = entry.get("agent_uid")
    if not isinstance(agent_uid, str) or not agent_uid.strip():
        raise AgentGuardError(f"agent {agent_ref} is missing required field: agent_uid")

    display_id = current_display_id(entry) or agent_uid
    return {
        "agent_uid": agent_uid,
        "display_id": display_id,
    }


def check_agent(agent_ref: str) -> dict[str, Any]:
    agent = resolve_agent(agent_ref)
    state = load_block_state()
    block_entry = state["blocks"].get(agent["agent_uid"])
    blocked = isinstance(block_entry, dict) and block_entry.get("blocked") is True
    return {
        **agent,
        "blocked": blocked,
        "block": block_entry if blocked else None,
    }


def block_agent(
    agent_ref: str,
    *,
    reason: str,
    detected_at: str,
    source: str,
    scope: str | None = None,
    handoff_path: str | None = None,
    rollout_path: str | None = None,
    clear_requires: str = "new_chat_bootstrap",
) -> dict[str, Any]:
    if not reason.strip():
        raise AgentGuardError("reason must be non-empty")
    if not detected_at.strip():
        raise AgentGuardError("detected_at must be non-empty")
    if not source.strip():
        raise AgentGuardError("source must be non-empty")

    agent = resolve_agent(agent_ref)
    state = load_block_state()
    entry: dict[str, Any] = {
        "blocked": True,
        "reason": reason.strip(),
        "detected_at": detected_at.strip(),
        "source": source.strip(),
        "clear_requires": clear_requires.strip(),
    }
    if scope and scope.strip():
        entry["scope"] = scope.strip()
    if handoff_path and handoff_path.strip():
        entry["handoff_path"] = handoff_path.strip()
    if rollout_path and rollout_path.strip():
        entry["rollout_path"] = rollout_path.strip()
    state["blocks"][agent["agent_uid"]] = entry
    save_block_state(state)
    return {
        **agent,
        "blocked": True,
        "block": entry,
    }


def status_payload() -> dict[str, Any]:
    state = load_block_state()
    return {
        "version": state["version"],
        "blocked_agents": state["blocks"],
    }


def format_block_message(result: dict[str, Any]) -> str:
    block = result.get("block") or {}
    lines = [
        "agent execution blocked",
        f"agent: {result['display_id']} ({result['agent_uid']})",
        f"reason: {block.get('reason', 'unknown')}",
        f"detected_at: {block.get('detected_at', 'unknown')}",
    ]
    handoff_path = block.get("handoff_path")
    if isinstance(handoff_path, str) and handoff_path.strip():
        lines.append(f"handoff: {handoff_path}")
    lines.append("next_step: open a new chat and continue from the handoff")
    return "\n".join(lines)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_guard.py",
        description="Persist and query repo-local agent execution guard state.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser("check", help="check whether an agent is blocked")
    check.add_argument("agent_ref", help="agent_uid or current display_id")
    check.add_argument("--json", action="store_true", help="emit JSON instead of plain text")

    block = subparsers.add_parser("block", help="write a blocked execution state for an agent")
    block.add_argument("agent_ref", help="agent_uid or current display_id")
    block.add_argument("--reason", required=True, help="block reason identifier")
    block.add_argument("--detected-at", required=True, dest="detected_at", help="detection timestamp")
    block.add_argument("--source", required=True, help="writer identifier such as agent_work_cycle.begin")
    block.add_argument("--scope", help="scope label active when the block was recorded")
    block.add_argument("--handoff-path", dest="handoff_path", help="mailbox path holding the handoff")
    block.add_argument("--rollout-path", dest="rollout_path", help="rollout JSONL path that triggered the block")
    block.add_argument(
        "--clear-requires",
        default="new_chat_bootstrap",
        dest="clear_requires",
        help="what kind of flow is allowed to clear the block later",
    )
    block.add_argument("--json", action="store_true", help="emit JSON instead of plain text")

    status = subparsers.add_parser("status", help="list blocked agents")
    status.add_argument("--json", action="store_true", help="emit JSON instead of plain text")

    return parser


def main() -> int:
    args = build_parser().parse_args()
    try:
        if args.command == "check":
            result = check_agent(args.agent_ref)
            if args.json:
                print(json.dumps(result, indent=2))
            elif result["blocked"]:
                print(format_block_message(result))
            else:
                print(f"agent allowed: {result['display_id']} ({result['agent_uid']})")
            return EXIT_BLOCKED if result["blocked"] else EXIT_ALLOWED

        if args.command == "block":
            result = block_agent(
                args.agent_ref,
                reason=args.reason,
                detected_at=args.detected_at,
                source=args.source,
                scope=args.scope,
                handoff_path=args.handoff_path,
                rollout_path=args.rollout_path,
                clear_requires=args.clear_requires,
            )
            if args.json:
                print(json.dumps(result, indent=2))
            else:
                print(format_block_message(result))
            return EXIT_BLOCKED

        if args.command == "status":
            result = status_payload()
            if args.json:
                print(json.dumps(result, indent=2))
            else:
                blocked_agents = result["blocked_agents"]
                if not blocked_agents:
                    print("blocked agents: none")
                else:
                    print(f"blocked agents: {len(blocked_agents)}")
                    for agent_uid, entry in sorted(blocked_agents.items()):
                        reason = entry.get("reason", "unknown")
                        print(f"- {agent_uid}: {reason}")
            return EXIT_ALLOWED
    except AgentGuardError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_STATE_ERROR

    print(f"error: unsupported command {args.command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
