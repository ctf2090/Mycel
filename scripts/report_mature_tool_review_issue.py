#!/usr/bin/env python3

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    from scripts.gh_cli_env import preferred_gh_env
except ImportError:  # pragma: no cover
    from gh_cli_env import preferred_gh_env


ROOT_DIR = Path(__file__).resolve().parent.parent
DEFAULT_TITLE = "Mature Tool Review"
LEGACY_TITLES = ("[Review] Mature tool check",)
DEFAULT_LABELS = ("periodic-review",)
HEAD_MARKER = "mature-tool-review-head"
THRESHOLD_MARKER = "mature-tool-review-threshold"


@dataclass(frozen=True)
class IssueRecord:
    number: int
    title: str
    state: str
    body: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/report_mature_tool_review_issue.py",
        description=(
            "Close the previous dedicated mature-tool review issue and create a new one "
            "whenever the commit threshold is reached."
        ),
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=400,
        help="minimum commits since the last reported HEAD before refreshing the review issue",
    )
    parser.add_argument(
        "--title",
        default=DEFAULT_TITLE,
        help=f"issue title to create when refreshing (default: {DEFAULT_TITLE!r})",
    )
    parser.add_argument(
        "--label",
        action="append",
        dest="labels",
        help="label to apply when creating the issue; may be passed more than once",
    )
    parser.add_argument(
        "--repo",
        help="target repository in [HOST/]OWNER/REPO format; defaults to the current repo",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the generated issue body and decision without creating or editing GitHub issues",
    )
    return parser.parse_args()


def run_cmd(args: list[str] | tuple[str, ...], *, input_text: str | None = None) -> str:
    proc = subprocess.run(
        list(args),
        cwd=ROOT_DIR,
        env=preferred_gh_env(),
        text=True,
        input=input_text,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "command failed"
        raise SystemExit(detail)
    return proc.stdout


def git_output(*args: str) -> str:
    return run_cmd(("git", *args)).strip()


def current_head() -> str:
    return git_output("rev-parse", "HEAD")


def short_head(rev: str) -> str:
    try:
        return git_output("rev-parse", "--short", rev)
    except SystemExit:
        return rev[:7]


def revision_exists(rev: str) -> bool:
    if not rev:
        return False
    proc = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"{rev}^{{commit}}"],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    return proc.returncode == 0


def commits_since(base_rev: str, head_rev: str) -> int:
    if not base_rev or not revision_exists(base_rev):
        return 10**9
    return int(git_output("rev-list", "--count", f"{base_rev}..{head_rev}"))


def list_matching_issues(args: argparse.Namespace) -> list[IssueRecord]:
    cmd = [
        "gh",
        "issue",
        "list",
        "--state",
        "all",
        "--limit",
        "100",
        "--json",
        "number,title,state,body",
    ]
    if args.repo:
        cmd.extend(["--repo", args.repo])
    raw = run_cmd(cmd)
    try:
        import json

        payload = json.loads(raw)
    except Exception as exc:  # pragma: no cover
        raise SystemExit(f"failed to parse gh issue list output: {exc}") from exc
    accepted_titles = {args.title, *LEGACY_TITLES}
    matches: list[IssueRecord] = []
    for entry in payload:
        if entry.get("title") not in accepted_titles:
            continue
        matches.append(
            IssueRecord(
                number=int(entry["number"]),
                title=str(entry["title"]),
                state=str(entry.get("state") or ""),
                body=str(entry.get("body") or ""),
            )
        )
    return matches


def extract_marker(body: str, key: str) -> str | None:
    import re

    match = re.search(rf"<!--\s*{re.escape(key)}:\s*([^\s>]+)\s*-->", body)
    if not match:
        return None
    return match.group(1)


def latest_issue(issues: list[IssueRecord]) -> IssueRecord | None:
    if not issues:
        return None
    return max(issues, key=lambda issue: issue.number)


def issue_needs_refresh(issue: IssueRecord | None, *, head_rev: str, threshold: int) -> tuple[bool, int]:
    if issue is None:
        return True, threshold
    marker_rev = extract_marker(issue.body, HEAD_MARKER)
    if marker_rev is None:
        return True, threshold
    commit_distance = commits_since(marker_rev, head_rev)
    return commit_distance >= threshold, commit_distance


