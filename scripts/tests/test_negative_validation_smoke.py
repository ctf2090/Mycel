import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SMOKE_SCRIPT = REPO_ROOT / "sim" / "negative-validation" / "smoke.py"


class NegativeValidationSmokeCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "sim" / "negative-validation").mkdir(parents=True, exist_ok=True)
        target = self.root / "sim" / "negative-validation" / "smoke.py"
        target.write_text(SMOKE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)

        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()
        self._write_fake_cargo()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def _write_fake_cargo(self) -> None:
        cargo = self.bin_dir / "cargo"
        cargo.write_text(
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import json
                import sys

                args = sys.argv[1:]
                if args[:5] != ["run", "-p", "mycel-cli", "--", "validate"]:
                    raise SystemExit(1)

                payload = {"status": "ok"}
                exit_code = 0
                target = None
                strict = False
                for arg in args[5:]:
                    if arg == "--json":
                        continue
                    if arg == "--strict":
                        strict = True
                        continue
                    if target is None:
                        target = arg

                if target is None:
                    payload = {"status": "ok"}
                elif "random-seed-prefix-mismatch" in target:
                    payload = {"status": "failed", "message": "seed_source 'random' mismatch"}
                    exit_code = 1
                elif "auto-seed-prefix-mismatch" in target:
                    payload = {"status": "failed", "message": "seed_source 'auto' mismatch"}
                    exit_code = 1
                elif "unknown-topology-reference" in target:
                    payload = {"status": "failed", "message": "does not match any loaded topology"}
                    exit_code = 1
                elif "unknown-fixture-reference" in target:
                    payload = {"status": "failed", "message": "does not match any loaded fixture"}
                    exit_code = 1
                elif "missing-seed-source" in target:
                    payload = {"status": "warning", "message": "does not include seed_source"}
                    exit_code = 1 if strict else 0

                print(json.dumps(payload))
                raise SystemExit(exit_code)
                """
            ),
            encoding="utf-8",
        )
        cargo.chmod(0o755)

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "sim" / "negative-validation" / "smoke.py"), *args],
            cwd=self.root,
            env={"PATH": f"{self.bin_dir}:/usr/bin:/bin"},
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"smoke.py failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_summary_only_runs_full_matrix(self) -> None:
        proc = self.run_cli("--summary-only")

        self.assertEqual(0, proc.returncode)
        self.assertIn("[smoke] summary", proc.stdout)
        self.assertIn("PASS  repo-validate-ok", proc.stdout)
        self.assertIn("PASS  missing-seed-source-strict -> non-zero exit under --strict", proc.stdout)

    def test_unknown_case_returns_exit_code_2(self) -> None:
        proc = self.run_cli("--case", "not-a-case", check=False)

        self.assertEqual(2, proc.returncode)
        self.assertIn("[smoke] unknown case: not-a-case", proc.stderr)


if __name__ == "__main__":
    unittest.main()
