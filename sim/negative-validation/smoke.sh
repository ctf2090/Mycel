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
echo "[smoke] validating intentional invalid report should fail"

set +e
invalid_output="$(cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json 2>&1)"
invalid_exit=$?
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

if [[ "$invalid_output" != *"seed_source 'random'"* ]]; then
  echo "[smoke] expected invalid report failure to mention seed_source mismatch" >&2
  exit 1
fi

echo
echo "[smoke] negative validation smoke passed"
