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
SOURCE_MAILBOX_HANDOFF = REPO_ROOT / "scripts" / "mailbox_handoff.py"
SOURCE_AGENT_GUARD = REPO_ROOT / "scripts" / "agent_guard.py"
SOURCE_CHECKLIST = REPO_ROOT / "scripts" / "item_id_checklist.py"
SOURCE_MARKER = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"
SOURCE_NEXT_WORK_ITEMS = REPO_ROOT / "scripts" / "render_next_work_items.py"


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
        shutil.copy2(SOURCE_MAILBOX_HANDOFF, self.root / "scripts" / "mailbox_handoff.py")
        shutil.copy2(SOURCE_AGENT_GUARD, self.root / "scripts" / "agent_guard.py")
        shutil.copy2(SOURCE_CHECKLIST, self.root / "scripts" / "item_id_checklist.py")
        shutil.copy2(SOURCE_MARKER, self.root / "scripts" / "item_id_checklist_mark.py")
        shutil.copy2(SOURCE_NEXT_WORK_ITEMS, self.root / "scripts" / "render_next_work_items.py")
        (self.root / "scripts" / "agent_work_cycle.py").chmod(0o755)
        (self.root / "scripts" / "agent_registry.py").chmod(0o755)
        (self.root / "scripts" / "agent_timestamp.py").chmod(0o755)
        (self.root / "scripts" / "codex_token_usage_summary.py").chmod(0o755)
        (self.root / "scripts" / "agent_checklist_gc.py").chmod(0o755)
        (self.root / "scripts" / "mailbox_gc.py").chmod(0o755)
        (self.root / "scripts" / "mailbox_handoff.py").chmod(0o755)
        (self.root / "scripts" / "agent_guard.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist_mark.py").chmod(0o755)
        (self.root / "scripts" / "render_next_work_items.py").chmod(0o755)

    def tearDown(self) -> None:
        self.remote_temp_dir.cleanup()
        self.temp_dir.cleanup()

    def write_fake_codex_thread_metadata(
        self,
        *,
        model: str = "gpt-5.4",
        effort: str = "medium",
        thread_id: str = "019d23a1-c85f-7d53-a4bb-075ea6504302",
    ) -> None:
        path = self.root / "scripts" / "codex_thread_metadata.py"
        path.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "if '--shell' in sys.argv:\n"
            f"    print('MODEL=\"{model}\"')\n"
            f"    print('EFFORT=\"{effort}\"')\n"
            f"    print('THREAD_ID=\"{thread_id}\"')\n"
            "    print('STATE_DB=\"/tmp/test.sqlite\"')\n"
            "else:\n"
            f"    print('model: {model}')\n"
            f"    print('effort: {effort}')\n",
            encoding="utf-8",
        )
        path.chmod(0o755)

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

    def commit_as_agent(self, agent_uid: str, message: str) -> None:
        self.run_git(
            "-c",
            f"user.name=gpt-5.4:{agent_uid}",
            "-c",
            "user.email=ctf2090+mycel@gmail.com",
            "commit",
            "--no-gpg-sign",
            "-m",
            (
                f"{message}\n\n"
                f"Agent-Id: {agent_uid}\n"
                "Model: gpt-5.4\n"
                "Reasoning-Effort: medium"
            ),
        )

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
- Run bootstrap runtime preflight <!-- item-id: bootstrap.runtime-preflight -->

