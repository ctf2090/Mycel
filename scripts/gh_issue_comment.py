#!/usr/bin/env python3

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

try:
    from scripts.gh_cli_env import preferred_gh_env
except ImportError:  # pragma: no cover - direct script execution path
    from gh_cli_env import preferred_gh_env


ROOT_DIR = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Safely comment on or close GitHub issues without shell-quoting markdown bodies."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    comment_parser = subparsers.add_parser("comment", help="post a comment to an issue")
    configure_issue_parser(comment_parser, require_body=True, allow_no_comment=False)

    close_parser = subparsers.add_parser(
        "close",
        help="optionally post a comment, then close an issue without inline shell-quoted markdown",
    )
    configure_issue_parser(close_parser, require_body=False, allow_no_comment=True)
    close_parser.add_argument(
        "--reason",
        choices=("completed", "not planned"),
        default="completed",
        help="closing reason passed to gh issue close (default: completed)",
    )

    return parser.parse_args()


def configure_issue_parser(
    parser: argparse.ArgumentParser,
    *,
    require_body: bool,
    allow_no_comment: bool,
) -> None:
    parser.add_argument("issue", help="issue number or full GitHub issue URL")
    parser.add_argument(
        "-R",
        "--repo",
        help="target repository in [HOST/]OWNER/REPO format; defaults to the current repo",
    )
    parser.add_argument(
        "-F",
        "--body-file",
        default="-",
        help=(
            "comment body file path, or '-' to read from stdin "
            + ("(required)." if require_body else "(optional).")
        ),
    )
    parser.add_argument(
        "--body",
        help="comment body text. Prefer --body-file - with a quoted heredoc for multi-line markdown.",
    )
    if allow_no_comment:
        parser.add_argument(
            "--no-comment",
            action="store_true",
            help="close without adding a comment first",
        )
    else:
        parser.set_defaults(no_comment=False)
    parser.set_defaults(require_body=require_body)


def read_body(args: argparse.Namespace) -> str | None:
    if args.no_comment:
        return None

    if args.body is not None:
        return args.body

    if args.body_file == "-":
        data = sys.stdin.read()
        if args.require_body and not data:
            raise SystemExit("comment body is required; pipe text on stdin or use --body/--body-file")
        return data or None

    data = Path(args.body_file).read_text(encoding="utf-8")
    if args.require_body and not data:
        raise SystemExit(f"comment body file is empty: {args.body_file}")
    return data or None


def run_gh(args: list[str], *, body: str | None = None) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT_DIR,
        env=preferred_gh_env(),
        text=True,
        input=body,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "gh command failed"
        raise SystemExit(detail)
    if proc.stdout:
        print(proc.stdout, end="")


def gh_issue_command(args: argparse.Namespace, subcommand: str) -> list[str]:
    base = ["gh", "issue", subcommand, args.issue]
    if args.repo:
        base.extend(["--repo", args.repo])
    return base


def main() -> int:
    args = parse_args()
    body = read_body(args)

    if args.command == "comment":
        if body is None:
            raise SystemExit("comment body is required; use stdin, --body-file, or --body")
        run_gh([*gh_issue_command(args, "comment"), "--body-file", "-"], body=body)
        return 0

    if body is not None:
        run_gh([*gh_issue_command(args, "comment"), "--body-file", "-"], body=body)

    close_args = [*gh_issue_command(args, "close"), "--reason", args.reason]
    run_gh(close_args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
