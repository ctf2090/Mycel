import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_MAILBOX_HANDOFF = REPO_ROOT / "scripts" / "mailbox_handoff.py"
SOURCE_AGENT_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_ITEM_ID_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_ITEM_ID_CHECKLIST_MARK = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class MailboxHandoffCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "mailboxes").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_MAILBOX_HANDOFF, self.root / "scripts" / "mailbox_handoff.py")
        shutil.copy2(SOURCE_AGENT_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_ITEM_ID_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_ITEM_ID_CHECKLIST_MARK, self.root / "scripts" / "item_id_checklist_mark.py")
        (self.root / "scripts" / "mailbox_handoff.py").chmod(0o755)
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist_mark.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "mailbox_handoff.py"), *args],
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

    def registry_entry(self, *, agent_uid: str, role: str, display_id: str) -> dict:
        return {
            "agent_uid": agent_uid,
            "role": role,
            "current_display_id": display_id,
            "display_history": [
                {
                    "display_id": display_id,
                    "assigned_at": "2026-03-13T10:00:00+0800",
                    "released_at": None,
                    "released_reason": None,
                }
            ],
            "assigned_by": "user",
            "assigned_at": "2026-03-13T10:00:00+0800",
            "confirmed_by_agent": True,
            "confirmed_at": "2026-03-13T10:00:00+0800",
            "last_touched_at": "2026-03-13T10:05:00+0800",
            "inactive_at": None,
            "paused_at": None,
            "status": "active",
            "scope": "mailbox-handoff-test",
            "files": [],
            "mailbox": f".agent-local/mailboxes/{agent_uid}.md",
            "recovery_of": None,
            "superseded_by": None,
        }

    def test_create_work_continuation_supersedes_existing_open_entry(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_coding", role="coding", display_id="coding-7")],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_coding.md",
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
- Date: 2026-03-13 09:00 UTC+8
- Source agent: coding-7
- Scope: older-scope
- Current state:
  - older state
- Next suggested step:
  - older next step
""",
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "coding-7",
                "work-continuation",
                "--scope",
                "new-scope",
                "--behavior-change",
                "new behavior landed",
                "--current-state",
                "new state",
                "--next-step",
                "new next step",
                "--verification",
                "cargo test -p mycel-cli",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual("work-continuation", payload["template"])
        self.assertEqual(1, payload["superseded_count"])
        self.assertIn("## Work Continuation Handoff", mailbox)
        self.assertIn("- Scope: new-scope", mailbox)
        self.assertIn("  - new state", mailbox)
        self.assertEqual(1, mailbox.count("- Status: open"))
        self.assertEqual(1, mailbox.count("- Status: superseded"))
        self.assertLess(mailbox.index("- Scope: older-scope"), mailbox.index("- Scope: new-scope"))

    def test_create_doc_continuation_uses_doc_template(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_doc", role="doc", display_id="doc-9")],
            }
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "agt_doc",
                "doc-continuation",
                "--scope",
                "planning-sync-batch",
                "--current-state",
                "refresh is due for doc, issue, and web",
                "--next-step",
                "start from docs/PLANNING-SYNC-PLAN.md",
                "--evidence",
                "scripts/check-plan-refresh.sh",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual("Doc Continuation Note", payload["entry_heading"])
        self.assertEqual("open", payload["status"])
        self.assertIn("## Doc Continuation Note", mailbox)
        self.assertIn("- Source agent: doc-9", mailbox)
        self.assertIn("  - refresh is due for doc, issue, and web", mailbox)
        self.assertIn("  - scripts/check-plan-refresh.sh", mailbox)

    def test_create_delivery_continuation_uses_delivery_template(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_delivery", role="delivery", display_id="delivery-2")],
            }
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "agt_delivery",
                "delivery-continuation",
                "--scope",
                "ci-flake-triage",
                "--current-state",
                "latest completed CI is failing in pages lint",
                "--next-step",
                "reproduce the failing job locally",
                "--evidence",
                "gh run view 123 --log-failed",
                "--blockers",
                "awaiting log retention confirmation",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual("Delivery Continuation Note", payload["entry_heading"])
        self.assertEqual("open", payload["status"])
        self.assertIn("## Delivery Continuation Note", mailbox)
        self.assertIn("- Source agent: delivery-2", mailbox)
        self.assertIn("  - latest completed CI is failing in pages lint", mailbox)
        self.assertIn("  - awaiting log retention confirmation", mailbox)

    def test_create_planning_sync_keeps_open_same_role_handoff(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_coding", role="coding", display_id="coding-7")],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_coding.md",
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
- Date: 2026-03-13 09:00 UTC+8
- Source agent: coding-7
- Scope: coding-follow-up
- Current state:
  - open coding state remains
- Next suggested step:
  - continue coding work
""",
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "coding-7",
                "planning-sync",
                "--scope",
                "planning-follow-up",
                "--planning-impact",
                "roadmap wording update needed",
                "--verification",
                "cargo test -p mycel-cli",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual("planning-sync", payload["template"])
        self.assertEqual(0, payload["superseded_count"])
        self.assertEqual(2, mailbox.count("- Status: open"))
        self.assertIn("## Work Continuation Handoff", mailbox)
        self.assertIn("## Planning Sync Handoff", mailbox)

    def test_create_planning_sync_supersedes_existing_open_cross_role_entry_only(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_coding", role="coding", display_id="coding-7")],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_coding.md",
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open

## Planning Sync Handoff

- Status: open
""",
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "coding-7",
                "planning-sync",
                "--scope",
                "new-planning-follow-up",
                "--planning-impact",
                "checklist wording update needed",
                "--verification",
                "cargo test -p mycel-cli",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual(1, payload["superseded_count"])
        self.assertEqual(2, mailbox.count("- Status: open"))
        self.assertEqual(1, mailbox.count("- Status: superseded"))
        self.assertIn("- Scope: new-planning-follow-up", mailbox)

    def test_planning_resolution_does_not_close_existing_open_entry(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-13T10:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry(agent_uid="agt_doc", role="doc", display_id="doc-9")],
            }
        )
        self.write_mailbox(
            ".agent-local/mailboxes/agt_doc.md",
            """# Mailbox for agt_doc

## Doc Continuation Note

- Status: open
- Date: 2026-03-13 17:18 UTC+8
- Source agent: doc-9
- Scope: planning-sync-batch
- Current state:
  - open state remains
- Evidence:
  - scripts/check-plan-refresh.sh
- Next suggested step:
  - continue the batch
""",
        )

        payload = json.loads(
            self.run_cli(
                "create",
                "doc-9",
                "planning-resolution",
                "--scope",
                "planning-sync-batch",
                "--source-handoff",
                "coding-7 peer-store sync",
                "--planning-impact",
                "roadmap wording refreshed",
                "--verification",
                "git diff --check",
                "--remaining-follow-up",
                "none",
                "--json",
            ).stdout
        )

        mailbox = (self.root / payload["mailbox"]).read_text(encoding="utf-8")
        self.assertEqual("resolved", payload["status"])
        self.assertEqual(0, payload["superseded_count"])
        self.assertEqual(1, mailbox.count("- Status: open"))
        self.assertEqual(1, mailbox.count("- Status: resolved"))


if __name__ == "__main__":
    unittest.main()
