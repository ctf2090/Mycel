import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_GUARD = REPO_ROOT / "scripts" / "agent_guard.py"
SOURCE_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentGuardCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_GUARD, scripts_dir / "agent_guard.py")
        shutil.copy2(SOURCE_REGISTRY, scripts_dir / "agent_registry.py")
        shutil.copy2(SOURCE_CHECKLIST, scripts_dir / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, scripts_dir / "item_id_checklist_mark.py")
        for script_name in [
            "agent_guard.py",
            "agent_registry.py",
            "item_id_checklist.py",
            "item_id_checklist_mark.py",
        ]:
            (scripts_dir / script_name).chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "agent_guard.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=False,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def run_registry(self, *args: str) -> dict:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "agent_registry.py"), *args, "--json"],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=True,
        )
        return json.loads(proc.stdout)

    def test_check_reports_allowed_when_no_block_exists(self) -> None:
        claim = self.run_registry("claim", "coding", "--scope", "guard-test", "--model-id", "gpt-5.4")

        proc = self.run_cli("check", claim["agent_uid"])

        self.assertEqual(0, proc.returncode)
        self.assertIn("agent allowed:", proc.stdout)

    def test_block_then_check_returns_blocked(self) -> None:
        claim = self.run_registry("claim", "coding", "--scope", "guard-test", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]

        block = self.run_cli(
            "block",
            agent_uid,
            "--reason",
            "compact_context_detected",
            "--detected-at",
            "2026-03-25T15:28:43.925Z",
            "--source",
            "agent_work_cycle.begin",
            "--scope",
            "guard-test",
            "--handoff-path",
            ".agent-local/mailboxes/test.md",
            "--rollout-path",
            "/tmp/rollout.jsonl",
            "--json",
            check=False,
        )
        self.assertEqual(10, block.returncode)
        payload = json.loads(block.stdout)
        self.assertTrue(payload["blocked"])
        self.assertEqual("compact_context_detected", payload["block"]["reason"])

        check = self.run_cli("check", agent_uid, check=False)
        self.assertEqual(10, check.returncode)
        self.assertIn("agent execution blocked", check.stdout)
        self.assertIn(".agent-local/mailboxes/test.md", check.stdout)

    def test_status_lists_blocked_agents(self) -> None:
        claim = self.run_registry("claim", "coding", "--scope", "guard-test", "--model-id", "gpt-5.4")

        proc = self.run_cli(
            "block",
            claim["agent_uid"],
            "--reason",
            "compact_context_detected",
            "--detected-at",
            "2026-03-25T15:28:43.925Z",
            "--source",
            "agent_work_cycle.begin",
            check=False,
        )
        self.assertEqual(10, proc.returncode)

        proc = self.run_cli("status", "--json")
        payload = json.loads(proc.stdout)
        self.assertIn(claim["agent_uid"], payload["blocked_agents"])

    def test_corrupt_state_fails_closed(self) -> None:
        runtime_dir = self.root / ".agent-local" / "runtime"
        runtime_dir.mkdir(parents=True, exist_ok=True)
        (runtime_dir / "agent-blocks.json").write_text("{not-json\n", encoding="utf-8")
        claim = self.run_registry("claim", "coding", "--scope", "guard-test", "--model-id", "gpt-5.4")

        proc = self.run_cli("check", claim["agent_uid"], check=False)

        self.assertEqual(12, proc.returncode)
        self.assertIn("invalid guard state JSON", proc.stderr)


if __name__ == "__main__":
    unittest.main()
