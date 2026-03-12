import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "inactive_coding_handoffs.py"


class InactiveCodingHandoffsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "mailboxes").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "inactive_coding_handoffs.py")
        (self.root / "scripts" / "inactive_coding_handoffs.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "inactive_coding_handoffs.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_registry(self, payload: dict) -> None:
        registry_path = self.root / ".agent-local" / "agents.json"
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        registry_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    def write_mailbox(self, relative_path: str, content: str) -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def registry_entry(
        self,
        agent_uid: str,
        *,
        display_id: str,
        status: str = "inactive",
        role: str = "coding",
        mailbox: str | None = None,
    ) -> dict:
        return {
            "agent_uid": agent_uid,
            "role": role,
            "current_display_id": None if status == "inactive" else display_id,
            "display_history": [
                {
                    "display_id": display_id,
                    "assigned_at": "2026-03-12T22:00:00+0800",
                    "released_at": "2026-03-12T23:00:00+0800" if status == "inactive" else None,
                    "released_reason": "finished" if status == "inactive" else None,
                }
            ],
            "assigned_by": "user",
            "assigned_at": "2026-03-12T22:00:00+0800",
            "confirmed_by_agent": True,
            "confirmed_at": "2026-03-12T22:01:00+0800",
            "last_touched_at": "2026-03-12T23:00:00+0800",
            "inactive_at": "2026-03-12T23:00:00+0800" if status == "inactive" else None,
            "paused_at": None,
            "status": status,
            "scope": "test-scope",
            "files": [],
            "mailbox": mailbox or f".agent-local/mailboxes/{agent_uid}.md",
            "recovery_of": None,
            "superseded_by": None,
        }

    def test_json_scan_reports_latest_open_handoff_for_each_inactive_coding_agent(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T23:30:00+0800",
                "agent_count": 3,
                "agents": [
                    self.registry_entry("agt_one", display_id="coding-1"),
                    self.registry_entry("agt_two", display_id="coding-2"),
                    self.registry_entry("agt_doc", display_id="doc-1", role="doc"),
                ],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_one.md",
            """# Mailbox for agt_one

## Work Continuation Handoff

- Status: open
- Date: 2026-03-12 22:10 UTC+8
- Source agent: coding-1
- Scope: older-scope
- Next suggested step:
  - older next step

## Work Continuation Handoff

- Status: open
- Date: 2026-03-12 22:50 UTC+8
- Source agent: coding-1
- Scope: newer-scope
- Next suggested step:
  - newer next step
""",
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_two.md",
            """# Mailbox for agt_two

## Work Continuation Handoff

- Status: superseded
- Date: 2026-03-12 22:30 UTC+8
- Source agent: coding-2
- Scope: old

## Work Continuation Handoff

- Status: open
- Date: 2026-03-12 22:40 UTC+8
- Source agent: coding-2
- Scope: current
- Next suggested step:
  - ship the current slice
""",
        )

        payload = json.loads(self.run_cli("--json").stdout)

        self.assertEqual(2, payload["inactive_coding_count"])
        self.assertEqual(2, payload["with_open_handoff_count"])
        self.assertEqual(0, payload["missing_mailbox_count"])
        self.assertEqual(0, payload["without_open_handoff_count"])
        first = next(entry for entry in payload["handoffs"] if entry["agent_uid"] == "agt_one")
        self.assertEqual("newer-scope", first["handoff"]["scope"])
        self.assertEqual(["newer next step"], first["handoff"]["next_suggested_step"])

    def test_scan_reports_missing_mailbox_and_agents_without_open_handoff(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T23:30:00+0800",
                "agent_count": 2,
                "agents": [
                    self.registry_entry("agt_missing", display_id="coding-4"),
                    self.registry_entry("agt_closed", display_id="coding-5"),
                ],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_closed.md",
            """# Mailbox for agt_closed

## Work Continuation Handoff

- Status: superseded
- Date: 2026-03-12 22:40 UTC+8
- Source agent: coding-5
- Scope: closed-scope
""",
        )

        payload = json.loads(self.run_cli("--json").stdout)

        self.assertEqual(1, payload["missing_mailbox_count"])
        self.assertEqual(1, payload["without_open_handoff_count"])
        self.assertEqual("agt_missing", payload["missing_mailboxes"][0]["agent_uid"])
        self.assertEqual("agt_closed", payload["without_open_handoff"][0]["agent_uid"])

    def test_human_output_includes_summary_and_next_step(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T23:30:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry("agt_human", display_id="coding-8")],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_human.md",
            """# Mailbox for agt_human

## Work Continuation Handoff

- Status: open
- Date: 2026-03-12 22:55 UTC+8
- Source agent: coding-8
- Scope: human-scope
- Next suggested step:
  - continue from the handoff
""",
        )

        proc = self.run_cli()

        self.assertIn("inactive_coding_agents: 1", proc.stdout)
        self.assertIn("open_handoffs:", proc.stdout)
        self.assertIn("continue from the handoff", proc.stdout)


if __name__ == "__main__":
    unittest.main()
