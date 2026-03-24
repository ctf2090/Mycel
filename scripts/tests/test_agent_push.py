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
        gh_cli_env_target = self.root / "scripts" / "gh_cli_env.py"
        gh_cli_env_target.write_text(SOURCE_GH_CLI_ENV.read_text(encoding="utf-8"), encoding="utf-8")
        subprocess.run(["git", "init"], cwd=self.root, check=True, capture_output=True, text=True)

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


if __name__ == "__main__":
    unittest.main()
