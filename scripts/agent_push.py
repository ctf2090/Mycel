#!/usr/bin/env python3

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

try:
    from scripts.gh_cli_env import preferred_git_https_env
except ImportError:  # pragma: no cover - direct script execution path
    from gh_cli_env import preferred_git_https_env


ROOT_DIR = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_push.py",
        description=(
            "Push an explicit commit ref to a GitHub HTTPS remote using the agent "
            "GH_TOKEN identity instead of the default Codespaces GITHUB_TOKEN identity."
        ),
    )
    parser.add_argument("commit_ref", help="commit to push, for example HEAD or a specific SHA")
    parser.add_argument(
        "--remote",
        default="origin",
        help="git remote name to push to (default: origin)",
    )
    parser.add_argument(
        "--branch",
        default="main",
        help="target branch to update on the remote (default: main)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the git push command without contacting the remote",
    )
    return parser.parse_args()


def validate_env() -> dict[str, str]:
    env = preferred_git_https_env()
    if not env.get("GH_TOKEN", "").strip():
        raise SystemExit("GH_TOKEN is required for agent HTTPS git operations")
    return env


def build_push_command(args: argparse.Namespace) -> list[str]:
    cmd = ["git", "push"]
    if args.dry_run:
        cmd.append("--dry-run")
    cmd.extend([args.remote, f"{args.commit_ref}:{args.branch}"])
    return cmd


def agent_id_from_commit_ref(commit_ref: str) -> str | None:
    proc = subprocess.run(
        ["git", "log", "-1", "--format=%B", commit_ref],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    for raw_line in proc.stdout.splitlines():
        line = raw_line.strip()
        if line.startswith("Agent-Id:"):
            agent_id = line.partition(":")[2].strip()
            if agent_id:
                return agent_id
    return None


def check_guard_for_agent(agent_id: str) -> tuple[int, str | None]:
    try:
        try:
            from scripts.agent_guard import (
                EXIT_BLOCKED as guard_exit_blocked,
                EXIT_STATE_ERROR as guard_exit_state_error,
                AgentGuardError,
                check_agent,
                format_block_message,
            )
        except ImportError:  # pragma: no cover - direct script execution path
            from agent_guard import (
                EXIT_BLOCKED as guard_exit_blocked,
                EXIT_STATE_ERROR as guard_exit_state_error,
                AgentGuardError,
                check_agent,
                format_block_message,
            )

        guard_result = check_agent(agent_id)
        if guard_result["blocked"]:
            return guard_exit_blocked, format_block_message(guard_result)
        return 0, None
    except AgentGuardError as exc:
        return guard_exit_state_error, f"error: {exc}"


def main() -> int:
    args = parse_args()
    agent_id = agent_id_from_commit_ref(args.commit_ref)
    if agent_id:
        guard_code, guard_message = check_guard_for_agent(agent_id)
        if guard_code != 0:
            if guard_message:
                print(guard_message, file=sys.stderr)
            return guard_code

    env = validate_env()
    cmd = build_push_command(args)
    proc = subprocess.run(
        cmd,
        cwd=ROOT_DIR,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "git push failed"
        print(detail, file=sys.stderr)
        return proc.returncode
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    if proc.stdout:
        print(proc.stdout, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
