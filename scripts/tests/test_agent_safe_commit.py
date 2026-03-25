import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_safe_commit.py"
SOURCE_TOKEN_USAGE = REPO_ROOT / "scripts" / "codex_token_usage_summary.py"


class AgentSafeCommitCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)

        target = scripts_dir / "agent_safe_commit.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)

        token_usage = scripts_dir / "codex_token_usage_summary.py"
        token_usage.write_text(SOURCE_TOKEN_USAGE.read_text(encoding="utf-8"), encoding="utf-8")
        token_usage.chmod(0o755)

        metadata = scripts_dir / "codex_thread_metadata.py"
        metadata.write_text(
            """#!/usr/bin/env python3
import sys

if "--shell" in sys.argv:
    print('MODEL="gpt-5.4"')
    print('EFFORT="medium"')
    print('THREAD_ID="thread-test"')
    raise SystemExit(0)
raise SystemExit(1)
""",
            encoding="utf-8",
        )
        metadata.chmod(0o755)

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

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "agent_safe_commit.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_workcycle_token_snapshots(
        self,
        agent_id: str,
        *,
        batch_num: int,
        start_input: int,
        end_input: int,
        start_total: int,
        end_total: int,
        thread_id: str = "thread-test",
    ) -> None:
        directory = self.root / ".agent-local" / "agents" / agent_id / "workcycles"
        directory.mkdir(parents=True, exist_ok=True)
        (directory / f"token-usage-{batch_num}.json").write_text(
            (
                "{\n"
                f'  "thread_id": "{thread_id}",\n'
                f'  "input_tokens": {start_input},\n'
                f'  "cumulative_total_tokens": {start_total}\n'
                "}\n"
            ),
            encoding="utf-8",
        )
        (directory / f"token-usage-end-{batch_num}.json").write_text(
            (
                "{\n"
                f'  "thread_id": "{thread_id}",\n'
                f'  "input_tokens": {end_input},\n'
                f'  "cumulative_total_tokens": {end_total}\n'
                "}\n"
            ),
            encoding="utf-8",
        )

    def test_commits_only_the_explicit_allowlist(self) -> None:
        doc = self.root / "docs.md"
        other = self.root / "other.md"
        doc.write_text("doc\n", encoding="utf-8")
        other.write_text("other\n", encoding="utf-8")
        self.write_workcycle_token_snapshots(
            "agt_test1234",
            batch_num=7,
            start_input=100000,
            end_input=145000,
            start_total=100000,
            end_total=2100000,
        )

        proc = self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--agent-id",
            "agt_test1234",
            "--model-id",
            "gpt-5-codex",
            "--message",
            "docs: add docs",
            "docs.md",
        )

        self.assertIn("docs: add docs", proc.stdout)
        show = subprocess.run(
            ["git", "show", "--name-only", "--format=", "HEAD"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(["docs.md"], [line for line in show.stdout.splitlines() if line.strip()])
        body = subprocess.run(
            ["git", "log", "-1", "--format=%B"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertIn("Agent-Id: agt_test1234", body.stdout)
        self.assertIn("Model: gpt-5.4", body.stdout)
        self.assertIn("Reasoning-Effort: medium", body.stdout)
        self.assertIn("Token-Spent: 45K", body.stdout)

    def test_rejects_extra_preexisting_staged_paths(self) -> None:
        doc = self.root / "docs.md"
        other = self.root / "other.md"
        doc.write_text("doc\n", encoding="utf-8")
        other.write_text("other\n", encoding="utf-8")

        subprocess.run(["git", "add", "other.md"], cwd=self.root, check=True, capture_output=True, text=True)

        proc = self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--agent-id",
            "agt_test1234",
            "--model-id",
            "gpt-5-codex",
            "--message",
            "docs: add docs",
            "docs.md",
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("extra staged paths: other.md", proc.stderr)

    def test_rejects_missing_paths(self) -> None:
        proc = self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--agent-id",
            "agt_test1234",
            "--model-id",
            "gpt-5-codex",
            "--message",
            "docs: add docs",
            "missing.md",
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("cannot stage missing paths: missing.md", proc.stderr)

    def test_requires_agent_id_argument(self) -> None:
        doc = self.root / "docs.md"
        doc.write_text("doc\n", encoding="utf-8")

        proc = self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--message",
            "docs: add docs",
            "docs.md",
            check=False,
        )

        self.assertEqual(2, proc.returncode)
        self.assertIn("--agent-id", proc.stderr)

    def test_does_not_require_model_id_argument(self) -> None:
        doc = self.root / "docs.md"
        doc.write_text("doc\n", encoding="utf-8")

        proc = self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--agent-id",
            "agt_test1234",
            "--message",
            "docs: add docs",
            "docs.md",
        )

        self.assertEqual(0, proc.returncode)

    def test_omits_token_spent_when_no_workcycle_snapshot_exists(self) -> None:
        doc = self.root / "docs.md"
        doc.write_text("doc\n", encoding="utf-8")

        self.run_cli(
            "--name",
            "gpt-5:doc-1",
            "--email",
            "agent@example.invalid",
            "--agent-id",
            "agt_test1234",
            "--message",
            "docs: add docs",
            "docs.md",
        )

        body = subprocess.run(
            ["git", "log", "-1", "--format=%B"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertNotIn("Token-Spent:", body.stdout)


if __name__ == "__main__":
    unittest.main()
