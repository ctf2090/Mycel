#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
summary_lines=()
summary_only=false

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [--summary-only]" >&2
  exit 2
fi

if [[ $# -eq 1 ]]; then
  case "$1" in
    --summary-only)
      summary_only=true
      ;;
    *)
      echo "usage: $0 [--summary-only]" >&2
      exit 2
      ;;
  esac
fi

cd "$repo_root"

print_block() {
  local output="$1"
  if [[ "$summary_only" == false ]]; then
    echo "$output"
  fi
}

if [[ "$summary_only" == false ]]; then
  echo "[smoke] validating repo root should pass"
fi
repo_output="$(cargo run -p mycel-cli -- validate --json 2>&1)"
print_block "$repo_output"

if [[ "$repo_output" != *'"status": "ok"'* ]]; then
  echo "[smoke] expected repo validation status ok" >&2
  exit 1
fi
summary_lines+=("PASS  repo-validate-ok")

if [[ "$summary_only" == false ]]; then
  echo
fi
validate_expected_failure() {
  local artifact_path="$1"
  local expected_source="$2"
  local case_name="$3"

  if [[ "$summary_only" == false ]]; then
    echo "[smoke] validating intentional invalid report should fail: $artifact_path"
  fi

  set +e
  local invalid_output
  invalid_output="$(cargo run -p mycel-cli -- validate "$artifact_path" --json 2>&1)"
  local invalid_exit=$?
  set -e

  print_block "$invalid_output"

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

  summary_lines+=("PASS  ${case_name} -> failed as expected")

  if [[ "$summary_only" == false ]]; then
    echo
  fi
}

validate_expected_error_text() {
  local artifact_path="$1"
  local expected_text="$2"
  local case_name="$3"

  if [[ "$summary_only" == false ]]; then
    echo "[smoke] validating intentional invalid report should fail: $artifact_path"
  fi

  set +e
  local invalid_output
  invalid_output="$(cargo run -p mycel-cli -- validate "$artifact_path" --json 2>&1)"
  local invalid_exit=$?
  set -e

  print_block "$invalid_output"

  if [[ $invalid_exit -eq 0 ]]; then
    echo "[smoke] expected invalid report validation to fail" >&2
    exit 1
  fi

  if [[ "$invalid_output" != *'"status": "failed"'* ]]; then
    echo "[smoke] expected invalid report validation status failed" >&2
    exit 1
  fi

  if [[ "$invalid_output" != *"$expected_text"* ]]; then
    echo "[smoke] expected invalid report failure to mention: $expected_text" >&2
    exit 1
  fi

  summary_lines+=("PASS  ${case_name} -> failed as expected")

  if [[ "$summary_only" == false ]]; then
    echo
  fi
}

validate_expected_failure \
  "sim/reports/invalid/random-seed-prefix-mismatch.example.json" \
  "random" \
  "random-seed-prefix-mismatch"
validate_expected_failure \
  "sim/reports/invalid/auto-seed-prefix-mismatch.example.json" \
  "auto" \
  "auto-seed-prefix-mismatch"
validate_expected_error_text \
  "sim/reports/invalid/unknown-topology-reference.example.json" \
  "does not match any loaded topology" \
  "unknown-topology-reference"
validate_expected_error_text \
  "sim/reports/invalid/unknown-fixture-reference.example.json" \
  "does not match any loaded fixture" \
  "unknown-fixture-reference"

if [[ "$summary_only" == false ]]; then
  echo "[smoke] validating intentional warning report should warn by default"
fi
warning_output="$(cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json 2>&1)"
print_block "$warning_output"

if [[ "$warning_output" != *'"status": "warning"'* ]]; then
  echo "[smoke] expected missing-seed-source validation status warning" >&2
  exit 1
fi

if [[ "$warning_output" != *"does not include seed_source"* ]]; then
  echo "[smoke] expected missing-seed-source warning message" >&2
  exit 1
fi
summary_lines+=("PASS  missing-seed-source -> warning in normal mode")

if [[ "$summary_only" == false ]]; then
  echo
  echo "[smoke] validating intentional warning report should fail under --strict"
fi

set +e
strict_warning_output="$(cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json --strict 2>&1)"
strict_warning_exit=$?
set -e

print_block "$strict_warning_output"

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
summary_lines+=("PASS  missing-seed-source -> non-zero exit under --strict")

echo
echo "[smoke] summary"
for line in "${summary_lines[@]}"; do
  echo "  $line"
done
echo
echo "[smoke] negative validation smoke passed"
