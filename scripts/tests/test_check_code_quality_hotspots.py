import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "check_code_quality_hotspots.py"


class CheckCodeQualityHotspotsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        target = self.root / "scripts" / "check_code_quality_hotspots.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)
        (self.root / "apps").mkdir(parents=True, exist_ok=True)
        (self.root / "crates").mkdir(parents=True, exist_ok=True)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(self.root / "scripts" / "check_code_quality_hotspots.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )

    def test_reports_large_file_large_function_and_repeated_literals(self) -> None:
        rust_file = self.root / "crates" / "big.rs"
        rust_file.write_text(
            "\n".join(
                [
                    "fn too_big() {",
                    *["    let _value = \"repeated literal value\";" for _ in range(3)],
                    *["    let _n = 1;" for _ in range(5)],
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--file-lines",
            "4",
            "--function-lines",
            "4",
            "--literal-repeats",
            "3",
            "crates",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("[file-size] crates/big.rs:1", proc.stdout)
        self.assertIn("[function-size] crates/big.rs:1", proc.stdout)
        self.assertIn("[literal-repeat] crates/big.rs:2", proc.stdout)
        self.assertIn("Ranked split candidates:", proc.stdout)
        self.assertIn("1. score=", proc.stdout)
        self.assertIn("crates/big.rs", proc.stdout)

    def test_reports_repeated_numeric_literals(self) -> None:
        rust_file = self.root / "crates" / "numeric.rs"
        rust_file.write_text(
            "\n".join(
                [
                    "fn uses_magic_numbers() {",
                    "    let _timeout_a = 30;",
                    "    let _timeout_b = 30;",
                    "    let _timeout_c = 30;",
                    "    let _ok = 1;",
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--numeric-repeats",
            "3",
            "crates",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("[numeric-literal-repeat] crates/numeric.rs:2", proc.stdout)
        self.assertIn("numeric literals=1 [30@L2 x3]", proc.stdout)

    def test_ignores_common_small_numeric_literals(self) -> None:
        py_file = self.root / "apps" / "common_values.py"
        py_file.write_text(
            "\n".join(
                [
                    "def uses_common_values():",
                    "    first = 1",
                    "    second = 1",
                    "    third = 1",
                    "    return 0",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--numeric-repeats",
            "3",
            "apps",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("No code-quality hotspots found", proc.stdout)

    def test_ignores_rust_numbers_inside_strings_and_comments(self) -> None:
        rust_file = self.root / "crates" / "stringy.rs"
        rust_file.write_text(
            "\n".join(
                [
                    'fn avoids_false_positives() {',
                    '    let _message = "30 should stay text";',
                    "    let _real = 30;",
                    "    let _commented = 1; // 30 in a comment should not count",
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--numeric-repeats",
            "2",
            "crates",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("No code-quality hotspots found", proc.stdout)

    def test_supports_github_warning_and_fail_on_findings(self) -> None:
        py_file = self.root / "apps" / "warn.py"
        py_file.write_text(
            "\n".join(
                [
                    "def very_long_function():",
                    *["    value = 'another repeated literal'" for _ in range(3)],
                    *["    step = 1" for _ in range(5)],
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--file-lines",
            "4",
            "--function-lines",
            "4",
            "--literal-repeats",
            "3",
            "--github-warning",
            "--fail-on-findings",
            "apps",
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("::warning file=apps/warn.py,line=1::", proc.stdout)
        self.assertIn("Summary:", proc.stdout)

    def test_ranks_more_complex_files_first(self) -> None:
        first = self.root / "crates" / "first.rs"
        first.write_text(
            "\n".join(
                [
                    "fn first_big() {",
                    *["    let _value = \"shared repeated literal\";" for _ in range(4)],
                    *["    let _n = 1;" for _ in range(8)],
                    "}",
                    "",
                    "fn second_big() {",
                    *["    let _value = \"shared repeated literal\";" for _ in range(4)],
                    *["    let _n = 2;" for _ in range(8)],
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        second = self.root / "crates" / "second.rs"
        second.write_text(
            "\n".join(
                [
                    "fn medium() {",
                    *["    let _n = 1;" for _ in range(5)],
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--file-lines",
            "8",
            "--function-lines",
            "6",
            "--literal-repeats",
            "3",
            "crates",
        )

        self.assertEqual(0, proc.returncode)
        ranked_lines = [
            line for line in proc.stdout.splitlines() if line.startswith(("1. score=", "2. score="))
        ]
        self.assertEqual(2, len(ranked_lines))
        self.assertIn("crates/first.rs", ranked_lines[0])
        self.assertIn("crates/second.rs", ranked_lines[1])

    def test_reports_clean_when_no_hotspots_found(self) -> None:
        rust_file = self.root / "crates" / "small.rs"
        rust_file.write_text("fn ok() {\n    let _value = \"short\";\n}\n", encoding="utf-8")

        proc = self.run_cli("crates")

        self.assertEqual(0, proc.returncode)
        self.assertIn("No code-quality hotspots found", proc.stdout)


if __name__ == "__main__":
    unittest.main()
