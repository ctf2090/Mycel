import unittest

from scripts.gh_cli_env import preferred_gh_env, preferred_git_https_env, preferred_user_gh_env


class GhCliEnvTest(unittest.TestCase):
    def test_preferred_gh_env_keeps_gh_token_as_agent_default(self) -> None:
        env = preferred_gh_env({"GH_TOKEN": "agent-token", "GH_TOKEN_USER": "user-token"})
        self.assertEqual("agent-token", env["GH_TOKEN"])

    def test_preferred_gh_env_falls_back_to_legacy_gh_token_agent(self) -> None:
        env = preferred_gh_env({"GH_TOKEN": "", "GH_TOKEN_AGENT": "legacy-agent-token"})
        self.assertEqual("legacy-agent-token", env["GH_TOKEN"])

    def test_preferred_user_gh_env_promotes_gh_token_user(self) -> None:
        env = preferred_user_gh_env({"GH_TOKEN": "agent-token", "GH_TOKEN_USER": "user-token"})
        self.assertEqual("user-token", env["GH_TOKEN"])

    def test_preferred_git_https_env_promotes_agent_token_to_github_token(self) -> None:
        env = preferred_git_https_env({"GH_TOKEN": "agent-token", "GITHUB_TOKEN": "user-token"})
        self.assertEqual("agent-token", env["GITHUB_TOKEN"])
        self.assertEqual("0", env["GIT_TERMINAL_PROMPT"])


if __name__ == "__main__":
    unittest.main()
