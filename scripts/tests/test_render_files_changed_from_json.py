import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_TABLE_SCRIPT = REPO_ROOT / "scripts" / "render_files_changed_table.py"
SOURCE_WRAPPER_SCRIPT = REPO_ROOT / "scripts" / "render_files_changed_from_json.py"


class RenderFilesChangedFromJsonCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        (scripts_dir / "render_files_changed_table.py").write_text(
            SOURCE_TABLE_SCRIPT.read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        wrapper = scripts_dir / "render_files_changed_from_json.py"
        wrapper.write_text(SOURCE_WRAPPER_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        wrapper.chmod(0o755)
        subprocess.run(["git", "init"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(
            ["git", "config", "user.name", "Test User"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(
        self, *args: str, stdin_text: str = "", check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "render_files_changed_from_json.py"), *args],
            cwd=self.root,
            text=True,
            input=stdin_text,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_renders_table_from_stdin_json_without_shell_escaped_notes(self) -> None:
        mailbox = self.root / ".agent-local" / "mailboxes" / "agt_test.md"
        mailbox.parent.mkdir(parents=True, exist_ok=True)
        mailbox.write_text("# mailbox\n", encoding="utf-8")

        spec = {
            "rows": [
                {
                    "path": ".agent-local/mailboxes/agt_test.md",
                    "added": "0",
                    "removed": "0",
                    "note": "Marked this cycle's mailbox note without shell escaping.",
                }
            ]
        }

        proc = self.run_cli("--diff-key", "json-wrapper", "-", stdin_text=json.dumps(spec))

        self.assertIn("[tracked artifact](", proc.stdout)
        self.assertIn("Marked this cycle's mailbox note without shell escaping.", proc.stdout)

    def test_rejects_invalid_json_spec(self) -> None:
        proc = self.run_cli("--diff-key", "json-wrapper", "-", stdin_text="{", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("invalid JSON spec", proc.stderr)

    def test_rejects_missing_note_field(self) -> None:
        spec = {"rows": [{"path": "foo", "added": "1", "removed": "0"}]}

        proc = self.run_cli("--diff-key", "json-wrapper", "-", stdin_text=json.dumps(spec), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("row 1 must provide non-empty string values", proc.stderr)


if __name__ == "__main__":
    unittest.main()
