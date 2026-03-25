import json
import os
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
SOURCE_CODEX_TOKEN_USAGE = REPO_ROOT / "scripts" / "codex_token_usage_summary.py"
SOURCE_RUNTIME_PREFLIGHT = REPO_ROOT / "scripts" / "check-runtime-preflight.py"
SOURCE_CHECKLIST_GC = REPO_ROOT / "scripts" / "agent_checklist_gc.py"
SOURCE_MAILBOX_GC = REPO_ROOT / "scripts" / "mailbox_gc.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentBootstrapCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / "bin").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_BOOTSTRAP, self.root / "scripts" / "agent_bootstrap.py")
        shutil.copy2(SOURCE_WORK_CYCLE, self.root / "scripts" / "agent_work_cycle.py")
        shutil.copy2(SOURCE_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_TIMESTAMP, self.root / "scripts" / "agent_timestamp.py")
        shutil.copy2(SOURCE_CODEX_TOKEN_USAGE, self.root / "scripts" / "codex_token_usage_summary.py")
        shutil.copy2(SOURCE_RUNTIME_PREFLIGHT, self.root / "scripts" / "check-runtime-preflight.py")
        shutil.copy2(SOURCE_CHECKLIST_GC, self.root / "scripts" / "agent_checklist_gc.py")
        shutil.copy2(SOURCE_MAILBOX_GC, self.root / "scripts" / "mailbox_gc.py")
        shutil.copy2(SOURCE_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, self.root / "scripts" / "item_id_checklist_mark.py")
        for script_name in [
            "agent_bootstrap.py",
            "agent_work_cycle.py",
            "agent_registry.py",
            "agent_timestamp.py",
            "codex_token_usage_summary.py",
            "check-runtime-preflight.py",
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
- Read dev setup status when present <!-- item-id: bootstrap.read-dev-setup-status -->
- Skip repeated dev setup checks when status is ready <!-- item-id: bootstrap.skip-dev-setup-when-ready -->
- Run bootstrap runtime preflight <!-- item-id: bootstrap.runtime-preflight -->
- Refresh dev setup status when it is missing or not ready <!-- item-id: bootstrap.refresh-dev-setup-when-needed -->
- Use the dev setup template when refreshing local status <!-- item-id: bootstrap.dev-setup-template -->
- Read the role checklist entrypoint <!-- item-id: bootstrap.read-role-checklists -->
- Read the agent registry docs and local registry <!-- item-id: bootstrap.read-agent-registry -->
- Start bootstrap immediately after the user assigns a role <!-- item-id: bootstrap.no-confirm-after-role-read -->
- Auto-claim a role when the user leaves it unspecified <!-- item-id: bootstrap.claim-auto -->
- Claim a fresh agent for each new chat <!-- item-id: bootstrap.claim-fresh-agent-for-new-chat -->
- Review the latest same-role handoff when one exists <!-- item-id: bootstrap.review-latest-same-role-handoff -->

## Work Cycle Workflow
- Begin the work cycle <!-- item-id: workflow.touch-work-cycle -->
- Run git status <!-- item-id: bootstrap.git-status -->
- Install additional tools if needed <!-- item-id: workflow.install-needed-tools -->
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
        (self.root / ".agent-local" / "dev-setup-status.md").write_text(
            """# Dev Setup Status

- Status: ready
""",
            encoding="utf-8",
        )
        (self.root / "docs" / "ROLE-CHECKLISTS").mkdir(parents=True, exist_ok=True)
        (self.root / "docs" / "AGENT-REGISTRY.md").write_text(
            "# Agent Registry Protocol\n",
            encoding="utf-8",
        )
        (self.root / "docs" / "ROLE-CHECKLISTS" / "README.md").write_text(
            "# Role Checklists\n",
            encoding="utf-8",
        )
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
        self.write_fake_gh(
            [
                {
                    "databaseId": 23539308106,
                    "status": "completed",
                    "conclusion": "success",
                    "workflowName": "CI",
                    "displayTitle": "bootstrap smoke",
                    "headSha": "c3e132e17d52e11a5d94bb3dd7d5124972a489bb",
                    "updatedAt": "2026-03-25T11:46:24Z",
                }
            ]
        )

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_fake_codex_thread_metadata(self, *, model: str = "gpt-5.4", effort: str = "medium") -> None:
        path = self.root / "scripts" / "codex_thread_metadata.py"
        path.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "if '--shell' in sys.argv:\n"
            f"    print('MODEL=\"{model}\"')\n"
            f"    print('EFFORT=\"{effort}\"')\n"
            "    print('THREAD_ID=\"test-thread\"')\n"
            "    print('STATE_DB=\"/tmp/test.sqlite\"')\n"
            "else:\n"
            f"    print('model: {model}')\n"
            f"    print('effort: {effort}')\n",
            encoding="utf-8",
        )
        path.chmod(0o755)

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        env.pop("CODEX_THREAD_ID", None)
        env["PATH"] = f"{self.root / 'bin'}:{env.get('PATH', '')}"
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_bootstrap.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=env,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_fake_gh(self, runs: list[dict[str, object]] | None = None, *, exit_code: int = 0, stderr: str = "") -> None:
        gh_path = self.root / "bin" / "gh"
        if runs is None:
            body = "[]"
        else:
            body = json.dumps(runs)
        gh_path.write_text(
            "#!/usr/bin/env python3\n"
            "import json\n"
            "import sys\n"
            f"EXIT_CODE = {exit_code}\n"
            f"STDERR = {stderr!r}\n"
            f"BODY = {body!r}\n"
            "if EXIT_CODE != 0:\n"
            "    if STDERR:\n"
            "        sys.stderr.write(STDERR)\n"
            "    raise SystemExit(EXIT_CODE)\n"
            "print(BODY)\n",
            encoding="utf-8",
        )
        gh_path.chmod(0o755)

    def set_checklist_state(self, relative_path: str, item_id: str, state: str, label: str) -> None:
        path = self.root / relative_path
        content = path.read_text(encoding="utf-8")
        new = f"- [{state}] {label} <!-- item-id: {item_id} -->"
        for current_state in (" ", "X", "-", "!"):
            old = f"- [{current_state}] {label} <!-- item-id: {item_id} -->"
            if old in content:
                path.write_text(content.replace(old, new), encoding="utf-8")
                return
        self.fail(f"missing checklist item {item_id} in {relative_path}")

    def mark_workcycle_defaults(self, relative_path: str) -> None:
        states = [
            ("bootstrap.git-status", "Run git status", "X"),
            ("workflow.install-needed-tools", "Install additional tools if needed", "-"),
            ("workflow.reply-with-plan-and-status", "Reply with a short plan", "-"),
            ("workflow.timestamped-commentary", "Use the exact emitted timestamp line", "X"),
            ("workflow.no-double-touch-finish", "Avoid double-touching the registry", "X"),
            ("workflow.files-changed-summary", "Include a files-changed summary when source changes land", "-"),
            ("workflow.final-after-work-line-before-next-items", "Put the after-work line before next-stage options", "X"),
            ("workflow.next-stage-options", "Offer next-stage options", "X"),
            ("workflow.next-stage-highest-value-first", "Highest-value option first", "X"),
            ("workflow.next-stage-numbered-options", "Use numbered options", "X"),
            ("workflow.next-stage-roadmap-location", "Include roadmap location when relevant", "-"),
            ("workflow.next-stage-clarifying-questions", "Ask short clarifying questions when needed", "-"),
        ]
        for item_id, label, state in states:
            self.set_checklist_state(relative_path, item_id, state, label)

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
            "use the latest completed CI result above as the baseline before choosing the next implementation slice",
            payload["next_actions"],
        )
        self.assertIn(
            "full mailbox scans unless the chat is resuming, taking over, or working an overlapping coding scope",
            payload["deferred_reads"],
        )
        self.assertEqual("completed", payload["latest_completed_ci"]["status"])
        self.assertEqual("CI", payload["latest_completed_ci"]["workflowName"])
        self.assertEqual("success", payload["latest_completed_ci"]["conclusion"])
        self.assertEqual(
            "use the latest completed CI result above as the baseline before choosing the next implementation slice",
            payload["next_actions"][0],
        )

    def test_model_id_appears_in_timestamp_and_claimed_agent_label(self) -> None:
        self.write_fake_codex_thread_metadata()
        proc = self.run_cli("coding", "--scope", "ci-triage", "--model-id", "claude-sonnet-4-6", "--concise")

        self.assertIn("claimed_agent: coding-1 (agt_", proc.stdout)
        self.assertIn("/gpt-5.4/medium)", proc.stdout)
        self.assertRegex(proc.stdout, r"Before work \| coding-1 \(agt_[a-z0-9]+/gpt-5\.4/medium\) \| ci-triage")

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
        self.assertIn("latest_completed_ci:", proc.stdout)
        self.assertIn("workflowName: CI", proc.stdout)
        self.assertIn("conclusion: success", proc.stdout)
        self.assertNotIn("bootstrap_output:", proc.stdout)
        self.assertNotIn("mailbox_link:", proc.stdout)
        self.assertNotIn("fast_path_steps:", proc.stdout)

    def test_bootstrap_reports_unavailable_ci_lookup_without_failing(self) -> None:
        self.write_fake_gh(exit_code=1, stderr="gh unavailable\n")

        proc = self.run_cli("--json", "delivery", "--scope", "ci-baseline", "--model-id", "test-model")
        payload = json.loads(proc.stdout)

        self.assertEqual("delivery", payload["role"])
        self.assertEqual("unavailable", payload["latest_completed_ci"]["status"])
        self.assertIn("gh unavailable", payload["latest_completed_ci"]["message"])
        self.assertEqual(
            "re-run the latest completed CI lookup before delivery follow-up because bootstrap could not confirm it",
            payload["next_actions"][0],
        )

    def test_bootstrap_appends_latest_same_role_handoff_review_to_next_actions(self) -> None:
        mailbox_dir = self.root / ".agent-local" / "mailboxes"
        mailbox_dir.mkdir(parents=True, exist_ok=True)
        (mailbox_dir / "agt_prev.md").write_text(
            """# Mailbox for agt_prev

## Work Continuation Handoff

- Status: open
- Date: 2026-03-24 12:10 UTC+8
- Source agent: coding-7
- Source role: coding
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
        self.assertIn("review the latest same-role handoff from coding-7 (role=coding)", proc.stdout)
        self.assertIn("restore-sync-gap", proc.stdout)
        self.assertIn("re-run the sync proof after wiring the stored root fixture", proc.stdout)
        bootstrap_checklists = list(self.root.glob(".agent-local/agents/*/checklists/AGENTS-bootstrap-checklist.md"))
        self.assertEqual(1, len(bootstrap_checklists))
        checklist_text = bootstrap_checklists[0].read_text(encoding="utf-8")
        self.assertIn("## Latest Same-Role Handoff Review", checklist_text)
        self.assertIn("Reviewed agent: `coding-7`", checklist_text)
        self.assertIn("Handoff source role: `coding`", checklist_text)
        self.assertIn("Handoff scope: `restore-sync-gap`", checklist_text)
        self.assertIn("re-run the sync proof after wiring the stored root fixture", checklist_text)

    def test_bootstrap_ignores_active_same_role_handoff_from_other_agent(self) -> None:
        mailbox_dir = self.root / ".agent-local" / "mailboxes"
        mailbox_dir.mkdir(parents=True, exist_ok=True)
        (mailbox_dir / "agt_active.md").write_text(
            """# Mailbox for agt_active

## Work Continuation Handoff

- Status: open
- Date: 2026-03-24 12:10 UTC+8
- Source agent: coding-7
- Source role: coding
- Scope: active-scope
- Next suggested step:
  - continue the still-active coding-7 task
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
                            "agent_uid": "agt_active",
                            "role": "coding",
                            "current_display_id": "coding-7",
                            "display_history": [
                                {
                                    "display_id": "coding-7",
                                    "assigned_at": "2026-03-24T11:00:00+0800",
                                    "released_at": None,
                                    "released_reason": None,
                                }
                            ],
                            "assigned_by": "user",
                            "assigned_at": "2026-03-24T11:00:00+0800",
                            "confirmed_by_agent": True,
                            "confirmed_at": "2026-03-24T11:00:10+0800",
                            "last_touched_at": "2026-03-24T12:12:00+0800",
                            "inactive_at": None,
                            "paused_at": None,
                            "status": "active",
                            "scope": "active-scope",
                            "files": [],
                            "mailbox": ".agent-local/mailboxes/agt_active.md",
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

        proc = self.run_cli("coding", "--scope", "fresh-scope", "--model-id", "test-model", "--concise")

        self.assertIn("next_actions:", proc.stdout)
        self.assertNotIn("review the latest same-role handoff from coding-7", proc.stdout)

    def test_bootstrap_ignores_compaction_abort_handoff(self) -> None:
        mailbox_dir = self.root / ".agent-local" / "mailboxes"
        mailbox_dir.mkdir(parents=True, exist_ok=True)
        (mailbox_dir / "agt_prev.md").write_text(
            """# Mailbox for agt_prev

## Work Continuation Handoff

- Status: open
- Date: 2026-03-24 12:10 UTC+8
- Source agent: coding-7
- Source role: coding
- Scope: pending scope
- Current state:
  - Compact context detected in the current chat thread before work started, so this workcycle was aborted.
- Next suggested step:
  - Open a fresh chat for better performance and continue from this handoff.
- Notes:
  - Compaction event detected at 2026-03-24T04:10:00Z in /tmp/rollout.jsonl.
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
                            "current_display_id": "coding-7",
                            "display_history": [
                                {
                                    "display_id": "coding-7",
                                    "assigned_at": "2026-03-24T11:00:00+0800",
                                    "released_at": None,
                                    "released_reason": None,
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
                            "scope": "pending scope",
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

        proc = self.run_cli("coding", "--scope", "fresh-scope", "--model-id", "test-model", "--concise")

        self.assertIn("next_actions:", proc.stdout)
        self.assertNotIn("review the latest same-role handoff from coding-7", proc.stdout)

    def test_bootstrap_marks_completed_bootstrap_items_for_clean_first_closeout(self) -> None:
        proc = self.run_cli("--json", "coding", "--scope", "clean-closeout", "--model-id", "test-model")
        payload = json.loads(proc.stdout)

        bootstrap_path = self.root / payload["bootstrap_output"]
        bootstrap_text = bootstrap_path.read_text(encoding="utf-8")
        self.assertIn("- [X] Scan the repo root <!-- item-id: bootstrap.repo-layout -->", bootstrap_text)
        self.assertIn("- [X] Read dev setup status when present <!-- item-id: bootstrap.read-dev-setup-status -->", bootstrap_text)
        self.assertIn("- [X] Skip repeated dev setup checks when status is ready <!-- item-id: bootstrap.skip-dev-setup-when-ready -->", bootstrap_text)
        self.assertIn("- [X] Run bootstrap runtime preflight <!-- item-id: bootstrap.runtime-preflight -->", bootstrap_text)
        self.assertIn("- [-] Refresh dev setup status when it is missing or not ready <!-- item-id: bootstrap.refresh-dev-setup-when-needed -->", bootstrap_text)
        self.assertIn("- [X] Read the role checklist entrypoint <!-- item-id: bootstrap.read-role-checklists -->", bootstrap_text)
        self.assertIn("- [X] Read the agent registry docs and local registry <!-- item-id: bootstrap.read-agent-registry -->", bootstrap_text)
        self.assertIn("- [X] Start bootstrap immediately after the user assigns a role <!-- item-id: bootstrap.no-confirm-after-role-read -->", bootstrap_text)
        self.assertIn("- [-] Auto-claim a role when the user leaves it unspecified <!-- item-id: bootstrap.claim-auto -->", bootstrap_text)
        self.assertIn("- [X] Claim a fresh agent for each new chat <!-- item-id: bootstrap.claim-fresh-agent-for-new-chat -->", bootstrap_text)
        self.assertIn("- [-] Review the latest same-role handoff when one exists <!-- item-id: bootstrap.review-latest-same-role-handoff -->", bootstrap_text)

        workcycle_rel = payload["workcycle_output"]
        self.mark_workcycle_defaults(workcycle_rel)
        end = subprocess.run(
            [str(self.root / "scripts" / "agent_work_cycle.py"), "end", payload["agent_uid"]],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(0, end.returncode, end.stderr or end.stdout)
        self.assertIn("unchecked_items: 0", end.stdout)
        self.assertNotIn("unchecked_in:", end.stdout)


if __name__ == "__main__":
    unittest.main()
