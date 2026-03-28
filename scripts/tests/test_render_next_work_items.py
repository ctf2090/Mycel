import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "render_next_work_items.py"


class RenderNextWorkItemsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        script = scripts_dir / "render_next_work_items.py"
        script.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        script.chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(
        self, *args: str, stdin_text: str = "", check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "render_next_work_items.py"), *args],
            cwd=self.root,
            text=True,
            input=stdin_text,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_renders_numbered_items_and_marks_first_as_highest_value(self) -> None:
        spec = {
            "items": [
                {
                    "text": "continue the M3 governance tooling slice",
                    "tradeoff": "best roadmap alignment, but it keeps us in the current surface area",
                    "roadmap": "M3 / Phase 2",
                },
                {
                    "text": "review the latest CQH issue",
                    "tradeoff": "usually cheaper to land, but less directly tied to the roadmap",
                },
            ]
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertEqual(
            "1. (最有價值) continue the M3 governance tooling slice Tradeoff: best roadmap alignment, "
            "but it keeps us in the current surface area Roadmap: M3 / Phase 2\n"
            "2. review the latest CQH issue Tradeoff: usually cheaper to land, but less directly tied to the roadmap\n",
            proc.stdout,
        )

    def test_prepends_compaction_item_when_compaction_is_detected(self) -> None:
        spec = {
            "compaction_detected": True,
            "items": [
                {
                    "text": "continue with the next coding slice",
                    "tradeoff": "fastest way back into implementation, but only after context is safe again",
                }
            ],
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertEqual(
            "1. (最有價值) compaction detected, we better open a new chat. Tradeoff: safest follow-up after compaction, "
            "but it pauses immediate work until a fresh chat is open.\n"
            "2. continue with the next coding slice Tradeoff: fastest way back into implementation, but only after context is safe again\n",
            proc.stdout,
        )

    def test_accepts_compaction_only_output_without_other_items(self) -> None:
        spec = {"compaction_detected": True}

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertIn("1. (最有價值) compaction detected, we better open a new chat.", proc.stdout)

    def test_rejects_empty_spec_without_items_or_compaction(self) -> None:
        proc = self.run_cli("-", stdin_text=json.dumps({}), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("at least one item or set compaction_detected=true", proc.stderr)


if __name__ == "__main__":
    unittest.main()
