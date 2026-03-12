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

    def read_registry(self) -> dict:
        registry_path = self.root / ".agent-local" / "agents.json"
        return json.loads(registry_path.read_text(encoding="utf-8"))

    def write_registry(self, payload: dict) -> None:
        registry_path = self.root / ".agent-local" / "agents.json"
        registry_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    def timestamp(self, dt: datetime) -> str:
        return dt.astimezone(TAIPEI_TZ).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")

    def make_v2_entry(
        self,
        *,
        agent_uid: str,
        role: str,
        display_id: str | None,
        assigned_at: str,
        status: str,
        scope: str,
        confirmed: bool = True,
        confirmed_at: str | None = None,
        last_touched_at: str | None = None,
        inactive_at: str | None = None,
        paused_at: str | None = None,
        recovery_of: str | None = None,
        superseded_by: str | None = None,
    ) -> dict:
        history = []
        if display_id is not None:
            history.append(
                {
                    "display_id": display_id,
                    "assigned_at": assigned_at,
                    "released_at": None,
                    "released_reason": None,
                }
            )
        return {
            "agent_uid": agent_uid,
            "role": role,
            "current_display_id": display_id,
            "display_history": history,
            "assigned_by": "user",
            "assigned_at": assigned_at,
            "confirmed_by_agent": confirmed,
            "confirmed_at": confirmed_at or assigned_at,
            "last_touched_at": last_touched_at,
            "inactive_at": inactive_at,
            "paused_at": paused_at,
            "status": status,
            "scope": scope,
            "files": [],
            "mailbox": f".agent-local/mailboxes/{agent_uid}.md",
            "recovery_of": recovery_of,
            "superseded_by": superseded_by,
        }

    def test_status_migrates_v1_registry_to_v2(self) -> None:
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

        status = json.loads(self.run_cli("status", "--json").stdout)
        registry = self.read_registry()
        entry = registry["agents"][0]

        self.assertEqual(2, registry["version"])
        self.assertTrue(entry["agent_uid"].startswith("agt_"))
        self.assertEqual("coding-1", entry["current_display_id"])
        self.assertEqual("coding-1", status["agents"][0]["display_id"])
        self.assertTrue(status["agents"][0]["mailbox"].startswith(".agent-local/mailboxes/"))
        self.assertTrue((self.root / status["agents"][0]["mailbox"]).exists())

    def test_claim_outputs_uid_and_display_id_with_auto_role(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T00:00:00+0800",
                "agent_count": 2,
                "agents": [
                    self.make_v2_entry(
                        agent_uid="agt_coding_a",
                        role="coding",
                        display_id="coding-1",
                        assigned_at="2026-03-12T00:00:00+0800",
                        status="active",
                        scope="pending-user-task",
                        last_touched_at="2026-03-12T00:05:00+0800",
                    ),
                    self.make_v2_entry(
                        agent_uid="agt_doc_a",
                        role="doc",
                        display_id="doc-1",
                        assigned_at="2026-03-12T00:00:10+0800",
                        status="active",
                        scope="pending-user-task",
                        last_touched_at="2026-03-12T00:06:00+0800",
                    ),
                ],
            }
        )

        claim = json.loads(self.run_cli("claim", "auto", "--scope", "lease-test", "--json").stdout)

        self.assertEqual("coding", claim["role"])
        self.assertEqual("coding-2", claim["display_id"])
        self.assertTrue(claim["agent_uid"].startswith("agt_"))
        self.assertTrue(claim["mailbox"].startswith(".agent-local/mailboxes/"))

    def test_touch_and_finish_accept_agent_uid(self) -> None:
        claim = json.loads(self.run_cli("claim", "coding", "--scope", "lease-test", "--json").stdout)
        agent_uid = claim["agent_uid"]
        display_id = claim["display_id"]

        start = json.loads(self.run_cli("start", agent_uid, "--json").stdout)
        self.assertEqual(agent_uid, start["agent_uid"])
        self.assertEqual(display_id, start["display_id"])

        finish = json.loads(self.run_cli("finish", agent_uid, "--json").stdout)
        self.assertEqual("inactive", finish["current_status"])
        self.assertEqual(display_id, finish["display_id"])

        touched = json.loads(self.run_cli("touch", display_id, "--json").stdout)
        self.assertEqual(agent_uid, touched["agent_uid"])
        self.assertEqual("active", touched["current_status"])

    def test_stale_slot_is_released_and_old_agent_must_recover(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 1,
                "agents": [
                    self.make_v2_entry(
                        agent_uid="agt_old",
                        role="coding",
                        display_id="coding-1",
                        assigned_at=self.timestamp(now - timedelta(hours=3)),
                        status="inactive",
                        scope="old-work",
                        last_touched_at=self.timestamp(now - timedelta(hours=2, minutes=55)),
                        inactive_at=self.timestamp(now - timedelta(hours=2, minutes=10)),
                    )
                ],
            }
        )

        claim = json.loads(self.run_cli("claim", "auto", "--scope", "new-chat", "--json").stdout)
        resume = self.run_cli("resume-check", "agt_old", "--json", check=False)
        registry = self.read_registry()
        old_entry = next(entry for entry in registry["agents"] if entry["agent_uid"] == "agt_old")

        self.assertEqual("coding", claim["role"])
        self.assertEqual("coding-1", claim["display_id"])
        self.assertIsNone(old_entry["current_display_id"])
        resume_payload = json.loads(resume.stdout)
        self.assertEqual(2, resume.returncode)
        self.assertFalse(resume_payload["safe_to_resume"])
        self.assertTrue(resume_payload["must_recover"])
        self.assertEqual("recover", resume_payload["recommended_action"])

    def test_recover_reuses_same_agent_uid_with_new_display_id(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        stale_entry = self.make_v2_entry(
            agent_uid="agt_old",
            role="coding",
            display_id=None,
            assigned_at=self.timestamp(now - timedelta(hours=3)),
            status="inactive",
            scope="old-work",
            last_touched_at=self.timestamp(now - timedelta(hours=2, minutes=55)),
            inactive_at=self.timestamp(now - timedelta(hours=2, minutes=10)),
        )
        stale_entry["display_history"] = [
            {
                "display_id": "coding-1",
                "assigned_at": self.timestamp(now - timedelta(hours=3)),
                "released_at": self.timestamp(now - timedelta(hours=2)),
                "released_reason": "stale-recycled",
            }
        ]
        active_entry = self.make_v2_entry(
            agent_uid="agt_active",
            role="coding",
            display_id="coding-1",
            assigned_at=self.timestamp(now - timedelta(minutes=30)),
            status="active",
            scope="new-work",
            last_touched_at=self.timestamp(now - timedelta(minutes=5)),
        )
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 2,
                "agents": [stale_entry, active_entry],
            }
        )

        recover = json.loads(self.run_cli("recover", "agt_old", "--json").stdout)
        registry = self.read_registry()
        old_entry = next(entry for entry in registry["agents"] if entry["agent_uid"] == "agt_old")

        self.assertEqual("agt_old", recover["agent_uid"])
        self.assertEqual("coding-1", recover["previous_display_id"])
        self.assertEqual("coding-2", recover["recovered_display_id"])
        self.assertEqual("coding-2", old_entry["current_display_id"])
        self.assertEqual("active", old_entry["status"])
        self.assertEqual(2, len(old_entry["display_history"]))

    def test_takeover_creates_new_agent_and_links_old_one(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        stale_entry = self.make_v2_entry(
            agent_uid="agt_old",
            role="coding",
            display_id=None,
            assigned_at=self.timestamp(now - timedelta(hours=3)),
            status="inactive",
            scope="handoff-work",
            last_touched_at=self.timestamp(now - timedelta(hours=2, minutes=55)),
            inactive_at=self.timestamp(now - timedelta(hours=2, minutes=10)),
        )
        stale_entry["display_history"] = [
            {
                "display_id": "coding-1",
                "assigned_at": self.timestamp(now - timedelta(hours=3)),
                "released_at": self.timestamp(now - timedelta(hours=2)),
                "released_reason": "stale-recycled",
            }
        ]
        doc_entry = self.make_v2_entry(
            agent_uid="agt_doc",
            role="doc",
            display_id="doc-1",
            assigned_at=self.timestamp(now - timedelta(minutes=15)),
            status="active",
            scope="docs",
            last_touched_at=self.timestamp(now - timedelta(minutes=5)),
        )
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 2,
                "agents": [stale_entry, doc_entry],
            }
        )

        takeover = json.loads(self.run_cli("takeover", "agt_old", "--json").stdout)
        registry = self.read_registry()
        old_entry = next(entry for entry in registry["agents"] if entry["agent_uid"] == "agt_old")
        new_entry = next(entry for entry in registry["agents"] if entry["agent_uid"] == takeover["replacement_agent_uid"])

        self.assertEqual("agt_old", takeover["stale_agent_uid"])
        self.assertTrue(takeover["replacement_agent_uid"].startswith("agt_"))
        self.assertEqual("coding-1", takeover["replacement_display_id"])
        self.assertEqual(takeover["replacement_agent_uid"], old_entry["superseded_by"])
        self.assertEqual("agt_old", new_entry["recovery_of"])
        self.assertEqual("active", new_entry["status"])
        self.assertEqual("paused", old_entry["status"])

    def test_cleanup_removes_entries_stale_for_24_hours(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        old_entry = self.make_v2_entry(
            agent_uid="agt_old",
            role="doc",
            display_id=None,
            assigned_at=self.timestamp(now - timedelta(hours=30)),
            status="inactive",
            scope="expired-task",
            last_touched_at=self.timestamp(now - timedelta(hours=29, minutes=55)),
            inactive_at=self.timestamp(now - timedelta(hours=25, minutes=5)),
        )
        old_entry["display_history"] = [
            {
                "display_id": "doc-1",
                "assigned_at": self.timestamp(now - timedelta(hours=30)),
                "released_at": self.timestamp(now - timedelta(hours=24)),
                "released_reason": "stale-recycled",
            }
        ]
        retained_entry = self.make_v2_entry(
            agent_uid="agt_recent",
            role="coding",
            display_id=None,
            assigned_at=self.timestamp(now - timedelta(hours=3)),
            status="inactive",
            scope="still-stale",
            last_touched_at=self.timestamp(now - timedelta(hours=2, minutes=55)),
            inactive_at=self.timestamp(now - timedelta(hours=2, minutes=10)),
        )
        retained_entry["display_history"] = [
            {
                "display_id": "coding-1",
                "assigned_at": self.timestamp(now - timedelta(hours=3)),
                "released_at": self.timestamp(now - timedelta(hours=1)),
                "released_reason": "stale-recycled",
            }
        ]
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 2,
                "agents": [old_entry, retained_entry],
            }
        )

        cleanup = json.loads(self.run_cli("cleanup", "--json").stdout)
        status = json.loads(self.run_cli("status", "--json").stdout)

        self.assertEqual(1, cleanup["removed_count"])
        self.assertEqual("agt_old", cleanup["removed_agents"][0]["agent_uid"])
        self.assertEqual(1, cleanup["stale_count"])
        self.assertEqual("agt_recent", cleanup["stale_agents"][0]["agent_uid"])
        self.assertEqual(1, status["agent_count"])
        self.assertEqual("agt_recent", status["agents"][0]["agent_uid"])

    def test_cleanup_releases_and_removes_paused_entries_after_paused_retention(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        old_entry = self.make_v2_entry(
            agent_uid="agt_paused_old",
            role="coding",
            display_id="coding-1",
            assigned_at=self.timestamp(now - timedelta(days=16)),
            status="paused",
            scope="paused-old",
            last_touched_at=self.timestamp(now - timedelta(days=16, minutes=5)),
            paused_at=self.timestamp(now - timedelta(days=15)),
        )
        recent_entry = self.make_v2_entry(
            agent_uid="agt_paused_recent",
            role="coding",
            display_id="coding-2",
            assigned_at=self.timestamp(now - timedelta(days=9)),
            status="paused",
            scope="paused-recent",
            last_touched_at=self.timestamp(now - timedelta(days=9, minutes=5)),
            paused_at=self.timestamp(now - timedelta(days=8)),
        )
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 2,
                "agents": [old_entry, recent_entry],
            }
        )

        cleanup = json.loads(self.run_cli("cleanup", "--json").stdout)
        status = json.loads(self.run_cli("status", "--json").stdout)
        retained_entry = status["agents"][0]

        self.assertEqual(1, cleanup["removed_count"])
        self.assertEqual("agt_paused_old", cleanup["removed_agents"][0]["agent_uid"])
        self.assertEqual(1, cleanup["stale_count"])
        self.assertEqual("agt_paused_recent", cleanup["stale_agents"][0]["agent_uid"])
        self.assertEqual(1, status["agent_count"])
        self.assertEqual("agt_paused_recent", retained_entry["agent_uid"])
        self.assertIsNone(retained_entry["display_id"])

    def test_cleanup_infers_paused_at_for_legacy_paused_entries(self) -> None:
        now = datetime.now(TAIPEI_TZ).replace(microsecond=0)
        self.write_registry(
            {
                "version": 2,
                "updated_at": self.timestamp(now),
                "agent_count": 1,
                "agents": [
                    {
                        **self.make_v2_entry(
                            agent_uid="agt_paused_legacy",
                            role="doc",
                            display_id="doc-1",
                            assigned_at=self.timestamp(now - timedelta(days=10)),
                            status="paused",
                            scope="legacy-paused",
                            last_touched_at=self.timestamp(now - timedelta(days=8)),
                        ),
                        "paused_at": None,
                    }
                ],
            }
        )

        cleanup = json.loads(self.run_cli("cleanup", "--json").stdout)
        registry = self.read_registry()
        entry = registry["agents"][0]

        self.assertEqual(1, cleanup["stale_count"])
        self.assertEqual("agt_paused_legacy", cleanup["stale_agents"][0]["agent_uid"])
        self.assertIsNotNone(entry["paused_at"])
        self.assertIsNone(entry["current_display_id"])


if __name__ == "__main__":
    unittest.main()
