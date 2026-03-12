import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_WORK_CYCLE = REPO_ROOT / "scripts" / "agent_work_cycle.py"
SOURCE_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_TIMESTAMP = REPO_ROOT / "scripts" / "agent_timestamp.py"


class AgentWorkCycleCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_WORK_CYCLE, self.root / "scripts" / "agent_work_cycle.py")
        shutil.copy2(SOURCE_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_TIMESTAMP, self.root / "scripts" / "agent_timestamp.py")
        (self.root / "scripts" / "agent_work_cycle.py").chmod(0o755)
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)
        (self.root / "scripts" / "agent_timestamp.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_work_cycle.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def run_registry(self, *args: str) -> dict:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_registry.py"), *args, "--json"],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=True,
        )
        return json.loads(proc.stdout)

    def test_begin_touches_agent_and_prints_before_work_line(self) -> None:
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")

        self.assertIn(f"agent_uid: {agent_uid}", proc.stdout)
        self.assertIn("current_status: active", proc.stdout)
        self.assertIn("Before work | doc-1 | timestamp-wrapper", proc.stdout)

    def test_end_finishes_agent_and_prints_after_work_line(self) -> None:
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_registry("touch", agent_uid)

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper")

        self.assertIn(f"agent_uid: {agent_uid}", proc.stdout)
        self.assertIn("current_status: inactive", proc.stdout)
        self.assertIn("After work | doc-1 | timestamp-wrapper", proc.stdout)


if __name__ == "__main__":
    unittest.main()
