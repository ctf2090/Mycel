import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_timestamp.py"


class AgentTimestampCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "agent_timestamp.py")
        (self.root / "scripts" / "agent_timestamp.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_timestamp.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_before_message_includes_agent_uid_and_scope(self) -> None:
        proc = self.run_cli(
            "before",
            "--agent",
            "doc-3",
            "--agent-uid",
            "agt_1234abcd",
            "--scope",
            "docs sync",
            "--now",
            "2026-03-12T06:15:30Z",
        )

        self.assertEqual(
            "[2026-03-12 14:15:30 UTC+8] Before work | doc-3 (agt_1234abcd) | docs sync",
            proc.stdout.strip(),
        )

    def test_after_message_without_optional_fields(self) -> None:
        proc = self.run_cli("after", "--now", "2026-03-12T06:15:30Z")

        self.assertEqual("[2026-03-12 14:15:30 UTC+8] After work", proc.stdout.strip())


if __name__ == "__main__":
    unittest.main()
