#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections import OrderedDict
from dataclasses import dataclass
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parent.parent
DEFAULT_TITLE = "Code Quality Hotspots"
LEGACY_TITLES = ("[Report] Code Quality Hotspots",)
DEFAULT_LABELS = ("code-quality-hotspot",)
HEAD_MARKER = "hotspot-report-head"
THRESHOLD_MARKER = "hotspot-report-threshold"
SCANNER_CMD = ("python3", "scripts/check_code_quality_hotspots.py", "apps", "crates", "scripts")


@dataclass(frozen=True)
class IssueRecord:
    number: int
    title: str
    state: str
    body: str


@dataclass(frozen=True)
class RankedCandidate:
    rank: int
    score: int
    path: str
    file_lines: int
    file_over_threshold: bool
    function_count: int
    function_note: str
    literal_count: int
    literal_note: str
    numeric_count: int
    numeric_note: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/report_code_quality_hotspots_issue.py",
        description=(
            "Close the previous dedicated GitHub issue and create a new code-quality "
            "hotspot report whenever the commit threshold is reached."
        ),
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=20,
        help="minimum commits since the last reported HEAD before refreshing the issue",
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
        "--top",
        type=int,
        default=10,
        help="number of ranked split candidates to include in the issue body",
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
    if not base_rev:
        return 10**9
    if not revision_exists(base_rev):
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
    match = re.search(rf"<!--\s*{re.escape(key)}:\s*([^\s>]+)\s*-->", body)
    if not match:
        return None
    return match.group(1)


def latest_issue(issues: list[IssueRecord]) -> IssueRecord | None:
    if not issues:
        return None
    return max(issues, key=lambda issue: issue.number)


def scanner_output() -> str:
    return run_cmd(SCANNER_CMD)


def ranked_candidates(scan_text: str, top_n: int) -> list[str]:
    marker = "Ranked split candidates:"
    if marker not in scan_text:
        return []
    lines = scan_text.splitlines()
    start = lines.index(marker) + 1
    ranked = [line for line in lines[start:] if re.match(r"^\d+\.\s+score=", line)]
    return ranked[:top_n]


def parse_ranked_candidate(line: str) -> RankedCandidate | None:
    match = re.match(
        r"^(?P<rank>\d+)\.\s+score=(?P<score>\d+)\s+(?P<path>\S+)\s+\|\s+"
        r"file\s+(?P<file_lines>\d+)\s+lines(?P<file_under>\s+\(under file threshold\))?;\s+"
        r"long functions=(?P<function_count>\d+)\s+\[(?P<function_note>.*)\];\s+"
        r"repeated literals=(?P<literal_count>\d+)\s+\[(?P<literal_note>.*)\];\s+"
        r"numeric literals=(?P<numeric_count>\d+)\s+\[(?P<numeric_note>.*)\]$",
        line,
    )
    if match is None:
        return None
    return RankedCandidate(
        rank=int(match.group("rank")),
        score=int(match.group("score")),
        path=match.group("path"),
        file_lines=int(match.group("file_lines")),
        file_over_threshold=match.group("file_under") is None,
        function_count=int(match.group("function_count")),
        function_note=match.group("function_note"),
        literal_count=int(match.group("literal_count")),
        literal_note=match.group("literal_note"),
        numeric_count=int(match.group("numeric_count")),
        numeric_note=match.group("numeric_note"),
    )


def categorized_hotspots(scan_text: str, top_n: int) -> OrderedDict[str, list[str]]:
    grouped: OrderedDict[str, list[str]] = OrderedDict(
        (
            ("file-size", []),
            ("function-size", []),
            ("literal-repeat", []),
            ("numeric-literal-repeat", []),
        )
    )
    for line in ranked_candidates(scan_text, top_n):
        candidate = parse_ranked_candidate(line)
        if candidate is None:
            continue
        if candidate.file_over_threshold:
            grouped["file-size"].append(
                f"{len(grouped['file-size']) + 1}. {candidate.path} "
                f"(rank {candidate.rank}, score={candidate.score}, {candidate.file_lines} lines)"
            )
        if candidate.function_count > 0:
            grouped["function-size"].append(
                f"{len(grouped['function-size']) + 1}. {candidate.path} "
                f"(rank {candidate.rank}, score={candidate.score}, "
                f"{candidate.function_count} long functions: {candidate.function_note})"
            )
        if candidate.literal_count > 0:
            grouped["literal-repeat"].append(
                f"{len(grouped['literal-repeat']) + 1}. {candidate.path} "
                f"(rank {candidate.rank}, score={candidate.score}, "
                f"{candidate.literal_count} repeated literals: {candidate.literal_note})"
            )
        if candidate.numeric_count > 0:
            grouped["numeric-literal-repeat"].append(
                f"{len(grouped['numeric-literal-repeat']) + 1}. {candidate.path} "
                f"(rank {candidate.rank}, score={candidate.score}, "
                f"{candidate.numeric_count} numeric literal repeats: {candidate.numeric_note})"
            )
    return grouped


def summary_line(scan_text: str) -> str:
    for line in scan_text.splitlines():
        if line.startswith("Summary: "):
            return line
    return "Summary: scanner output did not include a finding summary."


def build_issue_body(*, head_rev: str, threshold: int, scan_text: str, top_n: int) -> str:
    short = short_head(head_rev)
    grouped = categorized_hotspots(scan_text, top_n)
    grouped_sections: list[str] = []
    for category, entries in grouped.items():
        grouped_sections.extend(
            [
                f"### `{category}`",
                (
                    f"_From the top {top_n} ranked hotspot candidates._"
                    if entries
                    else f"_No top-{top_n} ranked hotspot candidates matched this category._"
                ),
                *(entries or ["1. None in the current top-ranked hotspot set."]),
                "",
            ]
        )
    return "\n".join(
        [
            f"# Code Quality Hotspots (`{short}`)",
            "",
            "This issue is refreshed when the hotspot scan reaches the configured landed-commit threshold.",
            "",
            "## Snapshot",
            f"- Reported `HEAD`: `{head_rev}`",
            f"- Refresh threshold: `{threshold}` commits",
            f"- {summary_line(scan_text).removeprefix('Summary: ')}",
            "",
            "## Hotspots by category",
            *grouped_sections,
            "",
            "## Manual refresh",
            "```bash",
            (
                "python3 scripts/report_code_quality_hotspots_issue.py "
                f"--threshold {threshold} --top {top_n} --title {DEFAULT_TITLE!r}"
            ),
            "```",
            "",
            "## Source command",
            "```bash",
            "python3 scripts/check_code_quality_hotspots.py apps crates scripts",
            "```",
            "",
            f"<!-- {HEAD_MARKER}: {head_rev} -->",
            f"<!-- {THRESHOLD_MARKER}: {threshold} -->",
        ]
    )


def issue_needs_refresh(issue: IssueRecord | None, *, head_rev: str, threshold: int) -> tuple[bool, int]:
    if issue is None:
        return True, threshold
    marker_rev = extract_marker(issue.body, HEAD_MARKER)
    if marker_rev is None:
        return True, threshold
    commit_distance = commits_since(marker_rev, head_rev)
    return commit_distance >= threshold, commit_distance


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
            f"No hotspot issue refresh needed: {commit_distance} commits since the last "
            f"reported head, threshold {args.threshold}."
        )
        return 0

    scan_text = scanner_output()
    body = build_issue_body(head_rev=head_rev, threshold=args.threshold, scan_text=scan_text, top_n=args.top)

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


if __name__ == "__main__":
    raise SystemExit(main())
