import os
import sys
import unittest
from unittest import mock
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts import report_mature_tool_review_issue as report


class ReportMatureToolReviewIssueTest(unittest.TestCase):
    def test_run_cmd_keeps_agent_token(self) -> None:
        recorded: dict[str, object] = {}

        def fake_run(args, **kwargs):
            recorded["env"] = kwargs["env"]
            return report.subprocess.CompletedProcess(args=args, returncode=0, stdout="ok", stderr="")

        with mock.patch.dict(os.environ, {"GH_TOKEN": "agent-token", "GH_TOKEN_USER": "user-token"}, clear=False):
            with mock.patch.object(report.subprocess, "run", side_effect=fake_run):
                self.assertEqual("ok", report.run_cmd(["gh", "issue", "list"]))

        self.assertEqual("agent-token", recorded["env"]["GH_TOKEN"])

    def test_extract_marker_reads_head_commit(self) -> None:
        body = "<!-- mature-tool-review-head: abc123 -->"
        self.assertEqual("abc123", report.extract_marker(body, report.HEAD_MARKER))

    def test_commits_since_returns_large_distance_when_base_revision_is_missing(self) -> None:
        original_revision_exists = report.revision_exists
        try:
            report.revision_exists = lambda rev: False
            self.assertEqual(10**9, report.commits_since("deadbeef", "abc123def456"))
        finally:
            report.revision_exists = original_revision_exists

    def test_build_issue_body_includes_hidden_markers_and_runbook(self) -> None:
        body = report.build_issue_body(head_rev="abc123def456", threshold=400)
        self.assertIn("<!-- mature-tool-review-head: abc123def456 -->", body)
        self.assertIn("<!-- mature-tool-review-threshold: 400 -->", body)
        self.assertIn("Mature Tool Review (`abc123d`)", body)
        self.assertIn("Review owner: `doc`", body)
        self.assertIn("docs/MATURE-TOOL-REVIEW-FLOW.md", body)
        self.assertIn("--threshold 400", body)

    def test_close_matching_open_issues_closes_only_open_matches(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(repo=None, title=report.DEFAULT_TITLE)
        issues = [
            report.IssueRecord(number=7, title=report.DEFAULT_TITLE, state="OPEN", body=""),
            report.IssueRecord(number=8, title=report.DEFAULT_TITLE, state="CLOSED", body=""),
            report.IssueRecord(number=9, title=report.DEFAULT_TITLE, state="open", body=""),
        ]
        closed_numbers: list[int] = []
        original_close_issue = report.close_issue
        try:
            report.close_issue = lambda current_args, issue_number: closed_numbers.append(issue_number)
            result = report.close_matching_open_issues(args, issues)
        finally:
            report.close_issue = original_close_issue
        self.assertEqual([9, 7], result)
        self.assertEqual([9, 7], closed_numbers)

    def test_main_skips_when_threshold_not_met(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            threshold=400,
            title=report.DEFAULT_TITLE,
            labels=None,
            repo=None,
            dry_run=False,
        )
        original_parse_args = report.parse_args
        original_current_head = report.current_head
        original_list_matching_issues = report.list_matching_issues
        original_issue_needs_refresh = report.issue_needs_refresh
        try:
            report.parse_args = lambda: args
            report.current_head = lambda: "abc123def456"
            report.list_matching_issues = lambda current_args: [
                report.IssueRecord(number=12, title=report.DEFAULT_TITLE, state="OPEN", body="")
            ]
            report.issue_needs_refresh = lambda issue, *, head_rev, threshold: (False, 37)
            with mock.patch("builtins.print") as print_mock:
                self.assertEqual(0, report.main())
        finally:
            report.parse_args = original_parse_args
            report.current_head = original_current_head
            report.list_matching_issues = original_list_matching_issues
            report.issue_needs_refresh = original_issue_needs_refresh
        print_mock.assert_any_call(
            "No mature-tool review refresh needed: 37 commits since the last reported head, threshold 400."
        )

    def test_create_issue_uses_periodic_review_default_label(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            repo=None,
            title=report.DEFAULT_TITLE,
            labels=None,
        )
        recorded: list[list[str]] = []
        original_run_cmd = report.run_cmd
        original_repo_labels = report.repo_labels
        try:
            report.repo_labels = lambda current_args: {"periodic-review"}
            report.run_cmd = lambda cmd, *, input_text=None: recorded.append(list(cmd)) or "created"
            report.create_issue(args, "body")
        finally:
            report.repo_labels = original_repo_labels
            report.run_cmd = original_run_cmd
        self.assertEqual(
            [
                "gh",
                "issue",
                "create",
                "--title",
                report.DEFAULT_TITLE,
                "--body-file",
                "-",
                "--label",
                "periodic-review",
            ],
            recorded[0],
        )