def build_issue_body(*, head_rev: str, threshold: int) -> str:
    short = short_head(head_rev)
    return "\n".join(
        [
            f"# Mature Tool Review (`{short}`)",
            "",
            "This issue is refreshed whenever the mature-tool review checkpoint reaches the configured landed-commit threshold.",
            "",
            "## Snapshot",
            f"- Reported `HEAD`: `{head_rev}`",
            f"- Refresh threshold: `{threshold}` commits",
            "- Review owner: `doc`",
            "- Issue form: `.github/ISSUE_TEMPLATE/mature_tool_review.yml`",
            "- Runbook: `docs/MATURE-TOOL-REVIEW-FLOW.md`",
            "",
            "## What To Do",
            "1. Review the last checkpoint window and list repeated maintenance pain signals.",
            "2. Identify only mature tool or module candidates with a concrete Mycel fit.",
            "3. Record `solves what now`, `why now`, and the main tradeoff for each viable candidate.",
            "4. Route any concrete follow-up to a narrower `coding` or `delivery` issue.",
            "",
            "## Manual refresh",
            "```bash",
            (
                "python3 scripts/report_mature_tool_review_issue.py "
                f"--threshold {threshold} --title {DEFAULT_TITLE!r}"
            ),
            "```",
            "",
            f"<!-- {HEAD_MARKER}: {head_rev} -->",
            f"<!-- {THRESHOLD_MARKER}: {threshold} -->",
        ]
    )


def repo_labels(args: argparse.Namespace) -> set[str]:
    cmd = ["gh", "label", "list", "--limit", "200", "--json", "name"]
    if args.repo:
        cmd.extend(["--repo", args.repo])
    raw = run_cmd(cmd)
    try:
        import json

        payload = json.loads(raw)
    except Exception as exc:  # pragma: no cover
        raise SystemExit(f"failed to parse gh label list output: {exc}") from exc
    labels: set[str] = set()
    for entry in payload:
        name = entry.get("name")
        if isinstance(name, str) and name:
            labels.add(name)
    return labels


def create_issue(args: argparse.Namespace, body: str) -> None:
    cmd = ["gh", "issue", "create", "--title", args.title, "--body-file", "-"]
    if args.repo:
        cmd.extend(["--repo", args.repo])
    available_labels = repo_labels(args)
    requested_labels = args.labels or list(DEFAULT_LABELS)
    for label in requested_labels:
        if label not in available_labels:
            print(f"warning: skipping missing label {label!r}", file=sys.stderr)
            continue
        cmd.extend(["--label", label])
    result = run_cmd(cmd, input_text=body)
    print(result, end="")


def close_issue(args: argparse.Namespace, issue_number: int) -> None:
    cmd = ["gh", "issue", "close", str(issue_number)]
    if args.repo:
        cmd.extend(["--repo", args.repo])
    result = run_cmd(cmd)
    print(result, end="")


def close_matching_open_issues(args: argparse.Namespace, issues: list[IssueRecord]) -> list[int]:
    closed: list[int] = []
    for issue in sorted(issues, key=lambda current: current.number, reverse=True):
        if issue.state.lower() != "open":
            continue
        close_issue(args, issue.number)
        closed.append(issue.number)
    return closed


def main() -> int:
    args = parse_args()
    head_rev = current_head()
    matches = list_matching_issues(args)
    issue = latest_issue(matches)
    refresh, commit_distance = issue_needs_refresh(issue, head_rev=head_rev, threshold=args.threshold)

    if not refresh:
        print(
            f"No mature-tool review refresh needed: {commit_distance} commits since the last "
            f"reported head, threshold {args.threshold}."
        )
        return 0

    body = build_issue_body(head_rev=head_rev, threshold=args.threshold)

    if args.dry_run:
        open_issues = [current.number for current in matches if current.state.lower() == "open"]
        if open_issues:
            print(f"Would close issues {open_issues} and create a new issue.")
        else:
            print("Would create a new issue.")
        print(body)
        return 0

    close_matching_open_issues(args, matches)
    create_issue(args, body)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
