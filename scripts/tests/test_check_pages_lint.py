import os
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "check-pages-lint.py"


class CheckPagesLintCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "check-pages-lint.py")
        (self.root / "scripts" / "check-pages-lint.py").chmod(0o755)
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_stub(self, name: str, body: str) -> None:
        path = self.bin_dir / name
        path.write_text(body, encoding="utf-8")
        path.chmod(0o755)

    def run_cli(self, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
        merged_env = os.environ.copy()
        if env:
            merged_env.update(env)
        return subprocess.run(
            [sys.executable, str(self.root / "scripts" / "check-pages-lint.py")],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=merged_env,
        )

    def test_exits_cleanly_when_no_pages_changes_are_staged(self) -> None:
        self.write_stub("git", "#!/usr/bin/env bash\nexit 0\n")

        proc = self.run_cli(env={"PATH": f"{self.bin_dir}:/usr/bin:/bin"})

        self.assertEqual(0, proc.returncode)
        self.assertEqual("", proc.stdout)

    def test_runs_npm_lint_when_pages_changes_are_staged(self) -> None:
        log_path = self.root / "calls.log"
        self.write_stub("git", "#!/usr/bin/env bash\nexit 1\n")
        self.write_stub(
            "npm",
            textwrap.dedent(
                f"""\
                #!/usr/bin/env python3
                import pathlib
                import sys
                pathlib.Path({str(log_path)!r}).write_text(" ".join(sys.argv[1:]), encoding="utf-8")
                raise SystemExit(0)
                """
            ),
        )

        proc = self.run_cli(env={"PATH": f"{self.bin_dir}:/usr/bin:/bin"})

        self.assertEqual(0, proc.returncode)
        self.assertIn("pages/ changes detected; running local Pages lint...", proc.stdout)
        self.assertEqual("run lint:pages", log_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
