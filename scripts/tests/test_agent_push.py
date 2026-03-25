import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts.agent_push import build_push_command
from scripts.gh_cli_env import preferred_git_https_env


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_push.py"
SOURCE_GH_CLI_ENV = REPO_ROOT / "scripts" / "gh_cli_env.py"
SOURCE_GUARD = REPO_ROOT / "scripts" / "agent_guard.py"
SOURCE_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentPushHelpersTest(unittest.TestCase):
    def test_preferred_git_https_env_promotes_agent_token_for_codespaces_git(self) -> None:
        env = preferred_git_https_env(
            {
                "GH_TOKEN": "agent-token",
                "GITHUB_TOKEN": "user-token",
            }
        )
        self.assertEqual("agent-token", env["GH_TOKEN"])
        self.assertEqual("agent-token", env["GITHUB_TOKEN"])
        self.assertEqual("0", env["GIT_TERMINAL_PROMPT"])
        self.assertEqual("https://github.com", env["GITHUB_SERVER_URL"])

    def test_build_push_command_uses_explicit_refspec(self) -> None:
        args = type(
            "Args",
            (),
            {
                "commit_ref": "abc1234",
                "remote": "origin",
                "branch": "main",
                "dry_run": False,
            },
        )()
        self.assertEqual(["git", "push", "origin", "abc1234:main"], build_push_command(args))


class AgentPushCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        agent_push_target = self.root / "scripts" / "agent_push.py"
        agent_push_target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        agent_push_target.chmod(0o755)
        guard_target = self.root / "scripts" / "agent_guard.py"
        guard_target.write_text(SOURCE_GUARD.read_text(encoding="utf-8"), encoding="utf-8")
        guard_target.chmod(0o755)
        gh_cli_env_target = self.root / "scripts" / "gh_cli_env.py"
        gh_cli_env_target.write_text(SOURCE_GH_CLI_ENV.read_text(encoding="utf-8"), encoding="utf-8")
        gh_cli_env_target.chmod(0o755)
        registry_target = self.root / "scripts" / "agent_registry.py"
        registry_target.write_text(SOURCE_REGISTRY.read_text(encoding="utf-8"), encoding="utf-8")
        registry_target.chmod(0o755)
        checklist_target = self.root / "scripts" / "item_id_checklist.py"
        checklist_target.write_text(SOURCE_CHECKLIST.read_text(encoding="utf-8"), encoding="utf-8")
        checklist_target.chmod(0o755)
        marker_target = self.root / "scripts" / "item_id_checklist_mark.py"
        marker_target.write_text(SOURCE_MARKER.read_text(encoding="utf-8"), encoding="utf-8")
        marker_target.chmod(0o755)
        subprocess.run(["git", "init"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.root, check=True, capture_output=True, text=True)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
        run_env = dict(os.environ)
        if env:
            run_env.update(env)
        return subprocess.run(
            ["python3", str(self.root / "scripts" / "agent_push.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=run_env,
            check=False,
        )

    def test_cli_requires_gh_token(self) -> None:
        proc = self.run_cli("HEAD", env={"GH_TOKEN": "", "GITHUB_TOKEN": ""})
        self.assertEqual(1, proc.returncode)
        self.assertIn("GH_TOKEN is required", proc.stderr)

    def test_cli_supports_dry_run_with_explicit_refspec(self) -> None:
        proc = self.run_cli(
            "HEAD",
            "--dry-run",
            env={"GH_TOKEN": "agent-token", "GITHUB_TOKEN": "user-token"},
        )
        self.assertNotEqual(0, proc.returncode)
        self.assertIn("origin", proc.stderr)

    def test_cli_rejects_push_when_commit_agent_is_blocked(self) -> None:
        tracked = self.root / "tracked.txt"
        tracked.write_text("one\n", encoding="utf-8")
        subprocess.run(["git", "add", "tracked.txt"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(
            [
                "git",
                "commit",
                "--no-gpg-sign",
                "-m",
                "guarded commit\n\nAgent-Id: agt_test1234\nModel: gpt-5.4\nReasoning-Effort: medium",
            ],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        (self.root / ".agent-local" / "runtime").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "runtime" / "agent-blocks.json").write_text(
            (
                "{\n"
                '  "version": 1,\n'
                '  "blocks": {\n'
                '    "agt_test1234": {\n'
                '      "blocked": true,\n'
                '      "reason": "compact_context_detected",\n'
                '      "detected_at": "2026-03-25T15:28:43.925Z",\n'
                '      "source": "agent_work_cycle.begin",\n'
                '      "handoff_path": ".agent-local/mailboxes/agt_test1234.md",\n'
                '      "clear_requires": "new_chat_bootstrap"\n'
                "    }\n"
                "  }\n"
                "}\n"
            ),
            encoding="utf-8",
        )
        (self.root / ".agent-local" / "agents.json").write_text(
            (
                "{\n"
                '  "version": 2,\n'
                '  "updated_at": "2026-03-25T12:00:00+0800",\n'
                '  "agent_count": 1,\n'
                '  "agents": [\n'
                "    {\n"
                '      "agent_uid": "agt_test1234",\n'
                '      "role": "coding",\n'
                '      "current_display_id": "coding-1",\n'
                '      "display_history": [],\n'
                '      "assigned_by": "user",\n'
                '      "assigned_at": "2026-03-25T12:00:00+0800",\n'
                '      "confirmed_by_agent": true,\n'
                '      "confirmed_at": "2026-03-25T12:00:00+0800",\n'
                '      "last_touched_at": "2026-03-25T12:00:00+0800",\n'
                '      "inactive_at": null,\n'
                '      "paused_at": null,\n'
                '      "status": "inactive",\n'
                '      "scope": "guard-test",\n'
                '      "files": [],\n'
                '      "mailbox": ".agent-local/mailboxes/agt_test1234.md",\n'
                '      "recovery_of": null,\n'
                '      "superseded_by": null\n'
                "    }\n"
                "  ]\n"
                "}\n"
            ),
            encoding="utf-8",
        )

        proc = self.run_cli("HEAD", env={"GH_TOKEN": "agent-token", "GITHUB_TOKEN": "user-token"})

        self.assertEqual(10, proc.returncode)
        self.assertIn("agent execution blocked", proc.stderr)


if __name__ == "__main__":
    unittest.main()
