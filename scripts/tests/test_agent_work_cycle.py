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
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentWorkCycleCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_WORK_CYCLE, self.root / "scripts" / "agent_work_cycle.py")
        shutil.copy2(SOURCE_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_TIMESTAMP, self.root / "scripts" / "agent_timestamp.py")
        shutil.copy2(SOURCE_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, self.root / "scripts" / "item_id_checklist_mark.py")
        (self.root / "scripts" / "agent_work_cycle.py").chmod(0o755)
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)
        (self.root / "scripts" / "agent_timestamp.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist_mark.py").chmod(0o755)

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

    def write_agents_md(self) -> None:
        (self.root / "AGENTS.md").write_text(
            """# Repo Working Agreements

## New chat bootstrap
- Bootstrap one <!-- item-id: bootstrap.one -->

## Work Cycle Workflow
- Begin the work cycle <!-- item-id: workflow.touch-work-cycle -->
- Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->
- Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->
- Finish the work cycle <!-- item-id: workflow.finish-work-cycle -->
""",
            encoding="utf-8",
        )

    def replace_in_file(self, relative_path: str, old: str, new: str) -> None:
        path = self.root / relative_path
        content = path.read_text(encoding="utf-8")
        path.write_text(content.replace(old, new), encoding="utf-8")

    def write_mailbox(self, agent_uid: str, content: str) -> None:
        path = self.root / ".agent-local" / "mailboxes" / f"{agent_uid}.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def write_shared_fallback_mailbox(self, relative_path: str, content: str) -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def prepare_completed_bootstrap_batch(self, *, role: str = "doc") -> str:
        self.write_agents_md()
        claim = self.run_registry("claim", role, "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.replace_in_file(
            start["bootstrap_output"],
            "- [ ] Bootstrap one <!-- item-id: bootstrap.one -->",
            "- [X] Bootstrap one <!-- item-id: bootstrap.one -->",
        )
        self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        self.replace_in_file(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            "- [ ] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
            "- [X] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
        )
        end = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper")
        self.assertEqual(0, end.returncode)
        return agent_uid

    def prepare_second_batch(self, *, role: str = "doc") -> str:
        agent_uid = self.prepare_completed_bootstrap_batch(role=role)
        begin = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        self.assertEqual(0, begin.returncode)
        self.replace_in_file(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "- [ ] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
            "- [X] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
        )
        self.replace_in_file(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "- [ ] Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->",
            "- [X] Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->",
        )
        return agent_uid

    def test_begin_touches_agent_and_prints_before_work_line(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        checklist = (
            self.root
            / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md"
        ).read_text(encoding="utf-8")

        self.assertIn(f"workcycle_output: .agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md", proc.stdout)
        self.assertIn("batch_num: 1", proc.stdout)
        self.assertIn(f"agent_uid: {agent_uid}", proc.stdout)
        self.assertIn("current_status: active", proc.stdout)
        self.assertIn(f"Before work | doc-1 ({agent_uid}) | timestamp-wrapper", proc.stdout)
        self.assertIn("- [-] Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->", checklist)

    def test_end_finishes_agent_and_prints_after_work_line(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.replace_in_file(
            start["bootstrap_output"],
            "- [ ] Bootstrap one <!-- item-id: bootstrap.one -->",
            "- [X] Bootstrap one <!-- item-id: bootstrap.one -->",
        )

        begin = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        self.assertEqual(0, begin.returncode)
        self.replace_in_file(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            "- [ ] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
            "- [X] Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper")

        self.assertEqual(0, proc.returncode)
        self.assertIn(f"agent_uid: {agent_uid}", proc.stdout)
        self.assertIn("current_status: inactive", proc.stdout)
        self.assertIn(f"After work | doc-1 ({agent_uid}) | timestamp-wrapper", proc.stdout)
        self.assertIn("bootstrap_batch: true", proc.stdout)
        self.assertIn("checklists_checked: 2", proc.stdout)
        self.assertIn("unchecked_items: 0", proc.stdout)
        self.assertIn(f"mailbox: .agent-local/mailboxes/{agent_uid}.md", proc.stdout)
        self.assertIn("open_handoffs: 0", proc.stdout)
        self.assertNotIn("checklist_paths:", proc.stdout)
        self.assertNotIn("open_handoff_lines:", proc.stdout)

    def test_end_returns_pending_when_bootstrap_or_workcycle_items_are_unchecked(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("bootstrap_batch: true", proc.stdout)
        self.assertIn("unchecked_items: 2", proc.stdout)

    def test_end_returns_pending_when_mailbox_has_multiple_open_handoffs(self) -> None:
        agent_uid = self.prepare_second_batch(role="doc")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_doc

## Doc Continuation Note

- Status: open

## Planning Sync Handoff

- Status: open

## Third

- Status: open
""",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("unchecked_items: 0", proc.stdout)
        self.assertIn("open_handoffs: 3", proc.stdout)
        self.assertIn("same_role_open_handoffs: 1", proc.stdout)
        self.assertIn("other_role_open_handoffs: 2", proc.stdout)
        self.assertIn("same_role_open_handoff_lines:", proc.stdout)
        self.assertIn("other_role_open_handoff_lines:", proc.stdout)

    def test_end_returns_pending_when_non_bootstrap_batch_has_no_open_same_role_handoff(self) -> None:
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Planning Sync Handoff

- Status: open
""",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("unchecked_items: 0", proc.stdout)
        self.assertIn("same_role_open_handoffs: 0", proc.stdout)
        self.assertIn("other_role_open_handoffs: 1", proc.stdout)

    def test_end_allows_one_open_same_role_handoff_and_one_open_other_role_handoff(self) -> None:
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open

## Planning Sync Handoff

- Status: open
""",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("unchecked_items: 0", proc.stdout)
        self.assertIn("open_handoffs: 2", proc.stdout)
        self.assertIn("same_role_open_handoffs: 1", proc.stdout)
        self.assertIn("other_role_open_handoffs: 1", proc.stdout)
        self.assertIn("oversized_shared_fallback_mailboxes: 0", proc.stdout)

    def test_end_allows_delivery_same_role_handoff_and_one_open_other_role_handoff(self) -> None:
        agent_uid = self.prepare_second_batch(role="delivery")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_delivery

## Delivery Continuation Note

- Status: open

## Planning Sync Handoff

- Status: open
""",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("same_role_open_handoffs: 1", proc.stdout)
        self.assertIn("other_role_open_handoffs: 1", proc.stdout)

    def test_end_returns_pending_when_shared_fallback_mailbox_exceeds_limit(self) -> None:
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        self.write_shared_fallback_mailbox(".agent-local/coding-to-doc.md", "x" * 1025)

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("shared_fallback_mailboxes_checked: 1", proc.stdout)
        self.assertIn("shared_fallback_mailbox_limit_bytes: 1024", proc.stdout)
        self.assertIn("oversized_shared_fallback_mailboxes: 1", proc.stdout)
        self.assertIn(".agent-local/coding-to-doc.md (1025 bytes)", proc.stdout)


if __name__ == "__main__":
    unittest.main()
