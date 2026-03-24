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


def main() -> int:
    args = parse_args()
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
