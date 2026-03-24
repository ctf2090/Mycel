import os
import unittest
from unittest import mock

from scripts import report_code_quality_hotspots_issue as report


class ReportCodeQualityHotspotsIssueTest(unittest.TestCase):
    def test_run_cmd_keeps_gh_token_when_it_is_already_the_agent_identity(self) -> None:
        recorded: dict[str, object] = {}

        def fake_run(args, **kwargs):
            recorded["env"] = kwargs["env"]
            return report.subprocess.CompletedProcess(args=args, returncode=0, stdout="ok", stderr="")

        with mock.patch.dict(os.environ, {"GH_TOKEN": "agent-token", "GH_TOKEN_USER": "user-token"}, clear=False):
            with mock.patch.object(report.subprocess, "run", side_effect=fake_run):
                self.assertEqual("ok", report.run_cmd(["gh", "issue", "list"]))

        self.assertEqual("agent-token", recorded["env"]["GH_TOKEN"])

    def test_run_cmd_falls_back_to_legacy_gh_token_agent_when_needed(self) -> None:
        recorded: dict[str, object] = {}

        def fake_run(args, **kwargs):
            recorded["env"] = kwargs["env"]
            return report.subprocess.CompletedProcess(args=args, returncode=0, stdout="ok", stderr="")

        with mock.patch.dict(os.environ, {"GH_TOKEN": "", "GH_TOKEN_AGENT": "agent-token"}, clear=False):
            with mock.patch.object(report.subprocess, "run", side_effect=fake_run):
                self.assertEqual("ok", report.run_cmd(["gh", "issue", "list"]))

        self.assertEqual("agent-token", recorded["env"]["GH_TOKEN"])

    def test_list_matching_issues_accepts_legacy_title(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            repo=None,
            title=report.DEFAULT_TITLE,
        )
        original_run_cmd = report.run_cmd
        try:
            report.run_cmd = lambda *a, **k: (
                '[{"number":7,"title":"[Report] Code Quality Hotspots","state":"OPEN","body":"legacy"},'
                '{"number":8,"title":"Code Quality Hotspots","state":"OPEN","body":"current"},'
                '{"number":9,"title":"Something Else","state":"OPEN","body":"skip"}]'
            )
            issues = report.list_matching_issues(args)
        finally:
            report.run_cmd = original_run_cmd
        self.assertEqual([7, 8], [issue.number for issue in issues])
        self.assertEqual(["OPEN", "OPEN"], [issue.state for issue in issues])

    def test_extract_marker_returns_none_when_missing(self) -> None:
        self.assertIsNone(report.extract_marker("plain body", report.HEAD_MARKER))

    def test_extract_marker_reads_head_commit(self) -> None:
        body = "<!-- hotspot-report-head: abc123 -->"
        self.assertEqual("abc123", report.extract_marker(body, report.HEAD_MARKER))

    def test_commits_since_returns_large_distance_when_base_revision_is_missing(self) -> None:
        original_revision_exists = report.revision_exists
        try:
            report.revision_exists = lambda rev: False
            self.assertEqual(10**9, report.commits_since("deadbeef", "abc123def456"))
        finally:
            report.revision_exists = original_revision_exists

    def test_ranked_candidates_keeps_top_n(self) -> None:
        scan = "\n".join(
            [
                "Code-quality hotspot warnings...",
                "Summary: 2 file-size",
                "",
                "Ranked split candidates:",
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]",
                "2. score=8 crates/b.rs | file 850 lines; long functions=1 [g@L2=101]; repeated literals=0 [none]",
                "3. score=7 crates/c.rs | file 820 lines; long functions=0 [none]; repeated literals=1 [L9 x3]",
            ]
        )
        self.assertEqual(
            [
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]",
                "2. score=8 crates/b.rs | file 850 lines; long functions=1 [g@L2=101]; repeated literals=0 [none]",
            ],
            report.ranked_candidates(scan, 2),
        )

    def test_categorized_hotspots_groups_ranked_candidates_by_kind(self) -> None:
        scan = "\n".join(
            [
                "Code-quality hotspot warnings...",
                "Summary: 2 file-size, 1 function-size, 1 literal-repeat, 1 numeric-literal-repeat",
                "",
                "Ranked split candidates:",
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]; numeric literals=0 [none]",
                "2. score=8 crates/b.rs | file 850 lines (under file threshold); long functions=0 [none]; repeated literals=1 [L9 x3]; numeric literals=1 [7@L10 x4]",
            ]
        )
        grouped = report.categorized_hotspots(scan, 5)
        self.assertEqual(
            ["1. crates/a.rs (rank 1, score=10, 900 lines)"],
            grouped["file-size"],
        )
        self.assertEqual(
            ["1. crates/a.rs (rank 1, score=10, 1 long functions: f@L1=120)"],
            grouped["function-size"],
        )
        self.assertEqual(
            ["1. crates/b.rs (rank 2, score=8, 1 repeated literals: L9 x3)"],
            grouped["literal-repeat"],
        )
        self.assertEqual(
            ["1. crates/b.rs (rank 2, score=8, 1 numeric literal repeats: 7@L10 x4)"],
            grouped["numeric-literal-repeat"],
        )

    def test_file_hotspots_renders_ranked_file_summary_list(self) -> None:
        scan = "\n".join(
            [
                "Code-quality hotspot warnings...",
                "Summary: 2 file-size, 1 function-size, 1 literal-repeat, 1 numeric-literal-repeat",
                "",
                "Ranked split candidates:",
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]; numeric literals=0 [none]",
                "2. score=8 crates/b.rs | file 850 lines (under file threshold); long functions=0 [none]; repeated literals=1 [L9 x3]; numeric literals=1 [7@L10 x4]",
            ]
        )
        self.assertEqual(
            [
                "1. crates/a.rs (rank 1, score=10; file lines=900; long functions=1; repeated literals=0; numeric literal repeats=0)",
                "2. crates/b.rs (rank 2, score=8; file lines=850; long functions=0; repeated literals=1; numeric literal repeats=1)",
            ],
            report.file_hotspots(scan, 5),
        )

    def test_build_issue_body_includes_hidden_markers(self) -> None:
        scan = "\n".join(
            [
                "Code-quality hotspot warnings...",
                "Summary: 1 file-size, 1 function-size",
                "",
                "Ranked split candidates:",
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]; numeric literals=0 [none]",
            ]
        )
        body = report.build_issue_body(
            head_rev="abc123def456",
            threshold=20,
            scan_text=scan,
            top_n=5,
        )
        self.assertIn("<!-- hotspot-report-head: abc123def456 -->", body)
        self.assertIn("<!-- hotspot-report-threshold: 20 -->", body)
        self.assertIn("Hotspots by category", body)
        self.assertIn("Hotspots by files", body)
        self.assertIn("### `file-size`", body)
        self.assertIn("### `function-size`", body)
        self.assertIn("## Snapshot", body)
        self.assertIn("## Manual refresh", body)
        self.assertIn("Code Quality Hotspots (`abc123d`)", body)
        self.assertIn("Refresh threshold: `20` commits", body)
        self.assertIn("--title 'Code Quality Hotspots'", body)
        self.assertIn("1. crates/a.rs (rank 1, score=10, 900 lines)", body)
        self.assertIn(
            "1. crates/a.rs (rank 1, score=10; file lines=900; long functions=1; repeated literals=0; numeric literal repeats=0)",
            body,
        )
        self.assertNotIn("From the top 5 ranked hotspot candidates.", body)

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

    def test_main_closes_open_issue_and_creates_new_one(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            threshold=20,
            title=report.DEFAULT_TITLE,
            labels=None,
            top=8,
            repo=None,
            dry_run=False,
        )
        original_parse_args = report.parse_args
        original_current_head = report.current_head
        original_list_matching_issues = report.list_matching_issues
        original_issue_needs_refresh = report.issue_needs_refresh
        original_scanner_output = report.scanner_output
        original_build_issue_body = report.build_issue_body
        original_close_matching_open_issues = report.close_matching_open_issues
        original_create_issue = report.create_issue
        events: list[object] = []
        try:
            report.parse_args = lambda: args
            report.current_head = lambda: "abc123def456"
            report.list_matching_issues = lambda current_args: [
                report.IssueRecord(number=12, title=report.DEFAULT_TITLE, state="OPEN", body=""),
            ]
            report.issue_needs_refresh = lambda issue, *, head_rev, threshold: (True, threshold)
            report.scanner_output = lambda: "Summary: 0 findings"
            report.build_issue_body = lambda **kwargs: "body"
            report.close_matching_open_issues = (
                lambda current_args, issues: events.append(("close", [issue.number for issue in issues])) or [12]
            )
            report.create_issue = lambda current_args, body: events.append(("create", body))
            self.assertEqual(0, report.main())
        finally:
            report.parse_args = original_parse_args
            report.current_head = original_current_head
            report.list_matching_issues = original_list_matching_issues
            report.issue_needs_refresh = original_issue_needs_refresh
            report.scanner_output = original_scanner_output
            report.build_issue_body = original_build_issue_body
            report.close_matching_open_issues = original_close_matching_open_issues
            report.create_issue = original_create_issue
        self.assertEqual([("close", [12]), ("create", "body")], events)

    def test_create_issue_uses_code_quality_hotspot_default_label(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            repo=None,
            title=report.DEFAULT_TITLE,
            labels=None,
        )
        recorded: list[list[str]] = []
        original_run_cmd = report.run_cmd
        original_repo_labels = report.repo_labels
        try:
            report.repo_labels = lambda current_args: {"code-quality-hotspot"}
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
                "code-quality-hotspot",
            ],
            recorded[0],
        )

    def test_create_issue_skips_missing_labels(self) -> None:
        args = report.parse_args.__globals__["argparse"].Namespace(
            repo=None,
            title=report.DEFAULT_TITLE,
            labels=["code-quality-hotspot", "missing-label"],
        )
        recorded: list[list[str]] = []
        original_run_cmd = report.run_cmd
        original_repo_labels = report.repo_labels
        try:
            report.repo_labels = lambda current_args: {"code-quality-hotspot"}
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
                "code-quality-hotspot",
            ],
            recorded[0],
        )


if __name__ == "__main__":
    unittest.main()
