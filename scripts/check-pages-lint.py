#!/usr/bin/env python3
"""Run local Pages lint only when staged changes touch pages/."""

from __future__ import annotations

import subprocess
import sys


def main() -> int:
    diff_proc = subprocess.run(
        ["git", "diff", "--cached", "--quiet", "--", "pages/"],
        check=False,
    )
    if diff_proc.returncode == 0:
        return 0
    if diff_proc.returncode not in (0, 1):
        return diff_proc.returncode

    print("pages/ changes detected; running local Pages lint...")
    lint_proc = subprocess.run(["npm", "run", "lint:pages"], check=False)
    return lint_proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
