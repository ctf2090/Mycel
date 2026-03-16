import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_checklist_gc.py"


class AgentChecklistGcCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "agents").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "agent_checklist_gc.py")
        (self.root / "scripts" / "agent_checklist_gc.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_checklist_gc.py"), *args],
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

    def write_checklist(self, relative_path: str) -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("# checklist\n", encoding="utf-8")

    def registry_entry(self, agent_uid: str) -> dict:
        return {
            "agent_uid": agent_uid,
            "role": "coding",
            "current_display_id": None,
            "display_history": [],
            "assigned_by": "user",
            "assigned_at": "2026-03-12T12:00:00+0800",
            "confirmed_by_agent": True,
            "confirmed_at": "2026-03-12T12:00:00+0800",
            "last_touched_at": "2026-03-12T12:00:00+0800",
            "inactive_at": "2026-03-12T12:00:00+0800",
            "paused_at": None,
            "status": "inactive",
            "scope": "checklist-gc-test",
            "files": [],
            "mailbox": f".agent-local/mailboxes/{agent_uid}.md",
            "recovery_of": None,
            "superseded_by": None,
        }

    def test_scan_reports_old_workcycle_batches_beyond_keep_limit(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T12:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry("agt_live")],
            }
        )
        self.write_checklist(".agent-local/agents/agt_live/checklists/AGENTS-bootstrap-checklist.md")
        for batch in range(1, 24):
            self.write_checklist(
                f".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-{batch}.md"
            )

        payload = json.loads(
            self.run_cli("scan", "--keep-workcycle-batches", "20", "--json").stdout
        )

        self.assertEqual(1, payload["referenced_agent_count"])
        self.assertEqual(3, payload["prune_candidate_count"])
        self.assertEqual(
            [
                ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-1.md",
                ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-2.md",
                ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-3.md",
            ],
            [record["path"] for record in payload["prune_candidates"]],
        )

    def test_prune_keeps_recent_batches_and_bootstrap(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T12:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry("agt_live")],
            }
        )
        self.write_checklist(".agent-local/agents/agt_live/checklists/AGENTS-bootstrap-checklist.md")
        for batch in range(1, 24):
            self.write_checklist(
                f".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-{batch}.md"
            )

        payload = json.loads(
            self.run_cli("prune", "--keep-workcycle-batches", "20", "--json").stdout
        )

        self.assertEqual(3, payload["deleted_count"])
        self.assertFalse(
            (self.root / ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-1.md").exists()
        )
        self.assertFalse(
            (self.root / ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-2.md").exists()
        )
        self.assertFalse(
            (self.root / ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-3.md").exists()
        )
        self.assertTrue(
            (self.root / ".agent-local/agents/agt_live/checklists/AGENTS-bootstrap-checklist.md").exists()
        )
        self.assertTrue(
            (self.root / ".agent-local/agents/agt_live/checklists/AGENTS-workcycle-checklist-23.md").exists()
        )


if __name__ == "__main__":
    unittest.main()
