import shutil
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CHECK_SCRIPT = REPO_ROOT / "scripts" / "check-labels.py"
SYNC_SCRIPT = REPO_ROOT / "scripts" / "sync-labels.py"
LIB_SCRIPT = REPO_ROOT / "scripts" / "github_labels_lib.py"


class LabelsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".github").mkdir(parents=True, exist_ok=True)
        shutil.copy2(CHECK_SCRIPT, self.root / "scripts" / "check-labels.py")
        shutil.copy2(SYNC_SCRIPT, self.root / "scripts" / "sync-labels.py")
        shutil.copy2(LIB_SCRIPT, self.root / "scripts" / "github_labels_lib.py")
        (self.root / "scripts" / "check-labels.py").chmod(0o755)
        (self.root / "scripts" / "sync-labels.py").chmod(0o755)
        self.write_labels_file()
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def env(self) -> dict[str, str]:
        return {"PATH": f"{self.bin_dir}:/usr/bin:/bin"}

    def write_labels_file(self) -> None:
        (self.root / ".github" / "labels.yml").write_text(
            textwrap.dedent(
                """\
                labels:
                  - name: ai-ready
                    color: "0e8a16"
                    description: Well-scoped task with clear acceptance criteria and verification commands.
                  - name: tests-needed
                    color: "5319e7"
                    description: Needs direct test, fixture, or regression coverage as part of completion.
                """
            ),
            encoding="utf-8",
        )

    def write_gh(self, body: str) -> None:
        path = self.bin_dir / "gh"
        path.write_text(body, encoding="utf-8")
        path.chmod(0o755)

    def run_check(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "check-labels.py"), *args],
            cwd=self.root,
            env=self.env(),
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"check-labels failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def run_sync(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "sync-labels.py"), *args],
            cwd=self.root,
            env=self.env(),
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"sync-labels failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_check_labels_reports_success_when_labels_match(self) -> None:
        self.write_gh(
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import json
                import sys
                if sys.argv[1:5] == ["label", "list", "--limit", "200"]:
                    print(json.dumps([
                        {"name": "ai-ready", "color": "0E8A16", "description": "Well-scoped task with clear acceptance criteria and verification commands."},
                        {"name": "tests-needed", "color": "5319E7", "description": "Needs direct test, fixture, or regression coverage as part of completion."}
                    ]))
                    raise SystemExit(0)
                raise SystemExit(1)
                """
            ),
        )

        proc = self.run_check()
        self.assertEqual("tracked labels are in sync for 2 labels", proc.stdout.strip())

    def test_sync_labels_creates_each_tracked_label(self) -> None:
        calls_path = self.root / "gh_calls.txt"
        self.write_gh(
            textwrap.dedent(
                f"""\
                #!/usr/bin/env python3
                import pathlib
                import sys
                path = pathlib.Path({str(calls_path)!r})
                existing = path.read_text(encoding="utf-8") if path.exists() else ""
                path.write_text(existing + " ".join(sys.argv[1:]) + "\\n", encoding="utf-8")
                raise SystemExit(0)
                """
            ),
        )

        proc = self.run_sync()
        calls = calls_path.read_text(encoding="utf-8").splitlines()

        self.assertIn("synced label: ai-ready", proc.stdout)
        self.assertIn("synced label: tests-needed", proc.stdout)
        self.assertEqual(2, len(calls))
        self.assertIn("label create ai-ready --color 0e8a16", calls[0])


if __name__ == "__main__":
    unittest.main()
