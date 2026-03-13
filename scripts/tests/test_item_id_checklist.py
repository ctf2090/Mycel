import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "item_id_checklist.py"


class ItemIdChecklistCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "item_id_checklist.py")
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "item_id_checklist.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_registry(self) -> None:
        (self.root / ".agent-local" / "agents.json").write_text(
            """{
  "version": 2,
  "updated_at": "2026-03-13T09:00:00+0800",
  "agent_count": 1,
  "agents": [
    {
      "agent_uid": "agt_doc",
      "role": "doc",
      "current_display_id": "doc-1",
      "display_history": [],
      "assigned_by": "user",
      "assigned_at": "2026-03-13T09:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-13T09:00:00+0800",
      "last_touched_at": "2026-03-13T09:00:00+0800",
      "inactive_at": null,
      "paused_at": null,
      "status": "active",
      "scope": "docs",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_doc.md",
      "recovery_of": null,
      "superseded_by": null
    }
  ]
}
""",
            encoding="utf-8",
        )

    def write_source(self, relative_path: str, content: str) -> Path:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_materializes_agent_local_checkbox_copy(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """# Source

- Read the file <!-- item-id: bootstrap.read -->
- [X] Existing checked item <!-- item-id: bootstrap.checked -->
""",
        )

        result = json.loads(self.run_cli("agt_doc", "docs/source.md", "--json").stdout)
        output_path = self.root / result["output"]
        content = output_path.read_text(encoding="utf-8")

        self.assertEqual(".agent-local/agents/agt_doc/checklists/source-checklist.md", result["output"])
        self.assertTrue(output_path.exists())
        self.assertIn("# Agent Item-ID Checklist Copy", content)
        self.assertIn("- [ ] Read the file <!-- item-id: bootstrap.read -->", content)
        self.assertIn("- [ ] Existing checked item <!-- item-id: bootstrap.checked -->", content)
        self.assertIn("update checks here instead of the tracked source file", content)
        self.assertIn("`- [-]` not needed for this work cycle", content)

    def test_keeps_only_item_id_sections_and_items(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """# Source

Intro paragraph that should not be copied.

## Kept Section

- Keep me <!-- item-id: kept.one -->
- Keep me too <!-- item-id: kept.two -->

## Dropped Section

- This plain bullet should not be copied.

### Nested Kept Section

Context line that should not be copied.
- Nested item <!-- item-id: kept.nested -->
""",
        )

        result = json.loads(self.run_cli("agt_doc", "docs/source.md", "--json").stdout)
        content = (self.root / result["output"]).read_text(encoding="utf-8")

        self.assertIn("# Source", content)
        self.assertIn("## Kept Section", content)
        self.assertIn("### Nested Kept Section", content)
        self.assertIn("- [ ] Keep me <!-- item-id: kept.one -->", content)
        self.assertIn("- [ ] Keep me too <!-- item-id: kept.two -->", content)
        self.assertIn("- [ ] Nested item <!-- item-id: kept.nested -->", content)
        self.assertNotIn("Intro paragraph that should not be copied.", content)
        self.assertNotIn("This plain bullet should not be copied.", content)
        self.assertNotIn("Context line that should not be copied.", content)
        self.assertNotIn("## Dropped Section", content)

    def test_preserves_subitem_indentation(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """# Source

## Nested

- Parent bullet
  - Child item <!-- item-id: nested.child -->
""",
        )

        result = json.loads(self.run_cli("agt_doc", "docs/source.md", "--json").stdout)
        content = (self.root / result["output"]).read_text(encoding="utf-8")

        self.assertIn("  - [ ] Child item <!-- item-id: nested.child -->", content)

    def test_agents_checklist_splits_bootstrap_and_workcycle_outputs(self) -> None:
        self.write_registry()
        self.write_source(
            "AGENTS.md",
            """# Repo Working Agreements

## New chat bootstrap
- Bootstrap one <!-- item-id: bootstrap.one -->
- Bootstrap two <!-- item-id: bootstrap.two -->

## Work Cycle Workflow
- Workflow one <!-- item-id: workflow.one -->
- Workflow two <!-- item-id: workflow.two -->
""",
        )

        first = json.loads(self.run_cli("agt_doc", "AGENTS.md", "--json").stdout)
        bootstrap_path = self.root / first["bootstrap_output"]
        workcycle_path = self.root / first["workcycle_output"]
        bootstrap_content = bootstrap_path.read_text(encoding="utf-8")
        workcycle_content = workcycle_path.read_text(encoding="utf-8")

        self.assertEqual(".agent-local/agents/agt_doc/checklists/AGENTS-bootstrap-checklist.md", first["bootstrap_output"])
        self.assertEqual(".agent-local/agents/agt_doc/checklists/AGENTS-workcycle-checklist-1.md", first["workcycle_output"])
        self.assertEqual(1, first["batch_num"])
        self.assertIn("## New chat bootstrap", bootstrap_content)
        self.assertNotIn("## Work Cycle Workflow", bootstrap_content)
        self.assertIn("## Work Cycle Workflow", workcycle_content)
        self.assertNotIn("## New chat bootstrap", workcycle_content)

        second = json.loads(self.run_cli("agt_doc", "AGENTS.md", "--json").stdout)
        second_workcycle_path = self.root / second["workcycle_output"]
        self.assertEqual(".agent-local/agents/agt_doc/checklists/AGENTS-workcycle-checklist-2.md", second["workcycle_output"])
        self.assertEqual(2, second["batch_num"])
        self.assertTrue(second_workcycle_path.exists())
        self.assertTrue(bootstrap_path.exists())

    def test_agents_checklist_rejects_explicit_output_override(self) -> None:
        self.write_registry()
        self.write_source(
            "AGENTS.md",
            """# Repo Working Agreements

## New chat bootstrap
- Bootstrap one <!-- item-id: bootstrap.one -->

## Work Cycle Workflow
- Workflow one <!-- item-id: workflow.one -->
""",
        )

        proc = self.run_cli("agt_doc", "AGENTS.md", "--output", ".agent-local/agents/agt_doc/checklists/custom.md", check=False)

        self.assertNotEqual(0, proc.returncode)
        self.assertIn("AGENTS.md checklist generation manages its own bootstrap/workcycle filenames", proc.stderr)

    def test_accepts_display_id_as_agent_ref(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """- Review docs <!-- item-id: bootstrap.review -->""",
        )

        proc = self.run_cli("doc-1", "docs/source.md")

        self.assertIn("agent_uid: agt_doc", proc.stdout)
        self.assertIn("item_count: 1", proc.stdout)

    def test_rejects_source_without_item_ids(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """# Source

No markers here.
""",
        )

        proc = self.run_cli("agt_doc", "docs/source.md", check=False)

        self.assertNotEqual(0, proc.returncode)
        self.assertIn("source file has no item-id markers", proc.stderr)

    def test_rejects_output_outside_checklists_dir(self) -> None:
        self.write_registry()
        self.write_source(
            "docs/source.md",
            """- Review docs <!-- item-id: bootstrap.review -->""",
        )

        proc = self.run_cli("agt_doc", "docs/source.md", "--output", ".agent-local/not-here.md", check=False)

        self.assertNotEqual(0, proc.returncode)
        self.assertIn("checklist output must live under .agent-local/agents/agt_doc/", proc.stderr)


if __name__ == "__main__":
    unittest.main()
