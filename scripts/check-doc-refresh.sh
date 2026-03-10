#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether planning-doc refresh is due.

Usage:
  scripts/check-doc-refresh.sh [--threshold N]

Examples:
  scripts/check-doc-refresh.sh
  scripts/check-doc-refresh.sh --threshold 20
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
THRESHOLD=20

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
  echo "threshold must be a non-negative integer" >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "git is required" >&2
  exit 1
fi

if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "not inside a git worktree: $ROOT_DIR" >&2
  exit 1
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
    echo "tracked file not found: $file" >&2
    exit 1
  fi

  last_commit=$(git -C "$ROOT_DIR" log -n 1 --format='%H' -- "$file")
  if [[ -z "$last_commit" ]]; then
    echo "no git history found for tracked file: $file" >&2
    exit 1
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
  printf '%s\t%s commits since %s\t%s\n' "$status" "$commit_count" "$short_commit" "$file"
done

if (( due )); then
  remaining=0
  printf 'docs refresh due: at least one tracked file reached the %s-commit threshold\n' "$THRESHOLD"
  printf 'highest commit distance across tracked files: %s\n' "$max_count"
  exit 1
fi

remaining=$((THRESHOLD - max_count))
printf 'docs refresh not due: %s commits remain before the threshold\n' "$remaining"
