import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_registry.py"


class AgentRegistryCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "agent_registry.py")
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_registry.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_registry(self, payload: dict) -> None:
        registry_path = self.root / ".agent-local" / "agents.json"
        registry_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    def test_auto_role_selection_prefers_doc_only_when_coding_is_already_active(self) -> None:
        self.write_registry(
            {
                "version": 1,
                "updated_at": "2026-03-12T00:00:00Z",
                "agent_count": 1,
                "agents": [
                    {
                        "id": "coding-1",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T00:00:00Z",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:00Z",
                        "last_touched_at": "2026-03-12T00:05:00Z",
                        "inactive_at": None,
                        "status": "active",
                        "scope": "pending-user-task",
                        "files": [],
                        "mailbox": ".agent-local/coding-1.md",
                    }
                ],
            }
        )

        claim = json.loads(self.run_cli("claim", "auto", "--scope", "lease-test", "--json").stdout)

        self.assertEqual("doc", claim["role"])
        self.assertEqual("doc-1", claim["agent_id"])

    def test_auto_role_selection_returns_to_coding_when_both_roles_are_active(self) -> None:
        self.write_registry(
            {
                "version": 1,
                "updated_at": "2026-03-12T00:00:00Z",
                "agent_count": 2,
                "agents": [
                    {
                        "id": "coding-1",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T00:00:00Z",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:00Z",
                        "last_touched_at": "2026-03-12T00:05:00Z",
                        "inactive_at": None,
                        "status": "active",
                        "scope": "pending-user-task",
                        "files": [],
                        "mailbox": ".agent-local/coding-1.md",
                    },
                    {
                        "id": "doc-1",
                        "role": "doc",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T00:00:01Z",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:01Z",
                        "last_touched_at": "2026-03-12T00:06:00Z",
                        "inactive_at": None,
                        "status": "active",
                        "scope": "pending-user-task",
                        "files": [],
                        "mailbox": ".agent-local/doc-1.md",
                    },
                ],
            }
        )

        claim = json.loads(self.run_cli("claim", "auto", "--scope", "lease-test", "--json").stdout)

        self.assertEqual("coding", claim["role"])
        self.assertEqual("coding-2", claim["agent_id"])

    def test_touch_and_finish_drive_the_activity_lease(self) -> None:
        claim = json.loads(self.run_cli("claim", "coding", "--scope", "lease-test", "--json").stdout)
        agent_id = claim["agent_id"]

        start = json.loads(self.run_cli("start", agent_id, "--json").stdout)
        self.assertEqual(agent_id, start["agent_id"])

        finish = json.loads(self.run_cli("finish", agent_id, "--json").stdout)
        self.assertEqual("inactive", finish["current_status"])
        self.assertIsNotNone(finish["inactive_at"])

        status_after_finish = json.loads(self.run_cli("status", agent_id, "--json").stdout)
        self.assertEqual("inactive", status_after_finish["agents"][0]["status"])
        self.assertIsNotNone(status_after_finish["agents"][0]["inactive_at"])

        touched = json.loads(self.run_cli("touch", agent_id, "--json").stdout)
        self.assertEqual("active", touched["current_status"])
        self.assertIsNotNone(touched["last_touched_at"])

        status_after_touch = json.loads(self.run_cli("status", agent_id, "--json").stdout)
        self.assertEqual("active", status_after_touch["agents"][0]["status"])
        self.assertIsNone(status_after_touch["agents"][0]["inactive_at"])

    def test_cleanup_prunes_entries_inactive_for_at_least_one_hour(self) -> None:
        self.write_registry(
            {
                "version": 1,
                "updated_at": "2026-03-12T00:00:00Z",
                "agent_count": 2,
                "agents": [
                    {
                        "id": "doc-9",
                        "role": "doc",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T00:00:00Z",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:00Z",
                        "last_touched_at": "2026-03-12T00:05:00Z",
                        "inactive_at": "2026-03-12T00:10:00Z",
                        "status": "inactive",
                        "scope": "old-task",
                        "files": [],
                        "mailbox": ".agent-local/doc-9.md",
                    },
                    {
                        "id": "coding-3",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T01:30:00Z",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T01:30:00Z",
                        "last_touched_at": "2026-03-12T01:35:00Z",
                        "inactive_at": "2026-03-12T01:40:00Z",
                        "status": "inactive",
                        "scope": "recent-task",
                        "files": [],
                        "mailbox": ".agent-local/coding-3.md",
                    },
                ],
            }
        )

        cleanup = json.loads(self.run_cli("cleanup", "--json").stdout)
        self.assertEqual(1, cleanup["removed_count"])
        self.assertEqual(["doc-9"], cleanup["removed_ids"])

        status = json.loads(self.run_cli("status", "--json").stdout)
        self.assertEqual(1, status["agent_count"])
        self.assertEqual("coding-3", status["agents"][0]["id"])

    def test_resume_check_refuses_inactive_agents(self) -> None:
        claim = json.loads(self.run_cli("claim", "coding", "--scope", "lease-test", "--json").stdout)
        agent_id = claim["agent_id"]
        self.run_cli("start", agent_id, "--json")
        self.run_cli("finish", agent_id, "--json")

        resume = self.run_cli("resume-check", agent_id, "--json", check=False)

        self.assertEqual(2, resume.returncode)
        payload = json.loads(resume.stdout)
        self.assertFalse(payload["safe_to_resume"])
        self.assertIn("inactive", payload["reason"])


if __name__ == "__main__":
    unittest.main()
