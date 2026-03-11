#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Recover from an interrupted chat by pausing a stale agent and starting a replacement agent.

Usage:
  scripts/agent-recover.sh <stale-agent-id> [--scope <scope>] [--assigned-by <source>] [--json]

Examples:
  scripts/agent-recover.sh coding-2
  scripts/agent-recover.sh doc-1 --scope "planning sync for forum notes"
  scripts/agent-recover.sh coding-2 --assigned-by maintainer --json

Behavior:
  - reads .agent-local/agents.json
  - validates the stale agent entry and mailbox path
  - pauses the stale agent if needed
  - claims a fresh agent id for the same role
  - starts the replacement agent immediately
  - appends a takeover note to the replacement mailbox

Read the stale mailbox before resuming tracked work.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REGISTRY_PATH="$ROOT_DIR/.agent-local/agents.json"
JSON_MODE=0
STALE_AGENT_ID=""
RECOVER_SCOPE=""
ASSIGNED_BY="user"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scope)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --scope" >&2
        exit 1
      fi
      RECOVER_SCOPE=$2
      shift 2
      ;;
    --assigned-by)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --assigned-by" >&2
        exit 1
      fi
      ASSIGNED_BY=$2
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
    -*)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ -n "$STALE_AGENT_ID" ]]; then
        echo "unexpected extra argument: $1" >&2
        usage >&2
        exit 1
      fi
      STALE_AGENT_ID=$1
      shift
      ;;
  esac
done

if [[ -z "$STALE_AGENT_ID" ]]; then
  usage >&2
  exit 1
fi

if [[ ! -f "$REGISTRY_PATH" ]]; then
  echo "missing registry file: $REGISTRY_PATH" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "missing required command: python3" >&2
  exit 1
fi

PYTHON_OUTPUT=$(
  STALE_AGENT_ID="$STALE_AGENT_ID" \
  RECOVER_SCOPE="$RECOVER_SCOPE" \
  ASSIGNED_BY="$ASSIGNED_BY" \
  REGISTRY_PATH="$REGISTRY_PATH" \
  ROOT_DIR="$ROOT_DIR" \
  python3 <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

stale_agent_id = os.environ["STALE_AGENT_ID"].strip()
recover_scope = os.environ["RECOVER_SCOPE"].strip()
assigned_by = os.environ["ASSIGNED_BY"].strip()
registry_path = Path(os.environ["REGISTRY_PATH"])
root_dir = Path(os.environ["ROOT_DIR"])

if not assigned_by:
    raise SystemExit("assigned_by must not be empty")

try:
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
except FileNotFoundError:
    raise SystemExit(f"missing registry file: {registry_path}")
except json.JSONDecodeError as exc:
    raise SystemExit(f"invalid registry JSON: {exc}")

if not isinstance(registry, dict):
    raise SystemExit("invalid registry: top-level JSON value must be an object")

agents = registry.get("agents")
if not isinstance(agents, list):
    raise SystemExit("invalid registry: agents must be an array")

agent_count = registry.get("agent_count")
if agent_count != len(agents):
    raise SystemExit(
        f"invalid registry: agent_count={agent_count!r} does not match agents length {len(agents)}"
    )

matches = [entry for entry in agents if isinstance(entry, dict) and entry.get("id") == stale_agent_id]
if not matches:
    raise SystemExit(f"agent entry not found: {stale_agent_id}")
if len(matches) > 1:
    raise SystemExit(f"invalid registry: duplicate agent id {stale_agent_id}")

stale_entry = matches[0]
role = stale_entry.get("role")
if not isinstance(role, str) or not role.strip():
    raise SystemExit(f"agent {stale_agent_id} is missing required field: role")
if role not in {"coding", "doc"}:
    raise SystemExit(f"unsupported role in stale entry: {role}")

scope = recover_scope or stale_entry.get("scope")
if not isinstance(scope, str) or not scope.strip():
    raise SystemExit(f"agent {stale_agent_id} is missing required field: scope")

stale_mailbox_value = stale_entry.get("mailbox")
if not isinstance(stale_mailbox_value, str) or not stale_mailbox_value.strip():
    raise SystemExit(f"agent {stale_agent_id} is missing required field: mailbox")

stale_mailbox_path = Path(stale_mailbox_value)
if not stale_mailbox_path.is_absolute():
    stale_mailbox_path = root_dir / stale_mailbox_path

stale_status = stale_entry.get("status")
if not isinstance(stale_status, str) or not stale_status.strip():
    raise SystemExit(f"agent {stale_agent_id} is missing required field: status")
if stale_status == "done":
    raise SystemExit(f"agent {stale_agent_id} cannot be recovered because status is done")

max_suffix = 0
for entry in agents:
    if not isinstance(entry, dict):
        raise SystemExit("invalid registry: agent entry must be an object")
    agent_id = entry.get("id")
    if isinstance(agent_id, str) and agent_id.startswith(f"{role}-"):
        suffix = agent_id[len(role) + 1 :]
        if suffix.isdigit():
            max_suffix = max(max_suffix, int(suffix))

new_agent_id = f"{role}-{max_suffix + 1}"
new_mailbox_rel = f".agent-local/{new_agent_id}.md"
new_mailbox_path = root_dir / new_mailbox_rel
new_mailbox_path.parent.mkdir(parents=True, exist_ok=True)
if not new_mailbox_path.exists():
    new_mailbox_path.write_text(f"# Mailbox for {new_agent_id}\n\n", encoding="utf-8")

now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
stale_entry["status"] = "paused"

new_entry = {
    "id": new_agent_id,
    "role": role,
    "assigned_by": assigned_by,
    "assigned_at": now,
    "confirmed_by_agent": True,
    "confirmed_at": now,
    "status": "active",
    "scope": scope,
    "files": [],
    "mailbox": new_mailbox_rel,
}
agents.append(new_entry)
registry["agent_count"] = len(agents)
registry["updated_at"] = now

registry_path.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")

takeover_note = f"taking over from {stale_agent_id} after interrupted chat"
existing_mailbox = new_mailbox_path.read_text(encoding="utf-8")
if takeover_note not in existing_mailbox:
    new_mailbox_path.write_text(
        existing_mailbox + f"- {takeover_note}\n",
        encoding="utf-8",
    )

result = {
    "status": "ok",
    "stale_agent_id": stale_agent_id,
    "stale_status": stale_status,
    "stale_mailbox": str(stale_mailbox_path.relative_to(root_dir)),
    "replacement_agent_id": new_agent_id,
    "replacement_role": role,
    "replacement_scope": scope,
    "replacement_mailbox": new_mailbox_rel,
    "assigned_by": assigned_by,
    "updated_at": now,
    "takeover_note": takeover_note,
}
print(json.dumps(result))
PY
)

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit 0
fi

python3 -c '
import json
import sys

data = json.loads(sys.argv[1])
print(f"stale_agent: {data['\''stale_agent_id'\'']}")
print(f"stale_status_before_recovery: {data['\''stale_status'\'']}")
print(f"stale_mailbox: {data['\''stale_mailbox'\'']}")
print(f"replacement_agent: {data['\''replacement_agent_id'\'']}")
print(f"role: {data['\''replacement_role'\'']}")
print(f"scope: {data['\''replacement_scope'\'']}")
print(f"replacement_mailbox: {data['\''replacement_mailbox'\'']}")
print(f"assigned_by: {data['\''assigned_by'\'']}")
print(f"updated_at: {data['\''updated_at'\'']}")
print(f"takeover_note: {data['\''takeover_note'\'']}")
print("next: read the stale mailbox before resuming tracked work")
' "$PYTHON_OUTPUT"
