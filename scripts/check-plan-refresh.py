#!/usr/bin/env python3
"""Check whether planning-surface refresh is due."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


TRACKED_FILES = (
    "ROADMAP.md",
    "ROADMAP.zh-TW.md",
    "IMPLEMENTATION-CHECKLIST.en.md",
    "IMPLEMENTATION-CHECKLIST.zh-TW.md",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check whether planning-surface refresh is due."
    )
    parser.add_argument("--doc-threshold", type=int, default=10)
    parser.add_argument("--issue-threshold", type=int, default=10)
    parser.add_argument("--web-threshold", type=int, default=20)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    for field_name in ("doc_threshold", "issue_threshold", "web_threshold"):
        value = getattr(args, field_name)
        if value < 0:
            parser.error(f"{field_name.upper()} must be a non-negative integer")

    return args


def git_output(repo_root: Path, *args: str) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(repo_root), *args],
            text=True,
        ).strip()
    except FileNotFoundError as exc:
        raise SystemExit("git is required") from exc
    except subprocess.CalledProcessError as exc:
        stderr = (exc.stderr or "").strip()
        stdout = (exc.stdout or "").strip()
        detail = stderr or stdout or "git command failed"
        raise SystemExit(detail) from exc


def fail(message: str, *, json_mode: bool, repo_root: Path) -> None:
    if json_mode:
        payload = {
            "status": "failed",
            "repo_root": str(repo_root),
            "checks": [],
            "surfaces": [],
            "error": message,
        }
        print(json.dumps(payload, separators=(",", ":")))
    else:
        print(message, file=sys.stderr)
    raise SystemExit(1)


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]

    try:
        inside_worktree = git_output(repo_root, "rev-parse", "--is-inside-work-tree")
    except SystemExit as exc:
        fail(str(exc), json_mode=args.json, repo_root=repo_root)
    if inside_worktree != "true":
        fail(f"not inside a git worktree: {repo_root}", json_mode=args.json, repo_root=repo_root)

    results: list[dict[str, object]] = []
    max_count = 0

    for file in TRACKED_FILES:
        path = repo_root / file
        if not path.is_file():
            fail(f"tracked file not found: {file}", json_mode=args.json, repo_root=repo_root)

        try:
            last_commit = git_output(repo_root, "log", "-n", "1", "--format=%H", "--", file)
            if not last_commit:
                fail(
                    f"no git history found for tracked file: {file}",
                    json_mode=args.json,
                    repo_root=repo_root,
                )
            commit_count = int(git_output(repo_root, "rev-list", "--count", f"{last_commit}..HEAD"))
            short_commit = git_output(repo_root, "rev-parse", "--short", last_commit)
        except SystemExit as exc:
            fail(str(exc), json_mode=args.json, repo_root=repo_root)

        max_count = max(max_count, commit_count)
        results.append(
            {
                "file": file,
                "status": "ok",
                "commit_count": commit_count,
                "last_commit": short_commit,
            }
        )
        if not args.json:
            print(f"ok\t{commit_count} commits since {short_commit}\t{file}")

    surfaces = [
        ("doc", args.doc_threshold),
        ("issue", args.issue_threshold),
        ("web", args.web_threshold),
    ]
    surface_results: list[dict[str, object]] = []
    overall_due = False
    smallest_remaining: int | None = None

    for name, threshold in surfaces:
        if max_count >= threshold:
            status = "due"
            remaining = 0
            overall_due = True
        else:
            status = "ok"
            remaining = threshold - max_count
            if smallest_remaining is None or remaining < smallest_remaining:
                smallest_remaining = remaining

        surface_results.append(
            {
                "name": name,
                "threshold": threshold,
                "status": status,
                "remaining_commits": remaining,
            }
        )
        if not args.json:
            if status == "due":
                print(f"due\t{name} refresh\tthreshold {threshold}\t0 commits remain")
            else:
                print(
                    f"ok\t{name} refresh\tthreshold {threshold}\t{remaining} commits remain"
                )

    remaining = 0 if overall_due else (smallest_remaining or 0)
    due_surfaces = [
        surface["name"] for surface in surface_results if surface["status"] == "due"
    ]

    if args.json:
        payload = {
            "status": "due" if overall_due else "ok",
            "repo_root": str(repo_root),
            "highest_commit_distance": max_count,
            "remaining_commits": remaining,
            "checks": results,
            "surfaces": surface_results,
            "due_surfaces": due_surfaces,
        }
        print(json.dumps(payload, separators=(",", ":")))
    elif overall_due:
        print(f"plan refresh due: {','.join(due_surfaces)}")
        print(f"highest commit distance across tracked files: {max_count}")
    else:
        print(
            f"plan refresh not due: {remaining} commits remain before the next threshold"
        )

    return 1 if overall_due else 0


if __name__ == "__main__":
    raise SystemExit(main())
