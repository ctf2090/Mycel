import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "codespaces_storage_gc.py"


class CodespacesStorageGcCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "codespaces_storage_gc.py")
        (self.root / "scripts" / "codespaces_storage_gc.py").chmod(0o755)
        self.workspace = self.root / "workspace"
        self.workspace.mkdir()
        self.home = self.root / "home"
        self.home.mkdir()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_bytes(self, relative_path: str, size: int) -> Path:
        path = self.workspace / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * size)
        return path

    def write_home_bytes(self, relative_path: str, size: int) -> Path:
        path = self.home / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * size)
        return path

    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["HOME"] = str(self.home)
        return subprocess.run(
            [sys.executable, str(self.root / "scripts" / "codespaces_storage_gc.py"), *args],
            cwd=self.workspace,
            text=True,
            capture_output=True,
            env=env,
        )

    def test_dry_run_defaults_to_workspace_targets_only(self) -> None:
        self.write_bytes("target/debug/app", 4096)
        self.write_home_bytes(".cargo/registry/cache/index.crate", 2048)

        proc = self.run_cli("--json")

        self.assertEqual(0, proc.returncode)
        payload = json.loads(proc.stdout)
        target_rows = {row["key"]: row for row in payload["targets"]}
        self.assertIn("repo-target", target_rows)
        self.assertNotIn("cargo-registry-cache", target_rows)
        self.assertEqual("present", target_rows["repo-target"]["status"])
        self.assertGreaterEqual(target_rows["repo-target"]["size_bytes"], 4096)

    def test_include_home_caches_adds_home_targets(self) -> None:
        self.write_bytes("target/debug/app", 1024)
        self.write_home_bytes(".npm/_cacache/blob.bin", 3072)

        proc = self.run_cli("--json", "--include-home-caches")

        self.assertEqual(0, proc.returncode)
        payload = json.loads(proc.stdout)
        target_rows = {row["key"]: row for row in payload["targets"]}
        self.assertIn("npm-cache", target_rows)
        self.assertEqual("present", target_rows["npm-cache"]["status"])
        self.assertGreaterEqual(target_rows["npm-cache"]["size_bytes"], 3072)

    def test_apply_removes_selected_target_only(self) -> None:
        repo_file = self.write_bytes("target/debug/app", 4096)
        home_file = self.write_home_bytes(".cargo/registry/cache/index.crate", 2048)

        proc = self.run_cli("--apply", "--target", "repo-target")

        self.assertEqual(0, proc.returncode)
        self.assertFalse(repo_file.exists())
        self.assertTrue(home_file.exists())
        self.assertIn("repo-target\tremoved\tremoved", proc.stdout)

    def test_missing_workspace_exits_nonzero(self) -> None:
        proc = self.run_cli("--workspace", str(self.workspace / "missing"))

        self.assertEqual(2, proc.returncode)
        self.assertIn("workspace does not exist", proc.stderr)


if __name__ == "__main__":
    unittest.main()
