import subprocess
import unittest
from argparse import Namespace
from pathlib import Path
from unittest import mock
import os

import scripts.gh_issue_comment as gh_issue_comment


class GhIssueCommentTest(unittest.TestCase):
    def test_comment_uses_stdin_body_file_path(self) -> None:
        with mock.patch.object(subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess(
                args=[],
                returncode=0,
                stdout="ok\n",
                stderr="",
            )

            gh_issue_comment.run_gh(
                ["gh", "issue", "comment", "12", "--body-file", "-"],
                body="hello\n`markdown`\n",
            )

        run_mock.assert_called_once_with(
            ["gh", "issue", "comment", "12", "--body-file", "-"],
            cwd=gh_issue_comment.ROOT_DIR,
            env=mock.ANY,
            text=True,
            input="hello\n`markdown`\n",
            capture_output=True,
            check=False,
        )

    def test_comment_prefers_gh_token_agent_when_present(self) -> None:
        with mock.patch.dict(os.environ, {"GH_TOKEN": "user-token", "GH_TOKEN_AGENT": "agent-token"}, clear=False):
            with mock.patch.object(subprocess, "run") as run_mock:
                run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout="", stderr="")
                gh_issue_comment.run_gh(["gh", "issue", "comment", "12", "--body-file", "-"], body="hello")

        used_env = run_mock.call_args.kwargs["env"]
        self.assertEqual("agent-token", used_env["GH_TOKEN"])

    def test_read_body_prefers_explicit_body(self) -> None:
        args = Namespace(body="inline", body_file="-", no_comment=False, require_body=True)
        self.assertEqual(gh_issue_comment.read_body(args), "inline")

    def test_read_body_loads_file_content(self) -> None:
        temp = Path(self.id().replace(".", "_") + ".md")
        self.addCleanup(lambda: temp.unlink(missing_ok=True))
        temp.write_text("from file\n", encoding="utf-8")

        args = Namespace(body=None, body_file=str(temp), no_comment=False, require_body=True)
        self.assertEqual(gh_issue_comment.read_body(args), "from file\n")

    def test_close_comments_first_then_closes_without_inline_comment(self) -> None:
        calls: list[list[str]] = []

        def fake_run(args, **kwargs):
            calls.append(args)
            return subprocess.CompletedProcess(args=args, returncode=0, stdout="", stderr="")

        with mock.patch.object(subprocess, "run", side_effect=fake_run):
            with mock.patch.object(
                gh_issue_comment,
                "parse_args",
                return_value=Namespace(
                    command="close",
                    issue="12",
                    repo=None,
                    body="safe markdown",
                    body_file="-",
                    no_comment=False,
                    require_body=False,
                    reason="completed",
                ),
            ):
                self.assertEqual(gh_issue_comment.main(), 0)

        self.assertEqual(
            calls,
            [
                ["gh", "issue", "comment", "12", "--body-file", "-"],
                ["gh", "issue", "close", "12", "--reason", "completed"],
            ],
        )


if __name__ == "__main__":
    unittest.main()
