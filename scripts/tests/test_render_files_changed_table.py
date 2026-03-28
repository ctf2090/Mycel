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

    def test_renders_markdown_table_when_all_rows_have_notes(self) -> None:
        agents = self.root / "AGENTS.md"
        agents.write_text("rules\n", encoding="utf-8")
        tool = self.root / "scripts" / "tool.py"
        tool.write_text("print('hi')\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            "--note",
            "AGENTS.md=Clarify agent workflow instructions.",
            "--note",
            "scripts/tool.py=Adjust repo tooling behavior and command output.",
            stdin_text="12\t3\tAGENTS.md\n7\t0\tscripts/tool.py\n",
        )

        self.assertIn("| File | +/- | One-line note |", proc.stdout)
        self.assertIn(f"| [AGENTS.md]({agents.resolve()}) | +12 / -3 |", proc.stdout)
        self.assertIn(f"| [scripts/tool.py]({tool.resolve()}) | +7 / -0 |", proc.stdout)
        self.assertIn("Clarify agent workflow instructions.", proc.stdout)
        self.assertIn("Adjust repo tooling behavior and command output.", proc.stdout)
        self.assertTrue(
            proc.stdout.endswith("\n\n"),
            f"expected trailing blank line after table, got: {proc.stdout!r}",
        )

    def test_emits_blank_line_after_table_so_following_pipe_text_stays_outside_table(self) -> None:
        agents = self.root / "AGENTS.md"
        agents.write_text("rules\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            "--note",
            "AGENTS.md=Clarify agent workflow instructions.",
            stdin_text="12\t3\tAGENTS.md\n",
        )

        rendered = (
            proc.stdout
            + "[2026-03-28 23:54:13 UTC+8] After work | coding-3 | usage 146K/258K\n"
        )
        self.assertIn(
            "\n\n[2026-03-28 23:54:13 UTC+8] After work | coding-3 | usage 146K/258K\n",
            rendered,
        )

    def test_renders_clickable_delta_links_in_stdin_mode_with_diff_key(self) -> None:
        mailbox = self.root / ".agent-local" / "mailboxes" / "agt_test.md"
        mailbox.parent.mkdir(parents=True, exist_ok=True)
        mailbox.write_text("# mailbox\n\n- note\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            "--diff-key",
            "agent-local-mailbox",
            "--note",
            ".agent-local/mailboxes/agt_test.md=Mailbox state for the current agent.",
            stdin_text="3\t0\t.agent-local/mailboxes/agt_test.md\n",
        )

        self.assertIn(
            f"| [.agent-local/mailboxes/agt_test.md]({mailbox.resolve()}) | [+3 / -0](",
            proc.stdout,
        )
        diff_root = self.root / ".agent-local" / "rendered-diffs"
        generated = list(diff_root.rglob("agt_test.md.diff"))
        self.assertEqual(1, len(generated))
        diff_text = generated[0].read_text(encoding="utf-8")
        self.assertIn("+++ b/.agent-local/mailboxes/agt_test.md", diff_text)
        self.assertIn("+# mailbox", diff_text)

    def test_renders_zero_zero_stdin_rows_as_tracked_artifacts(self) -> None:
        mailbox = self.root / ".agent-local" / "mailboxes" / "agt_test.md"
        mailbox.parent.mkdir(parents=True, exist_ok=True)
        mailbox.write_text("# mailbox\n\n- note\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            "--diff-key",
            "agent-local-mailbox-zero-zero",
            "--note",
            ".agent-local/mailboxes/agt_test.md=Mailbox state for the current agent.",
            stdin_text="0\t0\t.agent-local/mailboxes/agt_test.md\n",
        )

        self.assertIn(
            f"| [.agent-local/mailboxes/agt_test.md]({mailbox.resolve()}) | [tracked artifact](",
            proc.stdout,
        )
        self.assertNotIn("+0 / -0", proc.stdout)

    def test_renders_clickable_delta_links_and_generates_diff_files(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)
        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update"], cwd=self.root, check=True, capture_output=True, text=True)

        proc = self.run_cli("HEAD", "--note", "AGENTS.md=Update AGENTS wording.")

        self.assertIn(f"| [AGENTS.md]({tracked.resolve()}) | [+1 / -0](", proc.stdout)
        diff_path = self.root / ".agent-local" / "rendered-diffs"
        generated = list(diff_path.rglob("AGENTS.md.diff"))
        self.assertEqual(1, len(generated))
        self.assertIn("+after", generated[0].read_text(encoding="utf-8"))

    def test_renders_commit_range_without_traceback_and_generates_range_diff(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)

        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update"], cwd=self.root, check=True, capture_output=True, text=True)

        proc = self.run_cli("HEAD~1..HEAD", "--note", "AGENTS.md=Render only the update commit range.")

        self.assertIn(f"| [AGENTS.md]({tracked.resolve()}) | [+1 / -0](", proc.stdout)
        generated = list((self.root / ".agent-local" / "rendered-diffs").rglob("AGENTS.md.diff"))
        self.assertEqual(1, len(generated))
        self.assertIn("+after", generated[0].read_text(encoding="utf-8"))

    def test_ignores_untracked_files_when_rendering_head_commit(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)

        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update"], cwd=self.root, check=True, capture_output=True, text=True)

        backlog = self.root / "docs" / "REFACTOR-HOTSPOTS.md"
        backlog.parent.mkdir(parents=True, exist_ok=True)
        backlog.write_text("# backlog\n", encoding="utf-8")

        proc = self.run_cli("HEAD", "--note", "AGENTS.md=Render the latest committed AGENTS change only.")

        self.assertIn(f"| [AGENTS.md]({tracked.resolve()}) | [+1 / -0](", proc.stdout)
        self.assertNotIn("REFACTOR-HOTSPOTS.md", proc.stdout)

    def test_reuses_git_ref_bucket_but_cleans_stale_diff_files(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)

        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update agents"], cwd=self.root, check=True, capture_output=True, text=True)
        self.run_cli("HEAD", "--note", "AGENTS.md=Update AGENTS wording.")

        tool = self.root / "scripts" / "tool.py"
        tool.write_text("print('hi')\n", encoding="utf-8")
        subprocess.run(["git", "add", "scripts/tool.py"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "add tool"], cwd=self.root, check=True, capture_output=True, text=True)
        self.run_cli("HEAD", "--note", "scripts/tool.py=Add helper tool.")

        diff_root = self.root / ".agent-local" / "rendered-diffs"
        self.assertEqual([], list(diff_root.rglob("AGENTS.md.diff")))
        generated = list(diff_root.rglob("scripts/tool.py.diff"))
        self.assertEqual(1, len(generated))
        self.assertIn("+print('hi')", generated[0].read_text(encoding="utf-8"))

    def test_clears_old_git_ref_buckets_when_rendering_new_bucket(self) -> None:
        tracked = self.root / "AGENTS.md"
        tracked.write_text("before\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)

        tracked.write_text("before\nafter\n", encoding="utf-8")
        subprocess.run(["git", "add", "AGENTS.md"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "update"], cwd=self.root, check=True, capture_output=True, text=True)
        self.run_cli("HEAD", "--note", "AGENTS.md=Update AGENTS wording.")

        self.run_cli("HEAD~1", "--note", "AGENTS.md=Render previous AGENTS snapshot.")

        diff_root = self.root / ".agent-local" / "rendered-diffs"
        buckets = [path.name for path in diff_root.iterdir() if path.is_dir()]
        self.assertEqual(1, len(buckets))
        generated = list(diff_root.rglob("AGENTS.md.diff"))
        self.assertEqual(1, len(generated))
        self.assertIn("before", generated[0].read_text(encoding="utf-8"))

    def test_supports_note_overrides(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--note",
            "scripts/tool.py=Generated Markdown table helper.",
            stdin_text="7\t0\tscripts/tool.py\n",
        )

        self.assertIn("Generated Markdown table helper.", proc.stdout)

    def test_stdin_diff_key_requires_existing_file(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--diff-key",
            "missing-path",
            "--note",
            "missing/file.txt=Document a missing path placeholder.",
            stdin_text="1\t0\tmissing/file.txt\n",
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("stdin diff path does not exist: missing/file.txt", proc.stderr)

    def test_uses_manual_note_overrides_instead_of_path_heuristics(self) -> None:
        roadmap = self.root / "ROADMAP.zh-TW.md"
        roadmap.write_text("roadmap\n", encoding="utf-8")
        progress = self.root / "pages" / "progress.html"
        progress.parent.mkdir(parents=True, exist_ok=True)
        progress.write_text("<html></html>\n", encoding="utf-8")

        proc = self.run_cli(
            "--stdin",
            "--note",
            "ROADMAP.zh-TW.md=Refresh roadmap status and milestone wording.",
            "--note",
            "pages/progress.html=Sync public progress summary with current planning state.",
            stdin_text="8\t2\tROADMAP.zh-TW.md\n5\t4\tpages/progress.html\n",
        )

        self.assertIn("Refresh roadmap status and milestone wording.", proc.stdout)
        self.assertIn("Sync public progress summary with current planning state.", proc.stdout)

    def test_renders_binary_diffs_as_na_when_noted(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--note",
            "assets/logo.png=Refresh the binary logo asset.",
            stdin_text="-\t-\tassets/logo.png\n",
        )

        self.assertIn("| assets/logo.png | +n/a / -n/a |", proc.stdout)
        self.assertIn("Refresh the binary logo asset.", proc.stdout)

    def test_rejects_invalid_note_argument(self) -> None:
        proc = self.run_cli("--stdin", "--note", "bad-note", stdin_text="1\t1\tfoo\n", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("invalid --note value", proc.stderr)

    def test_errors_when_any_changed_file_is_missing_a_note(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--note",
            "AGENTS.md=Clarify agent workflow instructions.",
            stdin_text="1\t0\tAGENTS.md\n1\t0\tmissing/file.txt\n",
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("missing required --note entries for: missing/file.txt", proc.stderr)

    def test_leaves_missing_file_paths_as_plain_text_when_noted(self) -> None:
        proc = self.run_cli(
            "--stdin",
            "--note",
            "missing/file.txt=Document a missing path placeholder.",
            stdin_text="1\t0\tmissing/file.txt\n",
        )

        self.assertIn("| missing/file.txt | +1 / -0 |", proc.stdout)


if __name__ == "__main__":
    unittest.main()
