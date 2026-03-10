#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether planning-doc refresh is due.

Usage:
  scripts/check-doc-refresh.sh [--threshold N] [--json]

Examples:
  scripts/check-doc-refresh.sh
  scripts/check-doc-refresh.sh --threshold 20
  scripts/check-doc-refresh.sh --json
  scripts/check-doc-refresh.sh --threshold 20 --json
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
THRESHOLD=20
JSON_MODE=0
RESULTS=()
ERROR_MESSAGE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--threshold)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        usage >&2
        exit 1
      fi
      THRESHOLD="$2"
      shift 2
      ;;
    --json)
      JSON_MODE=1
      shift
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

if ! [[ "$THRESHOLD" =~ ^[0-9]+$ ]]; then
  ERROR_MESSAGE="threshold must be a non-negative integer"
  if (( JSON_MODE )); then
    printf '{"status":"failed","threshold":"%s","repo_root":"%s","checks":[],"error":"%s"}\n' \
      "$THRESHOLD" "$ROOT_DIR" "$ERROR_MESSAGE"
  else
    echo "$ERROR_MESSAGE" >&2
  fi
  exit 1
fi

json_escape() {
  local value=$1
  value=${value//\\/\\\\}
  value=${value//\"/\\\"}
  value=${value//$'\n'/\\n}
  value=${value//$'\r'/\\r}
  value=${value//$'\t'/\\t}
  printf '%s' "$value"
}

append_result() {
  local file=$1
  local status=$2
  local count=$3
  local short_commit=$4

  RESULTS+=("${file}"$'\t'"${status}"$'\t'"${count}"$'\t'"${short_commit}")
}

emit_json() {
  local overall_status=$1
  local remaining=$2
  local first=1

  printf '{'
  printf '"status":"%s",' "$(json_escape "$overall_status")"
  printf '"threshold":%s,' "$(json_escape "$THRESHOLD")"
  printf '"repo_root":"%s",' "$(json_escape "$ROOT_DIR")"
  printf '"highest_commit_distance":%s,' "$(json_escape "$max_count")"
  printf '"remaining_commits":%s,' "$(json_escape "$remaining")"
  printf '"checks":['
  for entry in "${RESULTS[@]}"; do
    IFS=$'\t' read -r file status count short_commit <<<"$entry"
    if (( first )); then
      first=0
    else
      printf ','
    fi
    printf '{'
    printf '"file":"%s",' "$(json_escape "$file")"
    printf '"status":"%s",' "$(json_escape "$status")"
    printf '"commit_count":%s,' "$(json_escape "$count")"
    printf '"last_commit":"%s"' "$(json_escape "$short_commit")"
    printf '}'
  done
  printf ']'
  if [[ -n "$ERROR_MESSAGE" ]]; then
    printf ',"error":"%s"' "$(json_escape "$ERROR_MESSAGE")"
  fi
  printf '}\n'
}

fail() {
  local message=$1
  ERROR_MESSAGE=$message
  if (( JSON_MODE )); then
    emit_json "failed" 0
  else
    echo "$message" >&2
  fi
  exit 1
}

if ! command -v git >/dev/null 2>&1; then
  fail "git is required"
fi

if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  fail "not inside a git worktree: $ROOT_DIR"
fi

tracked_files=(
  "ROADMAP.md"
  "ROADMAP.zh-TW.md"
  "IMPLEMENTATION-CHECKLIST.en.md"
  "IMPLEMENTATION-CHECKLIST.zh-TW.md"
)

due=0
max_count=0

for file in "${tracked_files[@]}"; do
  path="${ROOT_DIR}/${file}"
  if [[ ! -f "$path" ]]; then
    fail "tracked file not found: $file"
  fi

  last_commit=$(git -C "$ROOT_DIR" log -n 1 --format='%H' -- "$file")
  if [[ -z "$last_commit" ]]; then
    fail "no git history found for tracked file: $file"
  fi

  commit_count=$(git -C "$ROOT_DIR" rev-list --count "${last_commit}..HEAD")
  if (( commit_count > max_count )); then
    max_count=$commit_count
  fi
  if (( commit_count >= THRESHOLD )); then
    due=1
    status="due"
  else
    status="ok"
  fi

  short_commit=$(git -C "$ROOT_DIR" rev-parse --short "$last_commit")
  append_result "$file" "$status" "$commit_count" "$short_commit"
  if (( ! JSON_MODE )); then
    printf '%s\t%s commits since %s\t%s\n' "$status" "$commit_count" "$short_commit" "$file"
  fi
done

if (( due )); then
  remaining=0
  if (( JSON_MODE )); then
    emit_json "due" "$remaining"
  else
    printf 'docs refresh due: at least one tracked file reached the %s-commit threshold\n' "$THRESHOLD"
    printf 'highest commit distance across tracked files: %s\n' "$max_count"
  fi
  exit 1
fi

remaining=$((THRESHOLD - max_count))
if (( JSON_MODE )); then
  emit_json "ok" "$remaining"
else
  printf 'docs refresh not due: %s commits remain before the threshold\n' "$remaining"
fi
