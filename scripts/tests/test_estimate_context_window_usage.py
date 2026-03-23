import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "estimate_context_window_usage.py"


class EstimateContextWindowUsageCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        target = scripts_dir / "estimate_context_window_usage.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(
        self, *args: str, stdin_text: str = "", check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "estimate_context_window_usage.py"), *args],
            cwd=self.root,
            text=True,
            input=stdin_text,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_snapshot_mode_reports_ok_status(self) -> None:
        spec = {
            "context_window": 258000,
            "current_input_tokens": 70000,
            "last_output_tokens": 1200,
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertIn("71,200 / 258,000 tokens", proc.stdout)
        self.assertIn("Percent used: 27.6% (72.4% left)", proc.stdout)
        self.assertIn("Status: ok", proc.stdout)
        self.assertIn("Source: snapshot", proc.stdout)

    def test_ledger_mode_reports_prepare_handoff(self) -> None:
        spec = {
            "context_window": 1000,
            "turns": [
                {"added_tokens": 220},
                {"added_tokens": 210},
                {"added_tokens": 190},
            ],
        }

        proc = self.run_cli("--warn-threshold", "60", "--rotate-threshold", "80", "-", stdin_text=json.dumps(spec))

        self.assertIn("620 / 1,000 tokens", proc.stdout)
        self.assertIn("Status: prepare_handoff", proc.stdout)
        self.assertIn("Prepare a handoff note now", proc.stdout)

    def test_json_output_reports_rotate_chat(self) -> None:
        spec = {
            "context_window": 1000,
            "current_input_tokens": 760,
            "last_output_tokens": 40,
        }

        proc = self.run_cli("--json", "-", stdin_text=json.dumps(spec))
        payload = json.loads(proc.stdout)

        self.assertEqual("rotate_chat", payload["status"])
        self.assertEqual(800, payload["used_tokens"])
        self.assertEqual(80.0, payload["used_percent"])
        self.assertEqual(800, payload["raw_used_tokens"])

    def test_additive_calibration_uses_observed_sample_as_reference(self) -> None:
        spec = {
            "context_window": 10000,
            "current_input_tokens": 900,
            "last_output_tokens": 300,
            "calibration": {
                "mode": "additive",
                "estimated_tokens": 300,
                "observed_tokens": 2800,
            },
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertIn("3,700 / 10,000 tokens", proc.stdout)
        self.assertIn("Raw estimate before calibration: 1,200", proc.stdout)
        self.assertIn("additive calibration (+2,500 tokens)", proc.stdout)

    def test_multiplicative_calibration_reports_json_details(self) -> None:
        spec = {
            "context_window": 10000,
            "turns": [{"added_tokens": 500}, {"added_tokens": 300}],
            "calibration": {
                "mode": "multiplicative",
                "estimated_tokens": 1000,
                "observed_tokens": 1500,
            },
        }

        proc = self.run_cli("--json", "-", stdin_text=json.dumps(spec))
        payload = json.loads(proc.stdout)

        self.assertEqual(800, payload["raw_used_tokens"])
        self.assertEqual(1200, payload["used_tokens"])
        self.assertIn("multiplicative calibration", payload["calibration"])

    def test_rejects_missing_usage_source(self) -> None:
        spec = {"context_window": 1000}

        proc = self.run_cli("-", stdin_text=json.dumps(spec), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("either current_input_tokens or a non-empty turns array", proc.stderr)

    def test_rejects_invalid_threshold_order(self) -> None:
        spec = {
            "context_window": 1000,
            "current_input_tokens": 100,
        }

        proc = self.run_cli(
            "--warn-threshold",
            "80",
            "--rotate-threshold",
            "70",
            "-",
            stdin_text=json.dumps(spec),
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("rotate threshold must be greater than warn threshold", proc.stderr)

    def test_rejects_invalid_calibration_mode(self) -> None:
        spec = {
            "context_window": 1000,
            "current_input_tokens": 100,
            "calibration": {
                "mode": "mystery",
                "estimated_tokens": 100,
                "observed_tokens": 200,
            },
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("calibration mode must be 'additive' or 'multiplicative'", proc.stderr)

    def test_cli_calibration_shortcut_applies_without_json_calibration_block(self) -> None:
        spec = {
            "context_window": 10000,
            "current_input_tokens": 900,
            "last_output_tokens": 300,
        }

        proc = self.run_cli(
            "--calibration-mode",
            "additive",
            "--calibrate-estimated-tokens",
            "300",
            "--calibrate-observed-tokens",
            "2800",
            "-",
            stdin_text=json.dumps(spec),
        )

        self.assertIn("3,700 / 10,000 tokens", proc.stdout)
        self.assertIn("additive calibration (+2,500 tokens)", proc.stdout)

    def test_rejects_partial_cli_calibration_arguments(self) -> None:
        spec = {
            "context_window": 1000,
            "current_input_tokens": 100,
        }

        proc = self.run_cli(
            "--calibration-mode",
            "additive",
            "--calibrate-observed-tokens",
            "2800",
            "-",
            stdin_text=json.dumps(spec),
            check=False,
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("CLI calibration requires", proc.stderr)


if __name__ == "__main__":
    unittest.main()
