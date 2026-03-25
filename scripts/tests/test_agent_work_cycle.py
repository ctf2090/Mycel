import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_WORK_CYCLE = REPO_ROOT / "scripts" / "agent_work_cycle.py"
SOURCE_REGISTRY = REPO_ROOT / "scripts" / "agent_registry.py"
SOURCE_TIMESTAMP = REPO_ROOT / "scripts" / "agent_timestamp.py"
SOURCE_CODEX_TOKEN_USAGE = REPO_ROOT / "scripts" / "codex_token_usage_summary.py"
SOURCE_CHECKLIST_GC = REPO_ROOT / "scripts" / "agent_checklist_gc.py"
SOURCE_MAILBOX_GC = REPO_ROOT / "scripts" / "mailbox_gc.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class AgentWorkCycleCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.remote_temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_WORK_CYCLE, self.root / "scripts" / "agent_work_cycle.py")
        shutil.copy2(SOURCE_REGISTRY, self.root / "scripts" / "agent_registry.py")
        shutil.copy2(SOURCE_TIMESTAMP, self.root / "scripts" / "agent_timestamp.py")
        shutil.copy2(SOURCE_CODEX_TOKEN_USAGE, self.root / "scripts" / "codex_token_usage_summary.py")
        shutil.copy2(SOURCE_CHECKLIST_GC, self.root / "scripts" / "agent_checklist_gc.py")
        shutil.copy2(SOURCE_MAILBOX_GC, self.root / "scripts" / "mailbox_gc.py")
        shutil.copy2(SOURCE_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, self.root / "scripts" / "item_id_checklist_mark.py")
        (self.root / "scripts" / "agent_work_cycle.py").chmod(0o755)
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)
        (self.root / "scripts" / "agent_timestamp.py").chmod(0o755)
        (self.root / "scripts" / "codex_token_usage_summary.py").chmod(0o755)
        (self.root / "scripts" / "agent_checklist_gc.py").chmod(0o755)
        (self.root / "scripts" / "mailbox_gc.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist_mark.py").chmod(0o755)

    def tearDown(self) -> None:
        self.remote_temp_dir.cleanup()
        self.temp_dir.cleanup()

    def build_env(self, extra_env: dict[str, str] | None = None) -> dict[str, str]:
        env = dict(os.environ)
        env.pop("CODEX_THREAD_ID", None)
        env["HOME"] = str(self.root)
        if extra_env:
            env.update(extra_env)
        return env

    def run_cli(
        self, *args: str, check: bool = True, extra_env: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_work_cycle.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=self.build_env(extra_env),
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
            env=self.build_env(),
        )
        return json.loads(proc.stdout)

    def run_git(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["git", *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=self.build_env(),
        )
        if check and proc.returncode != 0:
            self.fail(f"git command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def load_work_cycle_module(self):
        sys.path.insert(0, str(self.root / "scripts"))
        spec = importlib.util.spec_from_file_location("agent_work_cycle_under_test", self.root / "scripts" / "agent_work_cycle.py")
        if spec is None or spec.loader is None:
            self.fail("failed to load agent_work_cycle.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def write_agents_md(self) -> None:
        (self.root / "AGENTS.md").write_text(
            """# Repo Working Agreements

## New chat bootstrap
- Bootstrap one <!-- item-id: bootstrap.one -->

## Work Cycle Workflow
- Run git status <!-- item-id: bootstrap.git-status -->
- Begin the work cycle <!-- item-id: workflow.touch-work-cycle -->
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

    def write_role_checklist(self, role: str) -> None:
        (self.root / "docs" / "ROLE-CHECKLISTS").mkdir(parents=True, exist_ok=True)
        (self.root / "docs" / "ROLE-CHECKLISTS" / f"{role}.md").write_text(
            f"""# {role.title()} Role Checklist

## New chat bootstrap
- Role bootstrap <!-- item-id: {role}.bootstrap.one -->

## Work Cycle Workflow
- Role workflow <!-- item-id: {role}.workflow.one -->
""",
            encoding="utf-8",
        )

    def init_git_repo(self) -> None:
        self.run_git("init")
        self.run_git("config", "user.name", "Test User")
        self.run_git("config", "user.email", "test@example.com")
        self.run_git("add", ".")
        self.run_git("commit", "-m", "initial")

    def init_origin_main_remote(self) -> Path:
        remote_path = Path(self.remote_temp_dir.name) / "origin.git"
        subprocess.run(["git", "init", "--bare", str(remote_path)], cwd=self.root, check=True, capture_output=True, text=True)
        self.run_git("remote", "add", "origin", str(remote_path))
        self.run_git("push", "-u", "origin", "HEAD:main")
        return remote_path

    def replace_in_file(self, relative_path: str, old: str, new: str) -> None:
        path = self.root / relative_path
        content = path.read_text(encoding="utf-8")
        path.write_text(content.replace(old, new), encoding="utf-8")

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

    def mark_workcycle_defaults(
        self,
        relative_path: str,
        *,
        mailbox_state: str | None,
        plan_state: str = "X",
        scrutinized_state: str = "-",
    ) -> None:
        states = [
            ("bootstrap.git-status", "Run git status", "X"),
            ("workflow.install-needed-tools", "Install additional tools if needed", "-"),
            ("workflow.runtime-preflight-before-verification", "Run runtime preflight before verification", scrutinized_state),
            ("workflow.reply-with-plan-and-status", "Reply with a short plan", plan_state),
            ("workflow.timestamped-commentary", "Use the exact emitted timestamp line", "X"),
            ("workflow.no-double-touch-finish", "Avoid double-touching the registry", "X"),
            ("workflow.files-changed-summary", "Include a files-changed summary when source changes land", scrutinized_state),
            ("workflow.final-after-work-line-before-next-items", "Put the after-work line before next-stage options", "X"),
            ("workflow.next-stage-options", "Offer next-stage options", "X"),
            ("workflow.next-stage-highest-value-first", "Highest-value option first", "X"),
            ("workflow.next-stage-numbered-options", "Use numbered options", "X"),
            ("workflow.next-stage-roadmap-location", "Include roadmap location when relevant", "-"),
            ("workflow.next-stage-clarifying-questions", "Ask short clarifying questions when needed", "-"),
        ]
        if mailbox_state is not None:
            states.append(("workflow.mailbox-handoff-each-cycle", "Leave a mailbox handoff", mailbox_state))
        for item_id, label, state in states:
            self.set_checklist_state(relative_path, item_id, state, label)

    def write_mailbox(self, agent_uid: str, content: str) -> None:
        path = self.root / ".agent-local" / "mailboxes" / f"{agent_uid}.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def write_shared_fallback_mailbox(self, relative_path: str, content: str) -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def write_codex_rollout(
        self,
        thread_id: str,
        *,
        cwd: str | None = None,
        totals: list[tuple[str, int, int]],
    ) -> None:
        rollout_dir = self.root / ".codex" / "sessions" / "2026" / "03" / "25"
        rollout_dir.mkdir(parents=True, exist_ok=True)
        rollout_path = rollout_dir / f"rollout-2026-03-25T06-14-58-{thread_id}.jsonl"
        actual_cwd = cwd or str(self.root)
        lines = [
            json.dumps(
                {
                    "timestamp": "2026-03-25T06:14:58.000Z",
                    "type": "turn_context",
                    "payload": {
                        "cwd": actual_cwd,
                        "turn_id": "turn_1",
                        "model": "gpt-5.4",
                        "effort": "medium",
                    },
                }
            )
        ]
        for timestamp, last_turn_total, cumulative_total in totals:
            lines.append(
                json.dumps(
                    {
                        "timestamp": timestamp,
                        "type": "event_msg",
                        "payload": {
                            "type": "token_count",
                            "info": {
                                "last_token_usage": {
                                    "input_tokens": last_turn_total,
                                    "cached_input_tokens": 0,
                                    "output_tokens": 0,
                                    "reasoning_output_tokens": 0,
                                    "total_tokens": last_turn_total,
                                },
                                "total_token_usage": {
                                    "input_tokens": cumulative_total,
                                    "cached_input_tokens": 0,
                                    "output_tokens": 0,
                                    "reasoning_output_tokens": 0,
                                    "total_tokens": cumulative_total,
                                },
                            },
                        },
                    }
                )
            )
        rollout_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def set_mtime_days_ago(self, relative_path: str, days: int) -> None:
        path = self.root / relative_path
        stat = path.stat()
        stale_time = stat.st_mtime - (days * 86400)
        path.touch()
        os.utime(path, (stale_time, stale_time))

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
        self.mark_workcycle_defaults(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            mailbox_state=None,
        )
        end = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper")
        self.assertEqual(0, end.returncode)
        return agent_uid

    def prepare_second_batch(self, *, role: str = "doc") -> str:
        agent_uid = self.prepare_completed_bootstrap_batch(role=role)
        begin = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        self.assertEqual(0, begin.returncode)
        self.mark_workcycle_defaults(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            mailbox_state="X",
            scrutinized_state="X",
        )
        return agent_uid

    def test_begin_touches_agent_and_prints_before_work_line(self) -> None:
        self.write_agents_md()
        self.write_role_checklist("doc")
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")
        checklist = (
            self.root
            / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md"
        ).read_text(encoding="utf-8")

        self.assertIn(f"workcycle_output: .agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md", proc.stdout)
        self.assertIn(
            f"role_workcycle_output: .agent-local/agents/{agent_uid}/checklists/ROLE-doc-workcycle-checklist-1.md",
            proc.stdout,
        )
        self.assertIn("batch_num: 1", proc.stdout)
        self.assertIn(f"closeout_command: python3 scripts/agent_work_cycle.py end {agent_uid}", proc.stdout)
        self.assertIn(f"agent_uid: {agent_uid}", proc.stdout)
        self.assertIn("current_status: active", proc.stdout)
        self.assertIn(f"Before work | doc-1 ({agent_uid}) | timestamp-wrapper", proc.stdout)
        self.assertIn("- [-] Leave a mailbox handoff <!-- item-id: workflow.mailbox-handoff-each-cycle -->", checklist)
        self.assertTrue(
            (self.root / f".agent-local/agents/{agent_uid}/checklists/ROLE-doc-workcycle-checklist-1.md").exists()
        )

    def test_begin_updates_registry_scope_when_scope_is_provided(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "pending-user-task")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "updated-cycle-scope")
        status = self.run_registry("status", agent_uid)

        self.assertEqual(0, proc.returncode)
        self.assertIn("Before work | doc-1", proc.stdout)
        self.assertEqual("updated-cycle-scope", status["agents"][0]["scope"])

    def test_start_alias_maps_to_begin(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("start", agent_uid, "--scope", "timestamp-wrapper")

        self.assertEqual(0, proc.returncode)
        self.assertIn("workcycle_output:", proc.stdout)
        self.assertIn(f"closeout_command: python3 scripts/agent_work_cycle.py end {agent_uid}", proc.stdout)
        self.assertIn(f"Before work | doc-1 ({agent_uid}) | timestamp-wrapper", proc.stdout)

    def test_start_rejects_model_id_with_targeted_guidance(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "claude-sonnet-4-6")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("start", agent_uid, "--model-id", "claude-sonnet-4-6", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("model id is inferred from the agent registry entry created at claim/bootstrap time", proc.stderr)
        self.assertIn(
            f"Use `python3 scripts/agent_work_cycle.py begin {agent_uid}` --scope <scope>`.",
            proc.stderr,
        )

    def test_end_rejects_batch_flag_with_targeted_guidance(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")

        proc = self.run_cli("end", agent_uid, "--batch", "1", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("batch is inferred from the latest workcycle checklist", proc.stderr)
        self.assertIn(
            f"Use `python3 scripts/agent_work_cycle.py end {agent_uid}`.",
            proc.stderr,
        )

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
        self.mark_workcycle_defaults(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            mailbox_state=None,
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

    def test_begin_includes_model_id_in_timestamp_when_set(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "delivery", "--scope", "ci-triage", "--model-id", "claude-sonnet-4-6")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "ci-triage")

        self.assertIn(f"Before work | delivery-1 ({agent_uid}/claude-sonnet-4-6) | ci-triage", proc.stdout)

    def test_begin_appends_last_turn_token_usage_when_available(self) -> None:
        self.write_agents_md()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )

        self.assertIn(
            f"Before work | doc-1 ({agent_uid}/gpt-5.4) | timestamp-wrapper | last thread turn: 60,135 tok",
            proc.stdout,
        )

    def test_begin_prefers_current_thread_id_over_newer_same_cwd_rollout(self) -> None:
        self.write_agents_md()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504303",
            totals=[("2026-03-25T06:25:03.000Z", 500000, 999999)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )

        self.assertIn("last thread turn: 60,135 tok", proc.stdout)
        self.assertNotIn("500,000 tok", proc.stdout)

    def test_end_returns_pending_when_bootstrap_or_workcycle_items_are_unchecked(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "timestamp-wrapper")

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("bootstrap_batch: true", proc.stdout)
        self.assertIn("unchecked_items: 12", proc.stdout)

    def test_end_appends_estimated_cycle_token_usage_when_available(self) -> None:
        self.write_agents_md()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.replace_in_file(
            start["bootstrap_output"],
            "- [ ] Bootstrap one <!-- item-id: bootstrap.one -->",
            "- [X] Bootstrap one <!-- item-id: bootstrap.one -->",
        )

        begin = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )
        self.assertEqual(0, begin.returncode)
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[
                ("2026-03-25T06:20:03.000Z", 60135, 887020),
                ("2026-03-25T06:25:03.000Z", 45000, 932020),
            ],
        )
        self.mark_workcycle_defaults(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            mailbox_state=None,
        )

        proc = self.run_cli(
            "end",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn(
            f"After work | doc-1 ({agent_uid}/gpt-5.4) | timestamp-wrapper | cycle est. on thread: 45,000 tok",
            proc.stdout,
        )

    def test_end_reuses_frozen_end_token_snapshot_on_repeat_closeout(self) -> None:
        self.write_agents_md()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.replace_in_file(
            start["bootstrap_output"],
            "- [ ] Bootstrap one <!-- item-id: bootstrap.one -->",
            "- [X] Bootstrap one <!-- item-id: bootstrap.one -->",
        )

        begin = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )
        self.assertEqual(0, begin.returncode)
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[
                ("2026-03-25T06:20:03.000Z", 60135, 887020),
                ("2026-03-25T06:25:03.000Z", 45000, 932020),
            ],
        )
        self.mark_workcycle_defaults(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md",
            mailbox_state=None,
        )

        first_end = self.run_cli(
            "end",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )
        self.assertEqual(0, first_end.returncode)
        self.assertIn(
            f"After work | doc-1 ({agent_uid}/gpt-5.4) | timestamp-wrapper | cycle est. on thread: 45,000 tok",
            first_end.stdout,
        )

        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[
                ("2026-03-25T06:20:03.000Z", 60135, 887020),
                ("2026-03-25T06:25:03.000Z", 45000, 932020),
                ("2026-03-25T06:30:03.000Z", 53000, 985020),
            ],
        )

        second_end = self.run_cli(
            "end",
            agent_uid,
            "--scope",
            "timestamp-wrapper",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )
        self.assertEqual(0, second_end.returncode)
        self.assertIn(
            f"After work | doc-1 ({agent_uid}/gpt-5.4) | timestamp-wrapper | cycle est. on thread: 45,000 tok",
            second_end.stdout,
        )
        self.assertNotIn("98,000 tok", second_end.stdout)

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

    def test_end_auto_prunes_stale_orphaned_mailboxes(self) -> None:
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        self.write_shared_fallback_mailbox(".agent-local/mailboxes/agt_orphan.md", "# orphan\n")
        self.set_mtime_days_ago(".agent-local/mailboxes/agt_orphan.md", 4)

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("mailbox_gc_status: ok", proc.stdout)
        self.assertIn("mailbox_gc_min_age_days: 3", proc.stdout)
        self.assertIn("mailbox_gc_deleted: 1", proc.stdout)
        self.assertIn(".agent-local/mailboxes/agt_orphan.md (4 days)", proc.stdout)
        self.assertFalse((self.root / ".agent-local/mailboxes/agt_orphan.md").exists())

    def test_resolve_agent_mailbox_path_rejects_mailbox_outside_mailbox_directory(self) -> None:
        module = self.load_work_cycle_module()
        module.run_registry = lambda command, agent_ref, scope=None: {  # type: ignore[assignment]
            "agents": [{"mailbox": "../escaped-mailbox.md"}]
        }

        with self.assertRaises(module.WorkCycleError) as exc_info:
            module.resolve_agent_mailbox_path("agt_bad")

        self.assertIn(
            "has mailbox outside .agent-local/mailboxes/: ../escaped-mailbox.md",
            str(exc_info.exception),
        )

    def test_end_auto_prunes_older_agent_workcycle_checklists(self) -> None:
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        template = self.root / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md"
        template_text = template.read_text(encoding="utf-8")
        for batch in range(3, 23):
            path = self.root / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-{batch}.md"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(template_text, encoding="utf-8")

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("agent_checklist_gc_status: ok", proc.stdout)
        self.assertIn("agent_checklist_gc_keep_workcycle_batches: 20", proc.stdout)
        self.assertIn("agent_checklist_gc_deleted: 2", proc.stdout)
        self.assertFalse(
            (self.root / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-1.md").exists()
        )
        self.assertFalse(
            (self.root / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md").exists()
        )
        self.assertTrue(
            (self.root / f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-22.md").exists()
        )

    def test_end_allows_files_changed_summary_not_needed_for_decision_only_cycle(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "-",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("scrutinized_not_needed_violations: 0", proc.stdout)

    def test_end_requires_files_changed_summary_when_cycle_changes_source_files(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "scripts" / "agent_timestamp.py").write_text(
            (self.root / "scripts" / "agent_timestamp.py").read_text(encoding="utf-8")
            + "\n# source change\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_timestamp.py")
        self.run_git("commit", "-m", "committed source change")
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "-",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("scrutinized_not_needed_violations: 1", proc.stdout)
        self.assertIn("workflow.files-changed-summary", proc.stdout)

    def test_end_allows_files_changed_summary_not_needed_for_docs_only_cycle(self) -> None:
        self.write_agents_md()
        (self.root / "docs").mkdir(parents=True, exist_ok=True)
        (self.root / "docs" / "guide.md").write_text("# Guide\n", encoding="utf-8")
        self.init_git_repo()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "docs" / "guide.md").write_text("# Guide\n\nupdated\n", encoding="utf-8")
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "-",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("scrutinized_not_needed_violations: 0", proc.stdout)

    def test_end_returns_pending_when_source_change_commit_is_not_pushed(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        self.init_origin_main_remote()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "scripts" / "agent_timestamp.py").write_text(
            (self.root / "scripts" / "agent_timestamp.py").read_text(encoding="utf-8")
            + "\n# source change for push guard\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_timestamp.py")
        self.run_git("commit", "-m", "local source change")
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "X",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("source_push_required: true", proc.stdout)
        self.assertIn("source_push_ok: false", proc.stdout)
        self.assertIn("HEAD is not yet reachable from origin/main", proc.stdout)

    def test_end_allows_source_change_cycle_after_push_to_origin_main(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        self.init_origin_main_remote()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "scripts" / "agent_timestamp.py").write_text(
            (self.root / "scripts" / "agent_timestamp.py").read_text(encoding="utf-8")
            + "\n# source change for push guard success\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_timestamp.py")
        self.run_git("commit", "-m", "pushed source change")
        self.run_git("push", "origin", "HEAD:main")
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "X",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("source_push_required: true", proc.stdout)
        self.assertIn("source_push_ok: true", proc.stdout)
        self.assertIn("HEAD is reachable from origin/main", proc.stdout)

    def test_end_allows_foreign_uncommitted_source_changes_without_files_changed_summary(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        self.init_origin_main_remote()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "scripts" / "agent_timestamp.py").write_text(
            (self.root / "scripts" / "agent_timestamp.py").read_text(encoding="utf-8")
            + "\n# foreign dirty source change\n",
            encoding="utf-8",
        )
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "-",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("scrutinized_not_needed_violations: 0", proc.stdout)
        self.assertIn("source_push_required: false", proc.stdout)
        self.assertIn("no committed source changes detected in the cycle", proc.stdout)

    def test_end_ignores_foreign_uncommitted_source_changes_after_pushed_source_cycle(self) -> None:
        self.write_agents_md()
        self.init_git_repo()
        self.init_origin_main_remote()
        agent_uid = self.prepare_second_batch(role="coding")
        self.write_mailbox(
            agent_uid,
            """# Mailbox for agt_coding

## Work Continuation Handoff

- Status: open
""",
        )
        (self.root / "scripts" / "agent_timestamp.py").write_text(
            (self.root / "scripts" / "agent_timestamp.py").read_text(encoding="utf-8")
            + "\n# pushed source change\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_timestamp.py")
        self.run_git("commit", "-m", "pushed source change")
        self.run_git("push", "origin", "HEAD:main")

        (self.root / "scripts" / "agent_registry.py").write_text(
            (self.root / "scripts" / "agent_registry.py").read_text(encoding="utf-8")
            + "\n# foreign dirty source change\n",
            encoding="utf-8",
        )

        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "X",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("source_push_required: true", proc.stdout)
        self.assertIn("source_push_ok: true", proc.stdout)
        self.assertIn("HEAD is reachable from origin/main", proc.stdout)


if __name__ == "__main__":
    unittest.main()
