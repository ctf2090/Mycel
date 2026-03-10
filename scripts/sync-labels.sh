#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Sync repo-tracked GitHub labels from .github/labels.yml.

Usage:
  scripts/sync-labels.sh [--repo OWNER/REPO]

Examples:
  scripts/sync-labels.sh
  scripts/sync-labels.sh --repo ctf2090/Mycel
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
LABELS_FILE="${ROOT_DIR}/.github/labels.yml"
REPO=""

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

count=0
while IFS=$'\t' read -r name color description; do
  [[ -z "$name" ]] && continue
  gh label create "$name" --color "$color" --description "$description" --force "${repo_args[@]}"
  printf 'synced label: %s\n' "$name"
  count=$((count + 1))
done < <(
  ruby -e '
    require "yaml"
    data = YAML.load_file(ARGV[0]) || {}
    labels = data.fetch("labels")
    labels.each do |label|
      values = [
        label.fetch("name"),
        label.fetch("color"),
        label.fetch("description", "")
      ].map { |value| value.to_s.gsub(/\s+/, " ").strip }
      puts values.join("\t")
    end
  ' "$LABELS_FILE"
)

printf 'synced %d labels from %s\n' "$count" "${LABELS_FILE#$ROOT_DIR/}"
