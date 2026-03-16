import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "render_files_changed_table.py"


class RenderFilesChangedTableCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        target = self.root / "scripts" / "render_files_changed_table.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, stdin_text: str = "", check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "render_files_changed_table.py"), *args],
            cwd=self.root,
            text=True,
            input=stdin_text,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_renders_markdown_table_with_colored_deltas(self) -> None:
        proc = self.run_cli(
            "--stdin",
            stdin_text="12\t3\tAGENTS.md\n7\t0\tscripts/tool.py\n",
        )

        self.assertIn("| File | +/- | One-line note |", proc.stdout)
        self.assertIn('<span style="color: #1a7f37;">+12</span>', proc.stdout)
        self.assertIn('<span style="color: #cf222e;">-3</span>', proc.stdout)
        self.assertIn("Updated content in this commit.", proc.stdout)
        self.assertIn("Added content in this commit.", proc.stdout)

    def test_supports_note_overrides(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--note",
            "scripts/tool.py=Generated Markdown table helper.",
            stdin_text="7\t0\tscripts/tool.py\n",
        )

        self.assertIn("Generated Markdown table helper.", proc.stdout)

    def test_renders_binary_diffs_as_na(self) -> None:
        proc = self.run_cli(
            "--stdin",
            stdin_text="-\t-\tassets/logo.png\n",
        )

        self.assertIn('<span style="color: #6e7781;">+n/a</span>', proc.stdout)
        self.assertIn('<span style="color: #6e7781;">-n/a</span>', proc.stdout)
        self.assertIn("Binary or non-line diff in this commit.", proc.stdout)

    def test_rejects_invalid_note_argument(self) -> None:
        proc = self.run_cli("--stdin", "--note", "bad-note", stdin_text="1\t1\tfoo\n", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("invalid --note value", proc.stderr)


if __name__ == "__main__":
    unittest.main()