## Work Cycle Workflow
- Run git status <!-- item-id: bootstrap.git-status -->
- Begin the work cycle <!-- item-id: workflow.touch-work-cycle -->
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

    def write_agents_local(self, locale: str = "zh-TW") -> None:
        (self.root / "AGENTS-LOCAL.md").write_text(
            "# AGENTS-LOCAL.md\n\n"
            "## Communication\n\n"
            f"- Respond to the user in Traditional Chinese (`{locale}`) by default unless the user explicitly asks for another language.\n",
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

    def write_registry(self, payload: dict[str, object]) -> None:
        path = self.root / ".agent-local" / "agents.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    def load_next_work_items_spec(self, agent_uid: str, batch_num: int) -> dict[str, object]:
        path = self.root / f".agent-local/agents/{agent_uid}/workcycles/next-work-items-{batch_num}.json"
        return json.loads(path.read_text(encoding="utf-8"))

    def load_next_work_items_markdown(self, agent_uid: str, batch_num: int) -> str:
        path = self.root / f".agent-local/agents/{agent_uid}/workcycles/next-work-items-{batch_num}.md"
        return path.read_text(encoding="utf-8")

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

    def mark_bootstrap_defaults(self, relative_path: str) -> None:
        states = [
            ("bootstrap.one", "Bootstrap one", "X"),
            ("bootstrap.runtime-preflight", "Run bootstrap runtime preflight", "X"),
        ]
        for item_id, label, state in states:
            self.set_checklist_state(relative_path, item_id, state, label)

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
        model_context_window: int = 258400,
        compaction_event_type: str | None = None,
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
                                "model_context_window": model_context_window,
                            },
                        },
                    }
                )
            )
        if compaction_event_type is not None:
            lines.append(
                json.dumps(
                    {
                        "timestamp": "2026-03-25T06:26:03.000Z",
                        "type": compaction_event_type,
                        "encrypted_content": "test-compaction-payload",
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
        self.mark_bootstrap_defaults(start["bootstrap_output"])
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

    def load_owned_paths(self, agent_uid: str, batch_num: int) -> list[str]:
        path = self.root / f".agent-local/agents/{agent_uid}/workcycles/owned-paths-{batch_num}.json"
        payload = json.loads(path.read_text(encoding="utf-8"))
        return payload["paths"]

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

    def test_record_paths_updates_latest_active_batch_owned_paths_snapshot(self) -> None:
        self.write_agents_md()
        agent_uid = self.prepare_second_batch(role="doc")
        guide_path = self.root / "docs" / "guide.md"
        guide_path.parent.mkdir(parents=True, exist_ok=True)
        guide_path.write_text("# Guide\n", encoding="utf-8")

        proc = self.run_cli(
            "record-paths",
            agent_uid,
            str(guide_path),
            "./docs/guide.md",
            ".agent-local/ignored.md",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("batch_num: 2", proc.stdout)
        self.assertIn("recorded_paths: 1", proc.stdout)
        self.assertEqual(["docs/guide.md"], self.load_owned_paths(agent_uid, 2))

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
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
        self.write_fake_codex_thread_metadata()
        claim = self.run_registry("claim", "delivery", "--scope", "ci-triage", "--model-id", "claude-sonnet-4-6")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)

        proc = self.run_cli("begin", agent_uid, "--scope", "ci-triage")

        self.assertIn(f"Before work | delivery-1 ({agent_uid}/gpt-5.4/medium) | ci-triage", proc.stdout)

    def test_begin_omits_token_usage_when_available(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
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
            f"Before work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper",
            proc.stdout,
        )
        self.assertNotIn("last thread turn:", proc.stdout)

    def test_begin_prefers_current_thread_id_over_newer_same_cwd_rollout(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata(thread_id="019d23a1-c85f-7d53-a4bb-075ea6504302")
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

        self.assertIn(f"Before work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper", proc.stdout)
        self.assertNotIn("last thread turn:", proc.stdout)
        self.assertNotIn("500,000 tok", proc.stdout)

    def test_begin_aborts_and_writes_handoff_when_compaction_detected(self) -> None:
        self.write_agents_md()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
            compaction_event_type="compaction",
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
            check=False,
        )

        self.assertEqual(3, proc.returncode)
        self.assertIn("compact_context_detected: true", proc.stdout)
        self.assertIn("alert: compact context detected, we better open a new chat for better performance, and handoff is ready.", proc.stdout)
        self.assertNotIn("Before work |", proc.stdout)

        status = self.run_registry("status", agent_uid)
        self.assertEqual("inactive", status["agents"][0]["status"])

        mailbox = (self.root / ".agent-local" / "mailboxes" / f"{agent_uid}.md").read_text(encoding="utf-8")
        self.assertIn("## Doc Continuation Note", mailbox)
        self.assertIn("Compact context detected in the current chat thread before work started", mailbox)
        self.assertIn("Open a fresh chat for better performance and continue from this handoff.", mailbox)
        block_state = json.loads(
            (self.root / ".agent-local" / "runtime" / "agent-blocks.json").read_text(encoding="utf-8")
        )
        self.assertTrue(block_state["blocks"][agent_uid]["blocked"])
        self.assertEqual("compact_context_detected", block_state["blocks"][agent_uid]["reason"])

    def test_begin_aborts_when_compacted_event_is_detected_on_metadata_thread(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata(thread_id="019d23a1-c85f-7d53-a4bb-075ea6504302")
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
            compaction_event_type="compacted",
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
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504303"},
            check=False,
        )

        self.assertEqual(3, proc.returncode)
        self.assertIn("compact_context_detected: true", proc.stdout)
        self.assertIn(
            "compaction_rollout_path: "
            f"{self.root / '.codex' / 'sessions' / '2026' / '03' / '25' / 'rollout-2026-03-25T06-14-58-019d23a1-c85f-7d53-a4bb-075ea6504302.jsonl'}",
            proc.stdout,
        )

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

    def test_end_refuses_normal_closeout_when_agent_is_blocked(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "blocked-closeout", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "blocked-closeout")
        runtime_dir = self.root / ".agent-local" / "runtime"
        runtime_dir.mkdir(parents=True, exist_ok=True)
        (runtime_dir / "agent-blocks.json").write_text(
            json.dumps(
                {
                    "version": 1,
                    "blocks": {
                        agent_uid: {
                            "blocked": True,
                            "reason": "compact_context_detected",
                            "detected_at": "2026-03-25T15:28:43.925Z",
                            "source": "agent_work_cycle.begin",
                            "handoff_path": f".agent-local/mailboxes/{agent_uid}.md",
                            "clear_requires": "new_chat_bootstrap",
                        }
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "blocked-closeout", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("--blocked-closeout", proc.stderr)

    def test_begin_explains_recoverable_display_slot_loss_is_not_guard_block(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "recover-needed", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        registry_path = self.root / ".agent-local" / "agents.json"
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        registry["agents"][0]["current_display_id"] = None
        registry["agents"][0]["display_history"][0]["released_at"] = "2026-03-25T15:40:00+0800"
        registry["agents"][0]["display_history"][0]["released_reason"] = "stale_release"
        self.write_registry(registry)

        proc = self.run_cli("begin", agent_uid, "--scope", "recover-needed", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("has no active display_id; recover it before touch", proc.stderr)
        self.assertIn("display-slot recovery problem", proc.stderr)
        self.assertIn("not by itself a compact_context_detected guard block", proc.stderr)

    def test_blocked_closeout_rejection_explains_guard_precondition(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "blocked-closeout", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "blocked-closeout")

        proc = self.run_cli("end", agent_uid, "--scope", "blocked-closeout", "--blocked-closeout", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("blocked closeout is only valid when `agent_guard.py check` reports `blocked: true`", proc.stderr)
        self.assertIn(f"use `python3 scripts/agent_work_cycle.py end {agent_uid}` instead.", proc.stderr)

    def test_blocked_closeout_succeeds_without_normal_checklist_completion(self) -> None:
        self.write_agents_md()
        claim = self.run_registry("claim", "doc", "--scope", "blocked-closeout", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        self.run_registry("start", agent_uid)
        self.run_cli("begin", agent_uid, "--scope", "blocked-closeout")
        runtime_dir = self.root / ".agent-local" / "runtime"
        runtime_dir.mkdir(parents=True, exist_ok=True)
        (runtime_dir / "agent-blocks.json").write_text(
            json.dumps(
                {
                    "version": 1,
                    "blocks": {
                        agent_uid: {
                            "blocked": True,
                            "reason": "compact_context_detected",
                            "detected_at": "2026-03-25T15:28:43.925Z",
                            "source": "agent_work_cycle.begin",
                            "handoff_path": f".agent-local/mailboxes/{agent_uid}.md",
                            "clear_requires": "new_chat_bootstrap",
                        }
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        self.write_mailbox(
            agent_uid,
            (
                f"# Mailbox for {agent_uid}\n\n"
                "## Doc Continuation Note\n\n"
                "- Status: open\n"
                "- Date: 2026-03-25 15:30 UTC+8\n"
                "- Source agent: doc-1\n"
                "- Source role: doc\n"
                "- Scope: blocked-closeout\n"
                "- Current state:\n"
                "  - Compact context detected.\n"
                "- Next suggested step:\n"
                "  - Open a fresh chat.\n"
            ),
        )

        proc = self.run_cli("end", agent_uid, "--scope", "blocked-closeout", "--blocked-closeout")

        self.assertEqual(0, proc.returncode)
        self.assertIn("blocked_closeout: true", proc.stdout)
        self.assertIn("blocked_closeout_reason: compact_context_detected", proc.stdout)
        self.assertIn("unchecked_items: 12", proc.stdout)

    def test_end_appends_estimated_cycle_token_usage_when_available(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K",
            proc.stdout,
        )
        self.assertIn(
            f"next_work_items_spec: .agent-local/agents/{agent_uid}/workcycles/next-work-items-1.json",
            proc.stdout,
        )
        self.assertIn(
            f"next_work_items_markdown: .agent-local/agents/{agent_uid}/workcycles/next-work-items-1.md",
            proc.stdout,
        )
        self.assertIn(
            f"next_work_items_render_command: python3 scripts/render_next_work_items.py .agent-local/agents/{agent_uid}/workcycles/next-work-items-1.json",
            proc.stdout,
        )
        self.assertIn("next_work_items_paste_rule: paste the rendered Markdown verbatim after the After work line;", proc.stdout)
        self.assertEqual(
            {"compaction_detected": False, "role": "doc"},
            self.load_next_work_items_spec(agent_uid, 1),
        )
        self.assertEqual(
            "1. (最有價值) review the freshest planning or documentation follow-up before choosing the next doc item "
            "Tradeoff: keeps doc work aligned with current repo state, but it adds a short review step first\n"
            "2. check whether planning-sync or issue-sync follow-up is due before writing the next doc update "
            "Tradeoff: helps avoid drift in planning surfaces, but it may defer narrower writing work briefly\n",
            self.load_next_work_items_markdown(agent_uid, 1),
        )

    def test_end_reuses_frozen_end_token_snapshot_on_repeat_closeout(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K",
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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K",
            second_end.stdout,
        )
        self.assertNotIn("98,000 tok", second_end.stdout)

    def test_end_prefers_latest_open_same_role_handoff_next_step_in_next_work_items(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "coding", "--scope", "hotspot follow-up", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

        begin = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "hotspot follow-up",
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
        self.write_mailbox(
            agent_uid,
            (
                f"# Mailbox for {agent_uid}\n\n"
                "## Work Continuation Handoff\n\n"
                "- Status: open\n"
                "- Date: 2026-03-25 15:30 UTC+8\n"
                "- Source agent: coding-1\n"
                "- Source role: coding\n"
                "- Scope: hotspot follow-up\n"
                "- Current state:\n"
                "  - The hotspot checker already accepts repo-relative file or directory paths.\n"
                "- Next suggested step:\n"
                "  - tighten docs/ROLE-CHECKLISTS/coding.md so hotspot scans target touched files by default.\n"
                "- Blockers:\n"
                "  - none\n"
            ),
        )

        proc = self.run_cli(
            "end",
            agent_uid,
            "--scope",
            "hotspot follow-up",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )

        self.assertEqual(0, proc.returncode)
        self.assertEqual(
            {
                "append_role_defaults": True,
                "compaction_detected": False,
                "items": [
                    {
                        "text": "tighten docs/ROLE-CHECKLISTS/coding.md so hotspot scans target touched files by default.",
                        "tradeoff": (
                            "builds on the latest confirmed state (The hotspot checker already accepts "
                            "repo-relative file or directory paths.), but it may still need a quick implementation pass to land cleanly"
                        ),
                    }
                ],
                "role": "coding",
            },
            self.load_next_work_items_spec(agent_uid, 1),
        )
        self.assertEqual(
            "1. (最有價值) tighten docs/ROLE-CHECKLISTS/coding.md so hotspot scans target touched files by default. "
            "Tradeoff: builds on the latest confirmed state (The hotspot checker already accepts repo-relative file or directory paths.), "
            "but it may still need a quick implementation pass to land cleanly\n"
            "2. review ROADMAP.md and identify the highest-value next coding work Tradeoff: best roadmap alignment, "
            "but it spends a little time on prioritization before implementation Roadmap: ROADMAP.md / next coding slice\n"
            "3. review the latest CQH issue and identify high-value work items Tradeoff: usually cheaper to land quickly, "
            "but it may be less directly tied to the main roadmap lane\n",
            self.load_next_work_items_markdown(agent_uid, 1),
        )

    def test_end_uses_repo_local_locale_for_next_work_items(self) -> None:
        self.write_agents_md()
        self.write_agents_local("zh-TW")
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
        self.assertEqual(
            {"compaction_detected": False, "locale": "zh-TW", "role": "doc"},
            self.load_next_work_items_spec(agent_uid, 1),
        )
        self.assertEqual(
            "1. (最有價值) 檢查最新的規劃或文件後續項，再決定下一個 doc 工作 取捨: "
            "能讓文件工作保持和目前 repo 狀態同步，但會先多一個短暫的檢查步驟\n"
            "2. 先確認 planning-sync 或 issue-sync 的後續是否到期，再撰寫下一份文件更新 取捨: "
            "有助於避免規劃面漂移，但可能會先稍微延後較窄範圍的寫作工作\n",
            self.load_next_work_items_markdown(agent_uid, 1),
        )

    def test_end_skips_english_handoff_items_when_locale_prefers_zh_tw(self) -> None:
        self.write_agents_md()
        self.write_agents_local("zh-TW")
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "coding", "--scope", "locale-fallback", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

        begin = self.run_cli(
            "begin",
            agent_uid,
            "--scope",
            "locale-fallback",
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
        self.write_mailbox(
            agent_uid,
            (
                f"# Mailbox for {agent_uid}\n\n"
                "## Work Continuation Handoff\n\n"
                "- Status: open\n"
                "- Date: 2026-03-25 15:30 UTC+8\n"
                "- Source agent: coding-1\n"
                "- Source role: coding\n"
                "- Scope: locale-fallback\n"
                "- Current state:\n"
                "  - origin/main now renders next-work-items in zh-TW by default.\n"
                "- Next suggested step:\n"
                "  - If we want fully localized closeouts, emit zh-TW handoff next-step text too.\n"
                "- Blockers:\n"
                "  - none\n"
            ),
        )

        proc = self.run_cli(
            "end",
            agent_uid,
            "--scope",
            "locale-fallback",
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504302"},
        )

        self.assertEqual(0, proc.returncode)
        self.assertEqual(
            {"compaction_detected": False, "locale": "zh-TW", "role": "coding"},
            self.load_next_work_items_spec(agent_uid, 1),
        )
        self.assertEqual(
            "1. (最有價值) 檢查 ROADMAP.md，找出最高價值的下一個 coding 工作 取捨: "
            "和 roadmap 對齊最好，但在開始實作前需要先花一點時間做優先順序判斷 路線圖: ROADMAP.md / next coding slice\n"
            "2. 檢查最新的 CQH issue，整理高價值工作項目 取捨: 通常比較快能落地，但可能沒有那麼直接貼近主要 roadmap 軌道\n",
            self.load_next_work_items_markdown(agent_uid, 1),
        )

    def test_after_work_uses_compact_k_format_for_large_cycle_estimates(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
                ("2026-03-25T06:25:03.000Z", 45000, 2084574),
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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K",
            proc.stdout,
        )

    def test_end_alerts_when_compaction_detected_after_begin(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
            compaction_event_type="compaction",
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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K | pre-boot +60K | compaction detected",
            proc.stdout,
        )
        self.assertIn("compact_context_detected_before_after_work: true", proc.stdout)
        self.assertIn("compaction_timestamp: 2026-03-25T06:26:03.000Z", proc.stdout)
        self.assertIn(
            "alert: compact context detected before after-work closeout; open a fresh chat before continuing.",
            proc.stdout,
        )
        self.assertEqual(
            {"compaction_detected": True, "role": "doc"},
            self.load_next_work_items_spec(agent_uid, 1),
        )
        self.assertEqual(
            "1. (最有價值) compaction detected, we better open a new chat. Tradeoff: safest follow-up after compaction, "
            "but it pauses immediate work until a fresh chat is open.\n"
            "2. review the freshest planning or documentation follow-up before choosing the next doc item "
            "Tradeoff: keeps doc work aligned with current repo state, but it adds a short review step first\n"
            "3. check whether planning-sync or issue-sync follow-up is due before writing the next doc update "
            "Tradeoff: helps avoid drift in planning surfaces, but it may defer narrower writing work briefly\n",
            self.load_next_work_items_markdown(agent_uid, 1),
        )

    def test_end_alerts_when_compaction_detected_on_begin_thread_after_thread_switch(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata(
            thread_id="019d23a1-c85f-7d53-a4bb-075ea6504302"
        )
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 60135, 887020)],
        )
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504303",
            totals=[("2026-03-25T06:27:03.000Z", 45000, 932020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
            compaction_event_type="compaction",
        )
        self.write_fake_codex_thread_metadata(
            thread_id="019d23a1-c85f-7d53-a4bb-075ea6504303"
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
            extra_env={"CODEX_THREAD_ID": "019d23a1-c85f-7d53-a4bb-075ea6504303"},
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn(
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 45K/258K | pre-boot +60K | compaction detected",
            proc.stdout,
        )
        self.assertIn("compact_context_detected_before_after_work: true", proc.stdout)
        self.assertIn("compaction_timestamp: 2026-03-25T06:26:03.000Z", proc.stdout)
        self.assertIn(
            "rollout-2026-03-25T06-14-58-019d23a1-c85f-7d53-a4bb-075ea6504302.jsonl",
            proc.stdout,
        )
        self.assertIn("thread_switch_detected_before_after_work: true", proc.stdout)
        self.assertIn(
            "begin_thread_id: 019d23a1-c85f-7d53-a4bb-075ea6504302",
            proc.stdout,
        )
        self.assertIn(
            "end_thread_id: 019d23a1-c85f-7d53-a4bb-075ea6504303",
            proc.stdout,
        )
        self.assertIn(
            "warning: begin/end Codex thread ids differ during after-work closeout; diagnostics may span a thread switch.",
            proc.stdout,
        )

    def test_after_work_estimates_token_spent_from_ui_usage_delta(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 100000, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
                ("2026-03-25T06:20:03.000Z", 100000, 887020),
                ("2026-03-25T06:25:03.000Z", 150000, 2084574),
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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 150K/258K | +50K this cycle est.",
            proc.stdout,
        )

    def test_bootstrap_batch_after_work_reports_pre_boot_usage(self) -> None:
        self.write_agents_md()
        self.write_fake_codex_thread_metadata()
        self.write_codex_rollout(
            "019d23a1-c85f-7d53-a4bb-075ea6504302",
            totals=[("2026-03-25T06:20:03.000Z", 50000, 887020)],
        )
        claim = self.run_registry("claim", "doc", "--scope", "timestamp-wrapper", "--model-id", "gpt-5.4")
        agent_uid = claim["agent_uid"]
        start = self.run_registry("start", agent_uid)
        self.mark_bootstrap_defaults(start["bootstrap_output"])

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
                ("2026-03-25T06:20:03.000Z", 50000, 887020),
                ("2026-03-25T06:25:03.000Z", 58000, 932020),
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
            f"After work | doc-1 ({agent_uid}/gpt-5.4/medium) | timestamp-wrapper | usage 58K/258K | +8K this cycle est. | pre-boot +50K",
            proc.stdout,
        )

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
        self.commit_as_agent(agent_uid, "committed source change")
        record_proc = self.run_cli("record-paths", agent_uid, "scripts/agent_timestamp.py")
        self.assertEqual(0, record_proc.returncode)
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

    def test_end_requires_record_paths_for_cycle_owned_committed_source_changes(self) -> None:
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
            + "\n# committed without record-paths\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_timestamp.py")
        self.commit_as_agent(agent_uid, "committed source change without record-paths")
        self.run_git("push", "origin", "HEAD:main")
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
        self.assertIn("missing record-paths entries", proc.stdout)
        self.assertIn("scripts/agent_timestamp.py", proc.stdout)

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
        self.commit_as_agent(agent_uid, "local source change")
        record_proc = self.run_cli("record-paths", agent_uid, "scripts/agent_timestamp.py")
        self.assertEqual(0, record_proc.returncode)
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
        self.commit_as_agent(agent_uid, "pushed source change")
        record_proc = self.run_cli("record-paths", agent_uid, "scripts/agent_timestamp.py")
        self.assertEqual(0, record_proc.returncode)
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

    def test_end_allows_uncommitted_file_changes_when_cycle_has_no_owned_commits(self) -> None:
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
        (self.root / "docs").mkdir(exist_ok=True)
        (self.root / "docs" / "guide.md").write_text(
            "# Guide\n\nupdated during this cycle\n",
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
        self.assertIn("source_push_required: false", proc.stdout)
        self.assertIn("source_push_ok: true", proc.stdout)
        self.assertIn(
            "no cycle-owned tracked-file changes detected; local-only changes do not block closeout",
            proc.stdout,
        )

    def test_end_returns_pending_when_recorded_owned_file_changes_are_uncommitted(self) -> None:
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
        (self.root / "docs").mkdir(exist_ok=True)
        guide_path = self.root / "docs" / "guide.md"
        guide_path.write_text("# Guide\n\nowned change during this cycle\n", encoding="utf-8")
        record_proc = self.run_cli("record-paths", agent_uid, "docs/guide.md")
        self.assertEqual(0, record_proc.returncode)
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
        self.assertIn(
            "cycle-owned tracked-file changes are still uncommitted; commit and push them first",
            proc.stdout,
        )

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
        self.commit_as_agent(agent_uid, "pushed source change")
        record_proc = self.run_cli("record-paths", agent_uid, "scripts/agent_timestamp.py")
        self.assertEqual(0, record_proc.returncode)
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

    def test_end_ignores_foreign_committed_source_changes_after_pushed_source_cycle(self) -> None:
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
        self.commit_as_agent(agent_uid, "pushed source change")
        record_proc = self.run_cli("record-paths", agent_uid, "scripts/agent_timestamp.py")
        self.assertEqual(0, record_proc.returncode)
        self.run_git("push", "origin", "HEAD:main")

        (self.root / "scripts" / "agent_registry.py").write_text(
            (self.root / "scripts" / "agent_registry.py").read_text(encoding="utf-8")
            + "\n# foreign committed source change\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_registry.py")
        self.run_git("commit", "-m", "foreign local source change")

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
        self.assertIn(
            "latest cycle-owned source commit is reachable from origin/main",
            proc.stdout,
        )

    def test_end_allows_files_changed_summary_not_needed_when_only_foreign_commits_advance_head(self) -> None:
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
        (self.root / "scripts" / "agent_registry.py").write_text(
            (self.root / "scripts" / "agent_registry.py").read_text(encoding="utf-8")
            + "\n# foreign committed source change\n",
            encoding="utf-8",
        )
        self.run_git("add", "scripts/agent_registry.py")
        self.run_git("commit", "-m", "foreign local source change")
        self.set_checklist_state(
            f".agent-local/agents/{agent_uid}/checklists/AGENTS-workcycle-checklist-2.md",
            "workflow.files-changed-summary",
            "-",
            "Include a files-changed summary when source changes land",
        )

        proc = self.run_cli("end", agent_uid, "--scope", "timestamp-wrapper", check=False)

        self.assertEqual(0, proc.returncode)
        self.assertIn("scrutinized_not_needed_violations: 0", proc.stdout)
        self.assertIn(
            "source_push_reason: no cycle-owned tracked-file changes detected; local-only changes do not block closeout",
            proc.stdout,
        )


if __name__ == "__main__":
    unittest.main()
