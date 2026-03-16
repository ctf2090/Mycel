import subprocess
import tempfile
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
        subprocess.run(["git", "init"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.root, check=True, capture_output=True, text=True)

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

    def test_renders_markdown_table_with_plain_deltas_from_stdin(self) -> None:
        agents = self.root / "AGENTS.md"
        agents.write_text("rules\n", encoding="utf-8")
        tool = self.root / "scripts" / "tool.py"
        tool.write_text("print('hi')\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            stdin_text="12\t3\tAGENTS.md\n7\t0\tscripts/tool.py\n",
        )

        self.assertIn("| File | +/- | One-line note |", proc.stdout)
        self.assertIn(f"| [AGENTS.md]({agents.resolve()}) | +12 / -3 |", proc.stdout)
        self.assertIn(f"| [scripts/tool.py]({tool.resolve()}) | +7 / -0 |", proc.stdout)
        self.assertIn("Updated content in this commit.", proc.stdout)
        self.assertIn("Added content in this commit.", proc.stdout)

    def test_renders_clickable_delta_links_and_generates_diff_files(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)
        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update"], cwd=self.root, check=True, capture_output=True, text=True)

        proc = self.run_cli("HEAD")

        self.assertIn(f"| [AGENTS.md]({tracked.resolve()}) | [+1 / -0](", proc.stdout)
        diff_path = self.root / ".agent-local" / "rendered-diffs"
        generated = list(diff_path.rglob("AGENTS.md.diff"))
        self.assertEqual(1, len(generated))
        self.assertIn("+after", generated[0].read_text(encoding="utf-8"))

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

        self.assertIn("| assets/logo.png | +n/a / -n/a |", proc.stdout)
        self.assertIn("Binary or non-line diff in this commit.", proc.stdout)

    def test_rejects_invalid_note_argument(self) -> None:
        proc = self.run_cli("--stdin", "--note", "bad-note", stdin_text="1\t1\tfoo\n", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("invalid --note value", proc.stderr)

    def test_leaves_missing_file_paths_as_plain_text(self) -> None:
        proc = self.run_cli(
            "--stdin",
            stdin_text="1\t0\tmissing/file.txt\n",
        )

        self.assertIn("| missing/file.txt | +1 / -0 |", proc.stdout)


if __name__ == "__main__":
    unittest.main()
