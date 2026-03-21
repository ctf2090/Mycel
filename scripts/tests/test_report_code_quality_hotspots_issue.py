import unittest

from scripts import report_code_quality_hotspots_issue as report


class ReportCodeQualityHotspotsIssueTest(unittest.TestCase):
    def test_extract_marker_returns_none_when_missing(self) -> None:
        self.assertIsNone(report.extract_marker("plain body", report.HEAD_MARKER))

    def test_extract_marker_reads_head_commit(self) -> None:
        body = "<!-- hotspot-report-head: abc123 -->"
        self.assertEqual("abc123", report.extract_marker(body, report.HEAD_MARKER))

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

    def test_build_issue_body_includes_hidden_markers(self) -> None:
        scan = "\n".join(
            [
                "Code-quality hotspot warnings...",
                "Summary: 1 file-size, 1 function-size",
                "",
                "Ranked split candidates:",
                "1. score=10 crates/a.rs | file 900 lines; long functions=1 [f@L1=120]; repeated literals=0 [none]",
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
        self.assertIn("Top split candidates", body)
        self.assertIn("crates/a.rs", body)


if __name__ == "__main__":
    unittest.main()
