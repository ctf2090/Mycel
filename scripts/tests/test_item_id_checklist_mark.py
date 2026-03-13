import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "item_id_checklist_mark.py"


class ItemIdChecklistMarkCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "item_id_checklist_mark.py")
        (self.root / "scripts" / "item_id_checklist_mark.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "item_id_checklist_mark.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_checklist(self, relative_path: str, content: str) -> Path:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_marks_checked(self) -> None:
        checklist = self.write_checklist(
            ".agent-local/checklists/test.md",
            "- [ ] Do the thing <!-- item-id: workflow.do-thing -->\n",
        )

        payload = json.loads(
            self.run_cli(str(checklist.relative_to(self.root)), "workflow.do-thing", "--json").stdout
        )
        content = checklist.read_text(encoding="utf-8")

        self.assertEqual("checked", payload["state"])
        self.assertIn("- [X] Do the thing <!-- item-id: workflow.do-thing -->", content)

    def test_marks_problem_and_adds_subitem(self) -> None:
        checklist = self.write_checklist(
            ".agent-local/checklists/test.md",
            "- [ ] Do the thing <!-- item-id: workflow.do-thing -->\n",
        )

        proc = self.run_cli(
            str(checklist.relative_to(self.root)),
            "workflow.do-thing",
            "--state",
            "problem",
            "--problem",
            "Latest verification failed",
        )
        content = checklist.read_text(encoding="utf-8")

        self.assertIn("state: problem", proc.stdout)
        self.assertIn("- [!] Do the thing <!-- item-id: workflow.do-thing -->", content)
        self.assertIn("  - Problem: Latest verification failed", content)

    def test_problem_state_requires_problem_text(self) -> None:
        checklist = self.write_checklist(
            ".agent-local/checklists/test.md",
            "- [ ] Do the thing <!-- item-id: workflow.do-thing -->\n",
        )

        proc = self.run_cli(
            str(checklist.relative_to(self.root)),
            "workflow.do-thing",
            "--state",
            "problem",
            check=False,
        )

        self.assertNotEqual(0, proc.returncode)
        self.assertIn("problem state requires --problem", proc.stderr)

    def test_clears_problem_subitem_when_marked_checked(self) -> None:
        checklist = self.write_checklist(
            ".agent-local/checklists/test.md",
            "- [!] Do the thing <!-- item-id: workflow.do-thing -->\n  - Problem: Old problem\n",
        )

        self.run_cli(str(checklist.relative_to(self.root)), "workflow.do-thing", "--state", "checked")
        content = checklist.read_text(encoding="utf-8")

        self.assertIn("- [X] Do the thing <!-- item-id: workflow.do-thing -->", content)
        self.assertNotIn("Problem: Old problem", content)

    def test_rejects_checklist_outside_agent_local(self) -> None:
        checklist = self.write_checklist(
            "docs/test.md",
            "- [ ] Do the thing <!-- item-id: workflow.do-thing -->\n",
        )

        proc = self.run_cli(str(checklist.relative_to(self.root)), "workflow.do-thing", check=False)

        self.assertNotEqual(0, proc.returncode)
        self.assertIn("checklist path must live under .agent-local/checklists/", proc.stderr)


if __name__ == "__main__":
    unittest.main()
