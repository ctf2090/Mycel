import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_BOOTSTRAP = REPO_ROOT / "scripts" / "agent_bootstrap.py"
SOURCE_WORK_CYCLE = REPO_ROOT / "scripts" / "agent_work_cycle.py"
SOURCE_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_TIMESTAMP = REPO_ROOT / "scripts" / "agent_timestamp.py"
SOURCE_CHECKLIST_GC = REPO_ROOT / "scripts" / "agent_checklist_gc.py"
SOURCE_MAILBOX_GC = REPO_ROOT / "scripts" / "mailbox_gc.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentBootstrapCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_BOOTSTRAP, self.root / "scripts" / "agent_bootstrap.py")
        shutil.copy2(SOURCE_WORK_CYCLE, self.root / "scripts" / "agent_work_cycle.py")
        shutil.copy2(SOURCE_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_TIMESTAMP, self.root / "scripts" / "agent_timestamp.py")
        shutil.copy2(SOURCE_CHECKLIST_GC, self.root / "scripts" / "agent_checklist_gc.py")
        shutil.copy2(SOURCE_MAILBOX_GC, self.root / "scripts" / "mailbox_gc.py")
        shutil.copy2(SOURCE_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, self.root / "scripts" / "item_id_checklist_mark.py")
        for script_name in [
            "agent_bootstrap.py",
            "agent_work_cycle.py",
            "agent_registry.py",
            "agent_timestamp.py",
            "agent_checklist_gc.py",
            "mailbox_gc.py",
            "item_id_checklist.py",
            "item_id_checklist_mark.py",
        ]:
            (self.root / "scripts" / script_name).chmod(0o755)

        (self.root / "AGENTS.md").write_text(
            """# Repo Working Agreements

## New chat bootstrap
- Scan the repo root <!-- item-id: bootstrap.repo-layout -->

## Work Cycle Workflow
- Begin the work cycle <!-- item-id: workflow.touch-work-cycle -->
- Run git status <!-- item-id: bootstrap.git-status -->
- Install additional tools if needed <!-- item-id: workflow.install-needed-tools -->
- Run runtime preflight before verification <!-- item-id: workflow.runtime-preflight-before-verification -->
- Reply with a short plan <!-- item-id: workflow.reply-with-plan-and-status -->
- Use the exact emitted timestamp line <!-- item-id: workflow.timestamped-commentary -->
- Avoid double-touching the registry <!-- item-id: workflow.no-double-touch-finish -->
- Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->
- Finish the work cycle <!-- item-id: workflow.finish-work-cycle -->
- Include a files-changed summary when source changes land <!-- item-id: workflow.files-changed-summary -->
- Put the after-work line before next-stage options <!-- item-id: workflow.final-after-work-line-before-next-items -->
- Offer next-stage options <!-- item-id: workflow.next-stage-options -->
  - Highest-value option first <!-- item-id: workflow.next-stage-highest-value-first -->
  - Use numbered options <!-- item-id: workflow.next-stage-numbered-options -->
  - Include roadmap location when relevant <!-- item-id: workflow.next-stage-roadmap-location -->
  - Ask short clarifying questions when needed <!-- item-id: workflow.next-stage-clarifying-questions -->
""",
            encoding="utf-8",
        )
        (self.root / "docs" / "ROLE-CHECKLISTS").mkdir(parents=True, exist_ok=True)
        (self.root / "docs" / "ROLE-CHECKLISTS" / "doc.md").write_text(
            """# Doc Role Checklist

## New chat bootstrap
- Doc bootstrap <!-- item-id: doc.bootstrap.one -->

## Work Cycle Workflow
- Doc workflow <!-- item-id: doc.workflow.one -->
""",
            encoding="utf-8",
        )
        (self.root / "docs" / "ROLE-CHECKLISTS" / "coding.md").write_text(
            """# Coding Role Checklist

## New chat bootstrap
- Coding bootstrap <!-- item-id: coding.bootstrap.one -->

## Work Cycle Workflow
- Coding workflow <!-- item-id: coding.workflow.one -->
""",
            encoding="utf-8",
        )
        (self.root / "docs" / "ROLE-CHECKLISTS" / "delivery.md").write_text(
            """# Delivery Role Checklist

## New chat bootstrap
- Delivery bootstrap <!-- item-id: delivery.bootstrap.one -->

## Work Cycle Workflow
- Delivery workflow <!-- item-id: delivery.workflow.one -->
""",
            encoding="utf-8",
        )
        subprocess.run(["git", "init", "-b", "main"], cwd=self.root, check=True, capture_output=True, text=True)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_bootstrap.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_text_output_combines_claim_start_begin_and_git_status(self) -> None:
        proc = self.run_cli("doc", "--scope", "fast-bootstrap", "--model-id", "test-model")

        self.assertIn("agent_uid: agt_", proc.stdout)
        self.assertIn("display_id: doc-1", proc.stdout)
        self.assertIn("role: doc", proc.stdout)
        self.assertIn("scope: fast-bootstrap", proc.stdout)
        self.assertIn("bootstrap_output: .agent-local/agents/", proc.stdout)
        self.assertIn("workcycle_output: .agent-local/agents/", proc.stdout)
        self.assertIn("current_status: active", proc.stdout)
        self.assertIn("startup_mode: fresh-chat-fast-path", proc.stdout)
        self.assertRegex(proc.stdout, r"Before work \| doc-1 \(agt_[a-z0-9]+/test-model\) \| fast-bootstrap")
        self.assertIn("repo_status:\n  ## No commits yet on main", proc.stdout)
        self.assertRegex(
            proc.stdout,
            r"closeout_command: python3 scripts/agent_work_cycle.py end agt_[a-z0-9]+",
        )
        self.assertIn("fast_path_steps:", proc.stdout)
        self.assertIn("next_actions:", proc.stdout)
        self.assertIn("deferred_reads:", proc.stdout)
        self.assertIn("wait for the concrete doc task", proc.stdout)
        self.assertIn("review open Dependabot pull requests first", proc.stdout)
        self.assertIn("review open human-authored product pull requests", proc.stdout)
        self.assertNotIn("latest completed CI result", proc.stdout)

    def test_json_output_returns_combined_payload(self) -> None:
        proc = self.run_cli("--json", "--model-id", "test-model")
        payload = json.loads(proc.stdout)

        self.assertEqual("coding", payload["role"])
        self.assertEqual("coding-1", payload["display_id"])
        self.assertEqual("pending scope", payload["scope"])
        self.assertTrue(payload["agent_uid"].startswith("agt_"))
        self.assertTrue(payload["bootstrap_output"].startswith(".agent-local/agents/"))
        self.assertTrue(payload["workcycle_output"].startswith(".agent-local/agents/"))
        self.assertEqual("active", payload["current_status"])
        self.assertEqual("fresh-chat-fast-path", payload["startup_mode"])
        self.assertRegex(
            payload["closeout_command"],
            r"python3 scripts/agent_work_cycle.py end agt_[a-z0-9]+",
        )
        self.assertRegex(
            payload["before_work_line"],
            r"Before work \| coding-1 \(agt_[a-z0-9]+/test-model\) \| pending scope",
        )
        self.assertEqual("## No commits yet on main", payload["repo_status"][0])
        self.assertIn("?? .agent-local/", payload["repo_status"])
        self.assertIn("?? AGENTS.md", payload["repo_status"])
        self.assertIn("?? scripts/", payload["repo_status"])
        self.assertEqual(
            [
                "scan the repo root with ls",
                "read AGENTS-LOCAL.md if it exists, then read .agent-local/dev-setup-status.md",
                "read docs/ROLE-CHECKLISTS/README.md, docs/AGENT-REGISTRY.md, and .agent-local/agents.json",
                "run scripts/agent_bootstrap.py <role> --model-id <model_id> or scripts/agent_bootstrap.py auto --model-id <model_id>",
                "check the latest completed CI result for the previous push before implementation or delivery work",
            ],
            payload["fast_path_steps"],
        )
        self.assertIn(
            "check the latest completed CI result for the previous push before implementation work",
            payload["next_actions"],
        )
        self.assertIn(
            "full mailbox scans unless the chat is resuming, taking over, or working an overlapping coding scope",
            payload["deferred_reads"],
        )

    def test_model_id_appears_in_timestamp_and_claimed_agent_label(self) -> None:
        proc = self.run_cli("coding", "--scope", "ci-triage", "--model-id", "claude-sonnet-4-6", "--concise")

        self.assertIn("claimed_agent: coding-1 (agt_", proc.stdout)
        self.assertIn("/claude-sonnet-4-6)", proc.stdout)
        self.assertRegex(proc.stdout, r"Before work \| coding-1 \(agt_[a-z0-9]+/claude-sonnet-4-6\) \| ci-triage")

    def test_concise_text_output_keeps_user_facing_summary_short(self) -> None:
        proc = self.run_cli("coding", "--scope", "relay-ready", "--model-id", "test-model", "--concise")

        self.assertIn("claimed_agent: coding-1 (agt_", proc.stdout)
        self.assertIn("/test-model)", proc.stdout)
        self.assertIn("role: coding", proc.stdout)
        self.assertIn("scope: relay-ready", proc.stdout)
        self.assertIn("startup_mode: fresh-chat-fast-path", proc.stdout)
        self.assertRegex(proc.stdout, r"Before work \| coding-1 \(agt_[a-z0-9]+/test-model\) \| relay-ready")
        self.assertIn("repo_status:\n  ## No commits yet on main", proc.stdout)
        self.assertRegex(
            proc.stdout,
            r"closeout_command: python3 scripts/agent_work_cycle.py end agt_[a-z0-9]+",
        )
        self.assertIn("next_actions:", proc.stdout)
        self.assertIn("deferred_reads:", proc.stdout)
        self.assertNotIn("bootstrap_output:", proc.stdout)
        self.assertNotIn("mailbox_link:", proc.stdout)
        self.assertNotIn("fast_path_steps:", proc.stdout)

    def test_bootstrap_appends_latest_same_role_handoff_review_to_next_actions(self) -> None:
        mailbox_dir = self.root / ".agent-local" / "mailboxes"
        mailbox_dir.mkdir(parents=True, exist_ok=True)
        (mailbox_dir / "agt_prev.md").write_text(
            """# Mailbox for agt_prev

## Work Continuation Handoff

- Status: open
- Date: 2026-03-24 12:10 UTC+8
- Source agent: coding-7
- Scope: restore-sync-gap
- Next suggested step:
  - re-run the sync proof after wiring the stored root fixture
""",
            encoding="utf-8",
        )
        (self.root / ".agent-local" / "agents.json").write_text(
            json.dumps(
                {
                    "version": 2,
                    "updated_at": "2026-03-24T12:12:00+0800",
                    "agent_count": 1,
                    "agents": [
                        {
                            "agent_uid": "agt_prev",
                            "role": "coding",
                            "current_display_id": None,
                            "display_history": [
                                {
                                    "display_id": "coding-7",
                                    "assigned_at": "2026-03-24T11:00:00+0800",
                                    "released_at": "2026-03-24T12:12:00+0800",
                                    "released_reason": "finished",
                                }
                            ],
                            "assigned_by": "user",
                            "assigned_at": "2026-03-24T11:00:00+0800",
                            "confirmed_by_agent": True,
                            "confirmed_at": "2026-03-24T11:00:10+0800",
                            "last_touched_at": "2026-03-24T12:12:00+0800",
                            "inactive_at": "2026-03-24T12:12:00+0800",
                            "paused_at": None,
                            "status": "inactive",
                            "scope": "restore-sync-gap",
                            "files": [],
                            "mailbox": ".agent-local/mailboxes/agt_prev.md",
                            "recovery_of": None,
                            "superseded_by": None,
                        }
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli("coding", "--scope", "resume-scan", "--model-id", "test-model", "--concise")

        self.assertIn("next_actions:", proc.stdout)
        self.assertIn("review the latest same-role handoff from coding-7", proc.stdout)
        self.assertIn("restore-sync-gap", proc.stdout)
        self.assertIn("re-run the sync proof after wiring the stored root fixture", proc.stdout)
        bootstrap_checklists = list(self.root.glob(".agent-local/agents/*/checklists/AGENTS-bootstrap-checklist.md"))
        self.assertEqual(1, len(bootstrap_checklists))
        checklist_text = bootstrap_checklists[0].read_text(encoding="utf-8")
        self.assertIn("## Latest Same-Role Handoff Review", checklist_text)
        self.assertIn("Reviewed agent: `coding-7`", checklist_text)
        self.assertIn("Handoff scope: `restore-sync-gap`", checklist_text)
        self.assertIn("re-run the sync proof after wiring the stored root fixture", checklist_text)


if __name__ == "__main__":
    unittest.main()
