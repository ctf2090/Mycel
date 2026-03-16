#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether the current shell session can run a verification command.

Usage:
  scripts/check-runtime-preflight.sh [--json] [--require <command>]...

Examples:
  scripts/check-runtime-preflight.sh
  scripts/check-runtime-preflight.sh --require grep --require tail
  scripts/check-runtime-preflight.sh --json --require cargo --require grep

Behavior:
  default   Check the baseline runtime commands: cargo, bash, rg.
  --require Add a command required by the exact verification command you plan to run.
  --json    Emit machine-readable JSON instead of human-oriented log lines.
EOF
}

JSON_MODE=0
REQUIRED_CMDS=(cargo bash rg)
RESULTS=()

json_escape() {
  local value=${1:-}
  value=${value//\\/\\\\}
  value=${value//\"/\\\"}
  value=${value//$'\n'/\\n}
  value=${value//$'\r'/\\r}
  value=${value//$'\t'/\\t}
  printf '%s' "$value"
}

append_result() {
  local name=$1
  local status=$2
  local detail=${3:-}
  RESULTS+=("${name}"$'\t'"${status}"$'\t'"${detail}")
}

emit_json() {
  local overall_status=$1
  local first=1

  printf '{'
  printf '"status":"%s",' "$(json_escape "$overall_status")"
  printf '"checks":['
  for entry in "${RESULTS[@]}"; do
    IFS=$'\t' read -r name status detail <<<"$entry"
    if (( first )); then
      first=0
    else
      printf ','
    fi
    printf '{'
    printf '"name":"%s",' "$(json_escape "$name")"
    printf '"status":"%s",' "$(json_escape "$status")"
    printf '"detail":"%s"' "$(json_escape "$detail")"
    printf '}'
  done
  printf ']}\n'
}

log_line() {
  if (( ! JSON_MODE )); then
    printf '%s\n' "$1"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON_MODE=1
      shift
      ;;
    --require)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --require" >&2
        usage >&2
        exit 1
      fi
      REQUIRED_CMDS+=("$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

declare -A seen=()
UNIQUE_CMDS=()
for cmd in "${REQUIRED_CMDS[@]}"; do
  if [[ -n "${seen[$cmd]:-}" ]]; then
    continue
  fi
  seen[$cmd]=1
  UNIQUE_CMDS+=("$cmd")
done

missing_count=0
for cmd in "${UNIQUE_CMDS[@]}"; do
  if path=$(command -v "$cmd" 2>/dev/null); then
    append_result "$cmd" "found" "$path"
    log_line "$(printf 'found %-12s %s' "$cmd" "$path")"
  else
    append_result "$cmd" "missing" ""
    log_line "$(printf 'missing %-10s' "$cmd")"
    missing_count=$((missing_count + 1))
  fi
done

if (( missing_count > 0 )); then
  if (( JSON_MODE )); then
    emit_json "blocked"
  else
    echo "runtime preflight blocked: missing ${missing_count} required command(s)" >&2
  fi
  exit 1
fi

if (( JSON_MODE )); then
  emit_json "passed"
else
  echo "runtime preflight passed"
fi
