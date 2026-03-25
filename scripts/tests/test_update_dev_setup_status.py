import json
import shutil
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "update-dev-setup-status.py"


class UpdateDevSetupStatusCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "update-dev-setup-status.py")
        (self.root / "scripts" / "update-dev-setup-status.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_checker(self, script_body: str) -> None:
        path = self.root / "scripts" / "check-dev-env.py"
        path.write_text(script_body, encoding="utf-8")
        path.chmod(0o755)

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "update-dev-setup-status.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_writes_ready_status_file_when_quick_and_full_checks_pass(self) -> None:
        self.write_checker(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -euo pipefail
                if [[ "${1:-}" == "--full" ]]; then
                  cat <<'EOF'
                {"status":"passed","mode":"full","repo_root":"TEMP_ROOT","checks":[{"kind":"command","name":"cargo","status":"found","detail":"cargo 1.94.0"},{"kind":"component","name":"rustfmt","status":"found","detail":"stable"},{"kind":"validation","name":"fmt","status":"passed","detail":"cargo fmt --all --check"}]}
                EOF
                  exit 0
                fi
                cat <<'EOF'
                {"status":"passed","mode":"quick","repo_root":"TEMP_ROOT","checks":[{"kind":"command","name":"cargo","status":"found","detail":"cargo 1.94.0"},{"kind":"command","name":"gh","status":"found","detail":"gh 2.83.1"},{"kind":"component","name":"rustfmt","status":"found","detail":"stable"},{"kind":"component","name":"clippy","status":"found","detail":"stable"}]}
                EOF
                """
            ).replace("TEMP_ROOT", str(self.root))
        )

        proc = self.run_cli("--actor", "doc-6", "--json")
        payload = json.loads(proc.stdout)
        status_path = self.root / ".agent-local" / "dev-setup-status.md"
        content = status_path.read_text(encoding="utf-8")

        self.assertEqual(0, proc.returncode)
        self.assertEqual("ready", payload["status"])
        self.assertIn("- Status: ready", content)
        self.assertIn("- Checked by: doc-6", content)
        self.assertIn("`scripts/check-dev-env.py --full --json` (passed)", content)
        self.assertIn("| fmt | passed | `cargo fmt --all --check` |", content)

    def test_writes_not_ready_file_when_full_validation_fails(self) -> None:
        self.write_checker(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -euo pipefail
                if [[ "${1:-}" == "--full" ]]; then
                  cat <<'EOF'
                {"status":"failed","mode":"full","repo_root":"TEMP_ROOT","checks":[{"kind":"command","name":"cargo","status":"found","detail":"cargo 1.94.0"},{"kind":"component","name":"rustfmt","status":"found","detail":"stable"},{"kind":"validation","name":"fmt","status":"failed","detail":"cargo fmt --all --check"}],"error":"validation step failed: fmt"}
                EOF
                  exit 1
                fi
                cat <<'EOF'
                {"status":"passed","mode":"quick","repo_root":"TEMP_ROOT","checks":[{"kind":"command","name":"cargo","status":"found","detail":"cargo 1.94.0"},{"kind":"component","name":"rustfmt","status":"found","detail":"stable"}]}
                EOF
                """
            ).replace("TEMP_ROOT", str(self.root))
        )

        proc = self.run_cli("--actor", "doc-6", "--json", check=False)
        payload = json.loads(proc.stdout)
        status_path = self.root / ".agent-local" / "dev-setup-status.md"
        content = status_path.read_text(encoding="utf-8")

        self.assertEqual(1, proc.returncode)
        self.assertEqual("not-ready", payload["status"])
        self.assertIn("- Status: not-ready", content)
        self.assertIn("- Full validation status: failed", content)
        self.assertIn("- full check: validation step failed: fmt", content)

    def test_rejects_output_path_outside_agent_local(self) -> None:
        self.write_checker(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -euo pipefail
                cat <<'EOF'
                {"status":"passed","mode":"quick","repo_root":"TEMP_ROOT","checks":[{"kind":"command","name":"cargo","status":"found","detail":"cargo 1.94.0"}]}
                EOF
                """
            ).replace("TEMP_ROOT", str(self.root))
        )

        proc = self.run_cli("--output", "../dev-setup-status.md", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("output path must live under .agent-local/", proc.stderr)


if __name__ == "__main__":
    unittest.main()
