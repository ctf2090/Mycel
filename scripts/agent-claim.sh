#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Claim a new local agent id for a declared role.

Usage:
  scripts/agent-claim.sh <role> [--scope <scope>] [--assigned-by <source>] [--json]

Examples:
  scripts/agent-claim.sh coding
  scripts/agent-claim.sh doc --scope "planning sync for forum notes"
  scripts/agent-claim.sh coding --assigned-by user --json

Behavior:
  - reads or initializes .agent-local/agents.json
  - allocates the next available id for the requested role
  - writes a paused, unconfirmed registry entry
  - creates the mailbox file if missing

After claim, run scripts/agent-start.sh <agent-id> to confirm the assignment
before tracked work begins.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REGISTRY_PATH="$ROOT_DIR/.agent-local/agents.json"
JSON_MODE=0
ROLE=""
SCOPE="pending scope"
ASSIGNED_BY="user"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scope)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --scope" >&2
        exit 1
      fi
      SCOPE=$2
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
      if [[ -n "$ROLE" ]]; then
        echo "unexpected extra argument: $1" >&2
        usage >&2
        exit 1
      fi
      ROLE=$1
      shift
      ;;
  esac
done

if [[ -z "$ROLE" ]]; then
  usage >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "missing required command: python3" >&2
  exit 1
fi

PYTHON_OUTPUT=$(
  ROLE="$ROLE" \
  SCOPE="$SCOPE" \
  ASSIGNED_BY="$ASSIGNED_BY" \
  REGISTRY_PATH="$REGISTRY_PATH" \
  ROOT_DIR="$ROOT_DIR" \
  python3 <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

role = os.environ["ROLE"].strip()
scope = os.environ["SCOPE"].strip()
assigned_by = os.environ["ASSIGNED_BY"].strip()
registry_path = Path(os.environ["REGISTRY_PATH"])
root_dir = Path(os.environ["ROOT_DIR"])

allowed_roles = {"coding", "doc"}
if role not in allowed_roles:
    raise SystemExit(f"unsupported role: {role}")
if not assigned_by:
    raise SystemExit("assigned_by must not be empty")
if not scope:
    raise SystemExit("scope must not be empty")

registry_path.parent.mkdir(parents=True, exist_ok=True)

if registry_path.exists():
    try:
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid registry JSON: {exc}")
else:
    registry = {
        "version": 1,
        "updated_at": None,
        "agent_count": 0,
        "agents": [],
    }

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

max_suffix = 0
for entry in agents:
    if not isinstance(entry, dict):
        raise SystemExit("invalid registry: agent entry must be an object")
    agent_id = entry.get("id")
    if isinstance(agent_id, str) and agent_id.startswith(f"{role}-"):
        suffix = agent_id[len(role) + 1 :]
        if suffix.isdigit():
            max_suffix = max(max_suffix, int(suffix))

new_id = f"{role}-{max_suffix + 1}"
mailbox_rel = f".agent-local/{new_id}.md"
mailbox_path = root_dir / mailbox_rel
mailbox_path.parent.mkdir(parents=True, exist_ok=True)
if not mailbox_path.exists():
    mailbox_path.write_text(f"# Mailbox for {new_id}\n\n", encoding="utf-8")

now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
entry = {
    "id": new_id,
    "role": role,
    "assigned_by": assigned_by,
    "assigned_at": now,
    "confirmed_by_agent": False,
    "confirmed_at": None,
    "status": "paused",
    "scope": scope,
    "files": [],
    "mailbox": mailbox_rel,
}
agents.append(entry)
registry["agent_count"] = len(agents)
registry["updated_at"] = now

registry_path.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")

result = {
    "status": "ok",
    "agent_id": new_id,
    "role": role,
    "scope": scope,
    "assigned_by": assigned_by,
    "assigned_at": now,
    "mailbox": mailbox_rel,
}
print(json.dumps(result))
PY
)

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit 0
fi

CLAIM_ID=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["agent_id"])' "$PYTHON_OUTPUT")
CLAIM_ROLE=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["role"])' "$PYTHON_OUTPUT")
CLAIM_SCOPE=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["scope"])' "$PYTHON_OUTPUT")
CLAIM_ASSIGNED_BY=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["assigned_by"])' "$PYTHON_OUTPUT")
CLAIM_ASSIGNED_AT=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["assigned_at"])' "$PYTHON_OUTPUT")
CLAIM_MAILBOX=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["mailbox"])' "$PYTHON_OUTPUT")

printf 'agent claimed: %s\n' "$CLAIM_ID"
printf 'role: %s\n' "$CLAIM_ROLE"
printf 'scope: %s\n' "$CLAIM_SCOPE"
printf 'assigned_by: %s\n' "$CLAIM_ASSIGNED_BY"
printf 'assigned_at: %s\n' "$CLAIM_ASSIGNED_AT"
printf 'mailbox: %s\n' "$CLAIM_MAILBOX"
printf 'next: scripts/agent-start.sh %s\n' "$CLAIM_ID"
