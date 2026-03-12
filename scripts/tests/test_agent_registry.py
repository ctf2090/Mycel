import json
import shutil
import subprocess
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_registry.py"
TAIPEI_TZ = timezone(timedelta(hours=8))


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

    def timestamp(self, dt: datetime) -> str:
        return dt.astimezone(TAIPEI_TZ).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")

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
                        "assigned_at": "2026-03-12T00:00:00+0800",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:00+0800",
                        "last_touched_at": "2026-03-12T00:05:00+0800",
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
                "updated_at": "2026-03-12T00:00:00+0800",
                "agent_count": 2,
                "agents": [
                    {
                        "id": "coding-1",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": "2026-03-12T00:00:00+0800",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:00+0800",
                        "last_touched_at": "2026-03-12T00:05:00+0800",
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
                        "assigned_at": "2026-03-12T00:00:01+0800",
                        "confirmed_by_agent": True,
                        "confirmed_at": "2026-03-12T00:00:01+0800",
                        "last_touched_at": "2026-03-12T00:06:00+0800",
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
        self.assertTrue(finish["inactive_at"].endswith("+0800"))

        status_after_finish = json.loads(self.run_cli("status", agent_id, "--json").stdout)
        self.assertEqual("inactive", status_after_finish["agents"][0]["status"])
        self.assertIsNotNone(status_after_finish["agents"][0]["inactive_at"])

        touched = json.loads(self.run_cli("touch", agent_id, "--json").stdout)
        self.assertEqual("active", touched["current_status"])
        self.assertIsNotNone(touched["last_touched_at"])
        self.assertTrue(touched["last_touched_at"].endswith("+0800"))

        status_after_touch = json.loads(self.run_cli("status", agent_id, "--json").stdout)
        self.assertEqual("active", status_after_touch["agents"][0]["status"])
        self.assertIsNone(status_after_touch["agents"][0]["inactive_at"])

    def test_cleanup_prunes_entries_inactive_for_at_least_one_hour(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        self.write_registry(
            {
                "version": 1,
                "updated_at": self.timestamp(now),
                "agent_count": 2,
                "agents": [
                    {
                        "id": "doc-9",
                        "role": "doc",
                        "assigned_by": "user",
                        "assigned_at": self.timestamp(now - timedelta(hours=2)),
                        "confirmed_by_agent": True,
                        "confirmed_at": self.timestamp(now - timedelta(hours=2)),
                        "last_touched_at": self.timestamp(now - timedelta(hours=2, minutes=5)),
                        "inactive_at": self.timestamp(now - timedelta(hours=2, minutes=10)),
                        "status": "inactive",
                        "scope": "old-task",
                        "files": [],
                        "mailbox": ".agent-local/doc-9.md",
                    },
                    {
                        "id": "coding-3",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": self.timestamp(now - timedelta(minutes=20)),
                        "confirmed_by_agent": True,
                        "confirmed_at": self.timestamp(now - timedelta(minutes=20)),
                        "last_touched_at": self.timestamp(now - timedelta(minutes=15)),
                        "inactive_at": self.timestamp(now - timedelta(minutes=10)),
                        "status": "inactive",
                        "scope": "recent-task",
                        "files": [],
                        "mailbox": ".agent-local/coding-3.md",
                    },
                ],
            }
        )

        cleanup = json.loads(self.run_cli("cleanup", "--json").stdout)
        self.assertEqual(1, cleanup["stale_count"])
        self.assertEqual(["doc-9"], cleanup["stale_ids"])

        status = json.loads(self.run_cli("status", "--json").stdout)
        self.assertEqual(2, status["agent_count"])
        self.assertEqual(["doc-9"], status["stale_inactive_ids"])
        self.assertEqual(["doc-9", "coding-3"], [entry["id"] for entry in status["agents"]])

    def test_resume_check_allows_inactive_confirmed_agents_to_resume(self) -> None:
        claim = json.loads(self.run_cli("claim", "coding", "--scope", "lease-test", "--json").stdout)
        agent_id = claim["agent_id"]
        self.run_cli("start", agent_id, "--json")
        self.run_cli("finish", agent_id, "--json")

        resume = self.run_cli("resume-check", agent_id, "--json", check=False)

        self.assertEqual(0, resume.returncode)
        payload = json.loads(resume.stdout)
        self.assertTrue(payload["safe_to_resume"])
        self.assertIn("inactive", payload["reason"])

    def test_stale_inactive_id_is_not_reused_by_a_new_claim(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        self.write_registry(
            {
                "version": 1,
                "updated_at": self.timestamp(now),
                "agent_count": 1,
                "agents": [
                    {
                        "id": "coding-1",
                        "role": "coding",
                        "assigned_by": "user",
                        "assigned_at": self.timestamp(now - timedelta(hours=2)),
                        "confirmed_by_agent": True,
                        "confirmed_at": self.timestamp(now - timedelta(hours=2)),
                        "last_touched_at": self.timestamp(now - timedelta(hours=2, minutes=5)),
                        "inactive_at": self.timestamp(now - timedelta(hours=2, minutes=10)),
                        "status": "inactive",
                        "scope": "old-work",
                        "files": [],
                        "mailbox": ".agent-local/coding-1.md",
                    }
                ],
            }
        )

        claim = json.loads(self.run_cli("claim", "auto", "--scope", "new-chat", "--json").stdout)
        resume = json.loads(self.run_cli("resume-check", "coding-1", "--json").stdout)

        self.assertEqual("coding-2", claim["agent_id"])
        self.assertEqual("coding", claim["role"])
        self.assertTrue(resume["safe_to_resume"])
        self.assertIn("inactive", resume["reason"])


if __name__ == "__main__":
    unittest.main()
