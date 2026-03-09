#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$repo_root"

echo "[smoke] validating repo root should pass"
repo_output="$(cargo run -p mycel-cli -- validate --json)"
echo "$repo_output"

if [[ "$repo_output" != *'"status": "ok"'* ]]; then
  echo "[smoke] expected repo validation status ok" >&2
  exit 1
fi

echo
validate_expected_failure() {
  local artifact_path="$1"
  local expected_source="$2"

  echo "[smoke] validating intentional invalid report should fail: $artifact_path"

  set +e
  local invalid_output
  invalid_output="$(cargo run -p mycel-cli -- validate "$artifact_path" --json 2>&1)"
  local invalid_exit=$?
  set -e

  echo "$invalid_output"

  if [[ $invalid_exit -eq 0 ]]; then
    echo "[smoke] expected invalid report validation to fail" >&2
    exit 1
  fi

  if [[ "$invalid_output" != *'"status": "failed"'* ]]; then
    echo "[smoke] expected invalid report validation status failed" >&2
    exit 1
  fi

  if [[ "$invalid_output" != *"seed_source '$expected_source'"* ]]; then
    echo "[smoke] expected invalid report failure to mention seed_source mismatch" >&2
    exit 1
  fi

  echo
}

validate_expected_failure \
  "sim/reports/invalid/random-seed-prefix-mismatch.example.json" \
  "random"
validate_expected_failure \
  "sim/reports/invalid/auto-seed-prefix-mismatch.example.json" \
  "auto"

echo "[smoke] validating intentional warning report should warn by default"
warning_output="$(cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json)"
echo "$warning_output"

if [[ "$warning_output" != *'"status": "warning"'* ]]; then
  echo "[smoke] expected missing-seed-source validation status warning" >&2
  exit 1
fi

if [[ "$warning_output" != *"does not include seed_source"* ]]; then
  echo "[smoke] expected missing-seed-source warning message" >&2
  exit 1
fi

echo
echo "[smoke] validating intentional warning report should fail under --strict"

set +e
strict_warning_output="$(cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json --strict 2>&1)"
strict_warning_exit=$?
set -e

echo "$strict_warning_output"

if [[ $strict_warning_exit -eq 0 ]]; then
  echo "[smoke] expected missing-seed-source strict validation to fail" >&2
  exit 1
fi

if [[ "$strict_warning_output" != *'"status": "warning"'* ]]; then
  echo "[smoke] expected missing-seed-source strict validation status warning" >&2
  exit 1
fi

if [[ "$strict_warning_output" != *"does not include seed_source"* ]]; then
  echo "[smoke] expected missing-seed-source strict warning message" >&2
  exit 1
fi

echo
echo "[smoke] negative validation smoke passed"
