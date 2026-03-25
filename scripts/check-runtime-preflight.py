#!/usr/bin/env python3
"""Check whether the current shell session can run a verification command."""

from __future__ import annotations

import argparse
import json
import shutil


DEFAULT_COMMANDS = ("cargo", "bash", "rg")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check whether the current shell session can run a verification command."
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit machine-readable JSON instead of human-oriented log lines.",
    )
    parser.add_argument(
        "--require",
        action="append",
        default=[],
        metavar="COMMAND",
        help="Add a command required by the exact verification command you plan to run.",
    )
    return parser.parse_args()


def emit_json(status: str, checks: list[dict[str, str]]) -> None:
    print(json.dumps({"status": status, "checks": checks}, separators=(",", ":")))


def main() -> int:
    args = parse_args()
    ordered_commands = list(dict.fromkeys([*DEFAULT_COMMANDS, *args.require]))
    checks: list[dict[str, str]] = []
    missing_count = 0

    for cmd in ordered_commands:
        resolved = shutil.which(cmd)
        if resolved:
            checks.append({"name": cmd, "status": "found", "detail": resolved})
            if not args.json:
                print(f"found {cmd:<12} {resolved}")
        else:
            checks.append({"name": cmd, "status": "missing", "detail": ""})
            if not args.json:
                print(f"missing {cmd:<10}")
            missing_count += 1

    if missing_count:
        if args.json:
            emit_json("blocked", checks)
        else:
            print(
                f"runtime preflight blocked: missing {missing_count} required command(s)",
                file=sys.stderr,
            )
        return 1

    if args.json:
        emit_json("passed", checks)
    else:
        print("runtime preflight passed")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(main())
