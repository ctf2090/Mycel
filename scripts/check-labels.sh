#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether GitHub host labels match .github/labels.yml.

Usage:
  scripts/check-labels.sh [--repo OWNER/REPO] [--strict]

Examples:
  scripts/check-labels.sh
  scripts/check-labels.sh --repo ctf2090/Mycel
  scripts/check-labels.sh --strict
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
LABELS_FILE="${ROOT_DIR}/.github/labels.yml"
REPO=""
STRICT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -R|--repo)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        usage >&2
        exit 1
      fi
      REPO="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --strict)
      STRICT=1
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 1
fi

if ! command -v ruby >/dev/null 2>&1; then
  echo "ruby is required" >&2
  exit 1
fi

if [[ ! -f "$LABELS_FILE" ]]; then
  echo "labels file not found: $LABELS_FILE" >&2
  exit 1
fi

repo_args=()
if [[ -n "$REPO" ]]; then
  repo_args=(--repo "$REPO")
fi

tmp_expected=$(mktemp)
tmp_actual=$(mktemp)
cleanup() {
  rm -f "$tmp_expected" "$tmp_actual"
}
trap cleanup EXIT

ruby -e '
  require "yaml"
  data = YAML.load_file(ARGV[0]) || {}
  labels = data.fetch("labels")
  labels.each do |label|
    values = [
      label.fetch("name"),
      label.fetch("color").downcase,
      label.fetch("description", "")
    ].map { |value| value.to_s.gsub(/\s+/, " ").strip }
    puts values.join("\t")
  end
' "$LABELS_FILE" | sort > "$tmp_expected"

gh label list --limit 200 --json name,color,description "${repo_args[@]}" | ruby -rjson -e '
  labels = JSON.parse(STDIN.read)
  labels.each do |label|
    values = [
      label.fetch("name"),
      label.fetch("color").downcase,
      label.fetch("description", "")
    ].map { |value| value.to_s.gsub(/\s+/, " ").strip }
    puts values.join("\t")
  end
' | sort > "$tmp_actual"

missing=0
mismatch=0
while IFS=$'\t' read -r name color description; do
  [[ -z "$name" ]] && continue
  expected_line="${name}"$'\t'"${color}"$'\t'"${description}"
  actual_line=$(awk -F '\t' -v target="$name" '$1 == target { print; exit }' "$tmp_actual")
  if [[ -z "$actual_line" ]]; then
    printf 'missing label on GitHub: %s\n' "$name"
    missing=1
  elif [[ "$actual_line" != "$expected_line" ]]; then
    printf 'mismatched label: %s\n' "$name"
    printf '  expected: %s\t%s\n' "$color" "$description"
    actual_color=${actual_line#*$'\t'}
    actual_desc=${actual_color#*$'\t'}
    actual_color=${actual_color%%$'\t'*}
    printf '  actual:   %s\t%s\n' "$actual_color" "$actual_desc"
    mismatch=1
  fi
done < "$tmp_expected"

extra=0
if (( STRICT )); then
  while IFS=$'\t' read -r name _rest; do
    [[ -z "$name" ]] && continue
    if ! awk -F '\t' -v target="$name" '$1 == target { found = 1; exit } END { exit found ? 0 : 1 }' "$tmp_expected"; then
      printf 'extra label on GitHub (not tracked): %s\n' "$name"
      extra=1
    fi
  done < "$tmp_actual"
fi

if (( missing || mismatch || extra )); then
  exit 1
fi

count=$(wc -l < "$tmp_expected" | tr -d ' ')
if (( STRICT )); then
  printf 'labels are in strict sync for %d tracked labels\n' "$count"
else
  printf 'tracked labels are in sync for %d labels\n' "$count"
fi
