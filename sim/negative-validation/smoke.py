#!/usr/bin/env python3
"""Run the negative-validation smoke matrix."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CASES = (
    ("repo-validate-ok", None, None, "ok", False),
    (
        "random-seed-prefix-mismatch",
        "sim/reports/invalid/random-seed-prefix-mismatch.example.json",
        "seed_source 'random'",
        "failed",
        True,
    ),
    (
        "auto-seed-prefix-mismatch",
        "sim/reports/invalid/auto-seed-prefix-mismatch.example.json",
        "seed_source 'auto'",
        "failed",
        True,
    ),
    (
        "unknown-topology-reference",
        "sim/reports/invalid/unknown-topology-reference.example.json",
        "does not match any loaded topology",
        "failed",
        True,
    ),
    (
        "unknown-fixture-reference",
        "sim/reports/invalid/unknown-fixture-reference.example.json",
        "does not match any loaded fixture",
        "failed",
        True,
    ),
    (
        "missing-seed-source",
        "sim/reports/invalid/missing-seed-source.example.json",
        "does not include seed_source",
        "warning",
        False,
    ),
    (
        "missing-seed-source-strict",
        "sim/reports/invalid/missing-seed-source.example.json",
        "does not include seed_source",
        "warning",
        True,
    ),
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--summary-only", action="store_true")
    parser.add_argument("--case")
    parser.add_argument("-h", "--help", action="help", help="show this help message and exit")
    return parser.parse_args()


def print_block(output: str, *, summary_only: bool) -> None:
    if not summary_only:
        print(output)


def run_validate(*args: str) -> tuple[int, str]:
    proc = subprocess.run(
        ["cargo", "run", "-p", "mycel-cli", "--", "validate", *args, "--json"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    return proc.returncode, proc.stdout + proc.stderr


def validate_repo_success(summary_lines: list[str], *, summary_only: bool) -> None:
    if not summary_only:
        print("[smoke] validating repo root should pass")
    _, output = run_validate()
    print_block(output, summary_only=summary_only)
    if '"status": "ok"' not in output:
        raise SystemExit("[smoke] expected repo validation status ok")
    summary_lines.append("PASS  repo-validate-ok")
    if not summary_only:
        print()


def validate_case(
    case_name: str,
    artifact_path: str,
    expected_text: str,
    expected_status: str,
    expect_nonzero: bool,
    summary_lines: list[str],
    *,
    summary_only: bool,
) -> None:
    if case_name == "missing-seed-source-strict":
        if not summary_only:
            print("[smoke] validating intentional warning report should fail under --strict")
        exit_code, output = run_validate(artifact_path, "--strict")
    elif expected_status == "warning":
        if not summary_only:
            print("[smoke] validating intentional warning report should warn by default")
        exit_code, output = run_validate(artifact_path)
    else:
        if not summary_only:
            print(f"[smoke] validating intentional invalid report should fail: {artifact_path}")
        exit_code, output = run_validate(artifact_path)

    print_block(output, summary_only=summary_only)

    if expect_nonzero and exit_code == 0:
        raise SystemExit(
            f"[smoke] expected {case_name} {'strict validation ' if case_name == 'missing-seed-source-strict' else 'validation '}to fail"
        )
    if not expect_nonzero and exit_code != 0:
        raise SystemExit(f"[smoke] expected {case_name} validation to succeed")
    if f'"status": "{expected_status}"' not in output:
        raise SystemExit(f"[smoke] expected {case_name} validation status {expected_status}")
    if expected_text not in output:
        if case_name == "missing-seed-source-strict":
            raise SystemExit(f"[smoke] expected {case_name} strict warning message")
        raise SystemExit(f"[smoke] expected invalid report failure to mention: {expected_text}")

    if case_name == "missing-seed-source":
        summary_lines.append("PASS  missing-seed-source -> warning in normal mode")
    elif case_name == "missing-seed-source-strict":
        summary_lines.append("PASS  missing-seed-source-strict -> non-zero exit under --strict")
    else:
        summary_lines.append(f"PASS  {case_name} -> failed as expected")

    if not summary_only:
        print()


def main() -> int:
    args = parse_args()
    summary_lines: list[str] = []
    selected_case = args.case
    seen_case = False

    for case_name, artifact_path, expected_text, expected_status, expect_nonzero in CASES:
        if selected_case and case_name != selected_case:
            continue
        seen_case = True
        if artifact_path is None:
            validate_repo_success(summary_lines, summary_only=args.summary_only)
            continue
        validate_case(
            case_name,
            artifact_path,
            expected_text,
            expected_status,
            expect_nonzero,
            summary_lines,
            summary_only=args.summary_only,
        )

    if selected_case and not seen_case:
        print(f"[smoke] unknown case: {selected_case}", file=sys.stderr)
        print(
            "[smoke] available cases: repo-validate-ok, random-seed-prefix-mismatch, auto-seed-prefix-mismatch, unknown-topology-reference, unknown-fixture-reference, missing-seed-source, missing-seed-source-strict",
            file=sys.stderr,
        )
        return 2

    print()
    print("[smoke] summary")
    for line in summary_lines:
        print(f"  {line}")
    print()
    print("[smoke] negative validation smoke passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
