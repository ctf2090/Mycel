import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "check-runtime-preflight.sh"


class CheckRuntimePreflightCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "check-runtime-preflight.sh")
        (self.root / "scripts" / "check-runtime-preflight.sh").chmod(0o755)
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_stub(self, name: str) -> None:
        path = self.bin_dir / name
        path.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        path.chmod(0o755)

    def run_cli(self, *args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
        merged_env = os.environ.copy()
        if env:
            merged_env.update(env)
        return subprocess.run(
            ["/bin/bash", str(self.root / "scripts" / "check-runtime-preflight.sh"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=merged_env,
        )

    def test_passes_with_default_commands_present(self) -> None:
        for cmd in ("cargo", "bash", "rg"):
            self.write_stub(cmd)

        proc = self.run_cli(env={"PATH": str(self.bin_dir)})

        self.assertEqual(0, proc.returncode)
        self.assertIn("runtime preflight passed", proc.stdout)

    def test_blocks_when_required_command_missing(self) -> None:
        for cmd in ("cargo", "bash", "rg"):
            self.write_stub(cmd)

        proc = self.run_cli("--require", "grep", env={"PATH": str(self.bin_dir)})

        self.assertEqual(1, proc.returncode)
        self.assertIn("missing grep", proc.stdout)
        self.assertIn("runtime preflight blocked", proc.stderr)

    def test_json_mode_reports_found_and_missing_commands(self) -> None:
        for cmd in ("cargo", "bash", "rg", "grep"):
            self.write_stub(cmd)

        proc = self.run_cli("--json", "--require", "grep", "--require", "tail", env={"PATH": str(self.bin_dir)})

        self.assertEqual(1, proc.returncode)
        payload = json.loads(proc.stdout)
        checks = {entry["name"]: entry for entry in payload["checks"]}

        self.assertEqual("blocked", payload["status"])
        self.assertEqual("found", checks["grep"]["status"])
        self.assertEqual("missing", checks["tail"]["status"])


if __name__ == "__main__":
    unittest.main()
