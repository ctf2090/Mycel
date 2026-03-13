#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import uuid
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from item_id_checklist import agents_bootstrap_checklist_path, materialize_checklist
from item_id_checklist_mark import ItemIdChecklistMarkError, apply_updates

ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
MAILBOX_DIR = ROOT_DIR / ".agent-local" / "mailboxes"
AGENT_LOCAL_DIR = ROOT_DIR / ".agent-local"
DEV_SETUP_STATUS_PATH = AGENT_LOCAL_DIR / "dev-setup-status.md"
AGENTS_PATH = ROOT_DIR / "AGENTS.md"
AGENTS_LOCAL_PATH = ROOT_DIR / "AGENTS-LOCAL.md"
PLANNING_SYNC_PLAN_PATH = ROOT_DIR / "docs" / "PLANNING-SYNC-PLAN.md"
ALLOWED_ROLES = {"coding", "doc"}
ALLOWED_STATUSES = {"active", "inactive", "paused", "blocked", "done"}
REGISTRY_VERSION = 2
STALE_INACTIVE_SECONDS = 3600
STALE_RETENTION_SECONDS = 3600
STALE_PAUSED_SECONDS = 3600
PAUSED_CLEANUP_SECONDS = 2 * 3600
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))


class RegistryError(Exception):
    pass


def format_timestamp(dt: datetime) -> str:
    return dt.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")


def utc_now() -> str:
    return format_timestamp(datetime.now(timezone.utc))


def parse_utc_timestamp(value: str) -> datetime:
    normalized = value.strip()
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"
    elif len(normalized) >= 5 and normalized[-5] in {"+", "-"} and normalized[-3] != ":":
        normalized = f"{normalized[:-2]}:{normalized[-2:]}"
    return datetime.fromisoformat(normalized).astimezone(timezone.utc)


def normalize_timestamp_string(value: Any) -> Any:
    if not isinstance(value, str) or not value.strip():
        return value
    try:
        parsed = parse_utc_timestamp(value)
    except ValueError:
        return value
    return format_timestamp(parsed)


def empty_registry() -> dict[str, Any]:
    return {
        "version": REGISTRY_VERSION,
        "updated_at": None,
        "agent_count": 0,
        "agents": [],
    }


def load_registry(*, allow_missing: bool = False) -> dict[str, Any]:
    if allow_missing and not REGISTRY_PATH.exists():
        REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
        return empty_registry()

    try:
        registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise RegistryError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise RegistryError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(registry, dict):
        raise RegistryError("invalid registry: top-level JSON value must be an object")

    agents = registry.get("agents")
    if not isinstance(agents, list):
        raise RegistryError("invalid registry: agents must be an array")

    agent_count = registry.get("agent_count")
    if agent_count != len(agents):
        raise RegistryError(
            f"invalid registry: agent_count={agent_count!r} does not match agents length {len(agents)}"
        )

    for entry in agents:
        if not isinstance(entry, dict):
            raise RegistryError("invalid registry: agent entry must be an object")

    return registry


def save_registry(registry: dict[str, Any]) -> None:
    REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
    REGISTRY_PATH.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")


def resolve_mailbox_path(mailbox_value: str) -> Path:
    mailbox_path = Path(mailbox_value)
    if not mailbox_path.is_absolute():
        mailbox_path = ROOT_DIR / mailbox_path
    return mailbox_path


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def resolve_agent_local_path(path_value: str | Path) -> Path:
    candidate = Path(path_value)
    if not candidate.is_absolute():
        candidate = ROOT_DIR / candidate
    resolved = candidate.resolve()
    try:
        resolved.relative_to(AGENT_LOCAL_DIR.resolve())
    except ValueError as exc:
        raise RegistryError("checklist output must live under .agent-local/") from exc
    return resolved


def agent_dir_for_uid(agent_uid: str) -> Path:
    return (AGENT_LOCAL_DIR / "agents" / agent_uid).resolve()


def mailbox_symlink_rel_for_uid(agent_uid: str) -> str:
    return f".agent-local/agents/{agent_uid}/mailbox.md"


def ensure_agent_mailbox_symlink(agent_uid: str, mailbox_path: Path) -> Path:
    agent_dir = agent_dir_for_uid(agent_uid)
    agent_dir.mkdir(parents=True, exist_ok=True)
    link_path = agent_dir / "mailbox.md"

    if link_path.exists() or link_path.is_symlink():
        if link_path.is_symlink() and link_path.resolve() == mailbox_path.resolve():
            return link_path
        if link_path.is_symlink():
            link_path.unlink()
        else:
            raise RegistryError(
                f"cannot create mailbox symlink because a non-symlink path exists: {relative_to_root(link_path)}"
            )

    target_rel = os.path.relpath(mailbox_path, start=link_path.parent)
    link_path.symlink_to(target_rel)
    return link_path


def path_exists_and_contains(path: Path, needle: str) -> bool:
    if not path.exists():
        return False
    return needle in path.read_text(encoding="utf-8")


def checkbox_line(checked: bool, text: str) -> str:
    marker = "[X]" if checked else "[ ]"
    return f"- {marker} {text}"


def checklist_item_line(item_id: str, checked: bool, text: str) -> str:
    marker = "[X]" if checked else "[ ]"
    return f"- {marker} {text} <!-- item-id: {item_id} -->"


def require_non_empty_str(entry: dict[str, Any], field: str, agent_ref: str) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"agent {agent_ref} is missing required field: {field}")
    return value


def require_status(entry: dict[str, Any], agent_ref: str) -> str:
    status = require_non_empty_str(entry, "status", agent_ref)
    if status not in ALLOWED_STATUSES:
        raise RegistryError(f"agent {agent_ref} has unsupported status: {status}")
    return status


def make_legacy_agent_uid(display_id: str) -> str:
    suffix = uuid.uuid5(uuid.NAMESPACE_URL, f"mycel-agent:{display_id}").hex[:8]
    return f"agt_{suffix}"


def make_new_agent_uid(registry: dict[str, Any]) -> str:
    existing = {entry.get("agent_uid") for entry in registry["agents"]}
    while True:
        candidate = f"agt_{uuid.uuid4().hex[:8]}"
        if candidate not in existing:
            return candidate


def mailbox_rel_for_uid(agent_uid: str) -> str:
    return f".agent-local/mailboxes/{agent_uid}.md"


def checklist_rel_for_uid(agent_uid: str) -> str:
    return f".agent-local/agents/{agent_uid}/work-checklist.md"


def legacy_checklist_rel_for_uid(agent_uid: str) -> str:
    return f".agent-local/checklists/{agent_uid}-work-checklist.md"


def legacy_root_checklist_rel_for_uid(agent_uid: str) -> str:
    return f".agent-local/{agent_uid}-work-checklist.md"


def resolve_existing_work_checklist_path(agent_uid: str) -> Path:
    for relative_path in [
        checklist_rel_for_uid(agent_uid),
        legacy_checklist_rel_for_uid(agent_uid),
        legacy_root_checklist_rel_for_uid(agent_uid),
    ]:
        candidate = resolve_agent_local_path(relative_path)
        if candidate.exists():
            return candidate
    return resolve_agent_local_path(checklist_rel_for_uid(agent_uid))


def ensure_mailbox(path: Path, *, title: str, source_path: Path | None = None) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        return
    if source_path is not None and source_path.exists():
        path.write_text(source_path.read_text(encoding="utf-8"), encoding="utf-8")
        return
    path.write_text(f"# Mailbox for {title}\n\n", encoding="utf-8")


def last_display_id(entry: dict[str, Any]) -> str | None:
    history = entry.get("display_history")
    if not isinstance(history, list):
        return None
    for record in reversed(history):
        if not isinstance(record, dict):
            continue
        display_id = record.get("display_id")
        if isinstance(display_id, str) and display_id.strip():
            return display_id
    return None


def current_display_id(entry: dict[str, Any]) -> str | None:
    value = entry.get("current_display_id")
    if isinstance(value, str) and value.strip():
        return value
    return None


def render_work_checklist(entry: dict[str, Any], *, generated_at: str) -> str:
    agent_uid = require_non_empty_str(entry, "agent_uid", "<agent>")
    role = require_non_empty_str(entry, "role", agent_uid)
    scope = require_non_empty_str(entry, "scope", agent_uid)
    status = require_status(entry, agent_uid)
    display_id = current_display_id(entry)
    confirmed = bool(entry.get("confirmed_by_agent")) and isinstance(entry.get("confirmed_at"), str)
    mailbox_rel = require_non_empty_str(entry, "mailbox", agent_uid)
    mailbox_path = resolve_mailbox_path(mailbox_rel)
    dev_setup_ready = path_exists_and_contains(DEV_SETUP_STATUS_PATH, "Status: ready")

    lines = [
        f"# Agent Work Checklist for {display_id or agent_uid}",
        "",
        f"- Agent UID: `{agent_uid}`",
        f"- Display ID: `{display_id or 'none'}`",
        f"- Role: `{role}`",
        f"- Scope: `{scope}`",
        f"- Status: `{status}`",
        f"- Mailbox: `{relative_to_root(mailbox_path)}`",
        f"- Generated at: `{generated_at}`",
        "- Mark items with `scripts/agent_registry.py work-checklist-mark <agent-ref> <item-id>`.",
        "",
        "## Bootstrap State",
        checklist_item_line("bootstrap.registry-entry", True, f"Registry entry exists for `{agent_uid}`."),
        checklist_item_line(
            "bootstrap.display-slot",
            bool(display_id),
            f"Display slot is assigned{f' as `{display_id}`' if display_id else ''}.",
        ),
        checklist_item_line("bootstrap.started", confirmed, "Agent has completed the `start` confirmation step."),
        checklist_item_line(
            "bootstrap.mailbox-exists",
            mailbox_path.exists(),
            f"Mailbox file exists at `{relative_to_root(mailbox_path)}`.",
        ),
        checklist_item_line("bootstrap.dev-setup-ready", dev_setup_ready, "Workspace dev setup status is marked `ready`."),
        checklist_item_line("bootstrap.agents-md", AGENTS_PATH.exists(), "`AGENTS.md` is available for repo-wide instructions."),
        checklist_item_line("bootstrap.agents-local-md", AGENTS_LOCAL_PATH.exists(), "`AGENTS-LOCAL.md` overlay is available."),
        "",
        "## Current Command Workflow",
        checklist_item_line("workflow.work-cycle-active", status == "active", "Current work cycle is active for this command."),
        checklist_item_line(
            "workflow.begin-next-command",
            False,
            f"Begin the next command with `scripts/agent_work_cycle.py begin {agent_uid} --scope <scope>`.",
        ),
        checklist_item_line(
            "workflow.end-this-command",
            False,
            f"End this command with `scripts/agent_work_cycle.py end {agent_uid} --scope {scope}`.",
        ),
        checklist_item_line("workflow.share-plan", False, "Share a short plan and the current repo status before making changes."),
        checklist_item_line("workflow.mailbox-handoffs", False, "Use mailbox handoffs when work changes planning-relevant state."),
        checklist_item_line("workflow.commit-push", False, "Commit and push serially to `origin/main` if tracked changes land."),
        "",
        "## Role-Specific Responsibilities",
    ]

    if role == "doc":
        lines.extend(
            [
                checklist_item_line(
                    "role.doc.check-plan-refresh",
                    False,
                    "Run `scripts/check-plan-refresh.sh` after each completed doc work item while preparing next items.",
                ),
                checklist_item_line(
                    "role.doc.use-planning-sync-plan",
                    False,
                    f"If cadence is due, use `{relative_to_root(PLANNING_SYNC_PLAN_PATH)}` as the planning-sync entry point.",
                ),
                checklist_item_line(
                    "role.doc.scan-mailboxes",
                    False,
                    "Scan active, paused, and recently inactive mailboxes before `sync doc`, `sync web`, or `sync plan`.",
                ),
                checklist_item_line(
                    "role.doc.limit-to-evidence",
                    False,
                    "Keep roadmap/checklist/progress updates limited to landed evidence or mailbox handoffs.",
                ),
            ]
        )
    else:
        lines.extend(
            [
                checklist_item_line(
                    "role.coding.check-latest-ci",
                    False,
                    "Check the latest completed CI result for the previous push before new coding work.",
                ),
                checklist_item_line(
                    "role.coding.leave-continuation-handoff",
                    False,
                    "Leave one open `Work Continuation Handoff` in the coding mailbox at the end of the work item.",
                ),
                checklist_item_line(
                    "role.coding.leave-planning-handoff",
                    False,
                    "Leave a `Planning Sync Handoff` when coding changes affect roadmap/checklist/progress or issue-triage inputs.",
                ),
                checklist_item_line(
                    "role.coding.skip-plan-refresh",
                    False,
                    "Do not run `scripts/check-plan-refresh.sh` from a coding task.",
                ),
            ]
        )

    lines.extend(
        [
            "",
            "## Refresh",
            checklist_item_line(
                "refresh.regenerate",
                False,
                f"Re-run `scripts/agent_registry.py work-checklist {agent_uid}` whenever the agent status or scope changes.",
            ),
            "",
        ]
    )
    return "\n".join(lines)


def entry_summary(entry: dict[str, Any]) -> dict[str, Any]:
    return {
        "agent_uid": entry.get("agent_uid"),
        "display_id": current_display_id(entry),
        "last_display_id": last_display_id(entry),
        "role": entry.get("role"),
        "status": entry.get("status"),
    }


def append_display_assignment(entry: dict[str, Any], display_id: str, assigned_at: str) -> None:
    history = entry.setdefault("display_history", [])
    history.append(
        {
            "display_id": display_id,
            "assigned_at": assigned_at,
            "released_at": None,
            "released_reason": None,
        }
    )
    entry["current_display_id"] = display_id


def release_display_assignment(entry: dict[str, Any], released_at: str, released_reason: str) -> None:
    display_id = current_display_id(entry)
    if display_id is None:
        return
    history = entry.setdefault("display_history", [])
    for record in reversed(history):
        if not isinstance(record, dict):
            continue
        if record.get("display_id") == display_id and not record.get("released_at"):
            record["released_at"] = released_at
            record["released_reason"] = released_reason
            break
    entry["current_display_id"] = None


def ensure_entry_v2(entry: dict[str, Any]) -> tuple[dict[str, Any], bool]:
    changed = False
    if "agent_uid" not in entry:
        legacy_display_id = require_non_empty_str(entry, "id", "<legacy>")
        entry["agent_uid"] = make_legacy_agent_uid(legacy_display_id)
        changed = True
    if "current_display_id" not in entry:
        legacy_display_id = entry.get("id")
        entry["current_display_id"] = legacy_display_id if isinstance(legacy_display_id, str) else None
        changed = True
    if "display_history" not in entry:
        display_id = current_display_id(entry)
        entry["display_history"] = []
        if display_id is not None:
            entry["display_history"].append(
                {
                    "display_id": display_id,
                    "assigned_at": entry.get("assigned_at"),
                    "released_at": None,
                    "released_reason": None,
                }
            )
        changed = True
    if "recovery_of" not in entry:
        entry["recovery_of"] = None
        changed = True
    if "superseded_by" not in entry:
        entry["superseded_by"] = None
        changed = True
    if "paused_at" not in entry:
        entry["paused_at"] = None
        changed = True
    if entry.get("status") == "paused":
        paused_at = entry.get("paused_at")
        if not isinstance(paused_at, str) or not paused_at.strip():
            for field in ["last_touched_at", "confirmed_at", "assigned_at"]:
                candidate = entry.get(field)
                if isinstance(candidate, str) and candidate.strip():
                    entry["paused_at"] = candidate
                    changed = True
                    break
    elif entry.get("paused_at") is not None:
        entry["paused_at"] = None
        changed = True
    if "id" in entry:
        del entry["id"]
        changed = True
    return entry, changed


def migrate_registry_to_v2(registry: dict[str, Any]) -> tuple[dict[str, Any], bool]:
    changed = False
    if registry.get("version") != REGISTRY_VERSION:
        changed = True
    registry["version"] = REGISTRY_VERSION

    for entry in registry["agents"]:
        _, entry_changed = ensure_entry_v2(entry)
        changed = changed or entry_changed

        agent_uid = require_non_empty_str(entry, "agent_uid", "<agent>")
        mailbox_rel = mailbox_rel_for_uid(agent_uid)
        old_mailbox_value = entry.get("mailbox")
        old_mailbox_path = None
        if isinstance(old_mailbox_value, str) and old_mailbox_value.strip():
            old_mailbox_path = resolve_mailbox_path(old_mailbox_value)
        new_mailbox_path = resolve_mailbox_path(mailbox_rel)
        ensure_mailbox(new_mailbox_path, title=agent_uid, source_path=old_mailbox_path)
        if entry.get("mailbox") != mailbox_rel:
            entry["mailbox"] = mailbox_rel
            changed = True

    registry["agent_count"] = len(registry["agents"])
    if changed and not registry.get("updated_at"):
        registry["updated_at"] = utc_now()
    return registry, changed


def resolve_agent_entry(registry: dict[str, Any], identifier: str) -> dict[str, Any]:
    uid_matches = [entry for entry in registry["agents"] if entry.get("agent_uid") == identifier]
    if len(uid_matches) == 1:
        return uid_matches[0]
    if len(uid_matches) > 1:
        raise RegistryError(f"invalid registry: duplicate agent_uid {identifier}")

    display_matches = [entry for entry in registry["agents"] if current_display_id(entry) == identifier]
    if len(display_matches) == 1:
        return display_matches[0]
    if len(display_matches) > 1:
        raise RegistryError(f"invalid registry: duplicate current_display_id {identifier}")

    raise RegistryError(f"agent entry not found: {identifier}")


def choose_auto_role(registry: dict[str, Any]) -> str:
    active_counts = {role: 0 for role in ALLOWED_ROLES}
    for entry in registry["agents"]:
        entry_role = entry.get("role")
        entry_status = entry.get("status")
        if entry_role in ALLOWED_ROLES and entry_status == "active":
            active_counts[entry_role] += 1

    if active_counts["coding"] == 0:
        return "coding"
    if active_counts["doc"] == 0:
        return "doc"
    return "coding"


def next_display_id(registry: dict[str, Any], role: str) -> str:
    used_suffixes: set[int] = set()
    for entry in registry["agents"]:
        if entry.get("role") != role:
            continue
        display_id = current_display_id(entry)
        if not display_id or not display_id.startswith(f"{role}-"):
            continue
        suffix = display_id[len(role) + 1 :]
        if suffix.isdigit():
            used_suffixes.add(int(suffix))

    candidate = 1
    while candidate in used_suffixes:
        candidate += 1
    return f"{role}-{candidate}"


def apply_agent_lifecycle(
    registry: dict[str, Any], *, now: datetime | None = None
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], bool]:
    current_time = now or datetime.now(timezone.utc)
    release_time = format_timestamp(current_time)
    stale_agents: list[dict[str, Any]] = []
    removed_agents: list[dict[str, Any]] = []
    kept_agents: list[dict[str, Any]] = []
    changed = False

    for entry in registry["agents"]:
        status = entry.get("status")
        lifecycle_field = None
        stale_after_seconds = None
        cleanup_after_seconds = None
        if status == "inactive":
            lifecycle_field = "inactive_at"
            stale_after_seconds = STALE_INACTIVE_SECONDS
            cleanup_after_seconds = STALE_INACTIVE_SECONDS + STALE_RETENTION_SECONDS
        elif status == "paused":
            lifecycle_field = "paused_at"
            stale_after_seconds = STALE_PAUSED_SECONDS
            cleanup_after_seconds = PAUSED_CLEANUP_SECONDS

        lifecycle_at = entry.get(lifecycle_field) if lifecycle_field else None
        if lifecycle_field and isinstance(lifecycle_at, str) and lifecycle_at.strip():
            try:
                lifecycle_since = parse_utc_timestamp(lifecycle_at)
            except ValueError:
                lifecycle_since = None
            if lifecycle_since is not None:
                age_seconds = (current_time - lifecycle_since).total_seconds()
                if age_seconds >= cleanup_after_seconds:
                    removed_agents.append(entry_summary(entry))
                    changed = True
                    continue
                if age_seconds >= stale_after_seconds:
                    if current_display_id(entry) is not None:
                        release_display_assignment(entry, release_time, "stale-recycled")
                        changed = True
                    stale_agents.append(entry_summary(entry))
        kept_agents.append(entry)

    if len(kept_agents) != len(registry["agents"]):
        registry["agents"] = kept_agents
        changed = True

    if changed:
        registry["agent_count"] = len(registry["agents"])
        registry["updated_at"] = release_time

    return stale_agents, removed_agents, changed


def load_registry_with_cleanup(
    *, allow_missing: bool = False
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    registry = load_registry(allow_missing=allow_missing)
    registry, migrated = migrate_registry_to_v2(registry)
    stale_agents, removed_agents, lifecycle_changed = apply_agent_lifecycle(registry)
    if migrated or lifecycle_changed:
        save_registry(registry)
    return registry, stale_agents, removed_agents


def normalized_entry(entry: dict[str, Any]) -> dict[str, Any]:
    mailbox_value = entry.get("mailbox")
    mailbox_exists = False
    mailbox_display = None
    if isinstance(mailbox_value, str) and mailbox_value.strip():
        mailbox_path = resolve_mailbox_path(mailbox_value)
        mailbox_exists = mailbox_path.exists()
        mailbox_display = relative_to_root(mailbox_path)

    return {
        "agent_uid": entry.get("agent_uid"),
        "display_id": current_display_id(entry),
        "last_display_id": last_display_id(entry),
        "role": entry.get("role"),
        "status": entry.get("status"),
        "scope": entry.get("scope"),
        "assigned_by": entry.get("assigned_by"),
        "assigned_at": normalize_timestamp_string(entry.get("assigned_at")),
        "confirmed_by_agent": entry.get("confirmed_by_agent", False),
        "confirmed_at": normalize_timestamp_string(entry.get("confirmed_at")),
        "last_touched_at": normalize_timestamp_string(entry.get("last_touched_at")),
        "inactive_at": normalize_timestamp_string(entry.get("inactive_at")),
        "paused_at": normalize_timestamp_string(entry.get("paused_at")),
        "files": entry.get("files", []),
        "mailbox": mailbox_display,
        "mailbox_exists": mailbox_exists,
        "recovery_of": entry.get("recovery_of"),
        "superseded_by": entry.get("superseded_by"),
    }


def print_json(data: dict[str, Any]) -> None:
    print(json.dumps(data))


def print_claim(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"assigned_by: {data['assigned_by']}")
    print(f"assigned_at: {data['assigned_at']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"next: scripts/agent_registry.py start {data['agent_uid']}")


def print_start(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"mailbox_link: {data['mailbox_link']}")
    print(f"confirmed_at: {data['confirmed_at']}")
    if "bootstrap_output" in data:
        print(f"bootstrap_output: {data['bootstrap_output']}")
        print(f"bootstrap_created: {data['bootstrap_created']}")


def print_stop(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"previous_status: {data['previous_status']}")
    print(f"current_status: {data['current_status']}")
    print(f"updated_at: {data['updated_at']}")


def print_status(data: dict[str, Any], *, verbose: bool = False) -> None:
    print(f"registry: {data['registry_path']}")
    print(f"version: {data['version']}")
    print(f"updated_at: {data['updated_at']}")
    print(f"agents: {data['agent_count']}")
    if not verbose:
        if data.get("cleanup_removed_agents"):
            print(f"cleanup_removed_agents: {len(data['cleanup_removed_agents'])}")
        if data.get("stale_agents"):
            print(f"stale_agents: {len(data['stale_agents'])}")

        agents = data.get("agents", [])
        if len(agents) == 1:
            entry = agents[0]
            print(f"agent_uid: {entry['agent_uid']}")
            print(f"display_id: {entry['display_id']}")
            if entry.get("last_display_id") != entry.get("display_id"):
                print(f"last_display_id: {entry['last_display_id']}")
            print(f"role: {entry['role']}")
            print(f"status: {entry['status']}")
            print(f"scope: {entry['scope']}")
            print(f"mailbox: {entry['mailbox']}")
            print(f"last_touched_at: {entry['last_touched_at']}")
            return

        role_counts = Counter(entry.get("role") for entry in agents if entry.get("role"))
        status_counts = Counter(entry.get("status") for entry in agents if entry.get("status"))
        if role_counts:
            print(
                "role_counts: "
                + ", ".join(f"{role}={count}" for role, count in sorted(role_counts.items()))
            )
        if status_counts:
            print(
                "status_counts: "
                + ", ".join(f"{status}={count}" for status, count in sorted(status_counts.items()))
            )
        return

    if data.get("cleanup_removed_agents"):
        print("cleanup_removed_agents:")
        for entry in data["cleanup_removed_agents"]:
            print(f"  - {entry['agent_uid']} ({entry.get('last_display_id')})")
    if data.get("stale_agents"):
        print("stale_agents:")
        for entry in data["stale_agents"]:
            print(f"  - {entry['agent_uid']} ({entry.get('last_display_id')})")
    for entry in data["agents"]:
        print(f"agent_uid: {entry['agent_uid']}")
        print(f"  display_id: {entry['display_id']}")
        print(f"  last_display_id: {entry['last_display_id']}")
        print(f"  role: {entry['role']}")
        print(f"  status: {entry['status']}")
        print(f"  scope: {entry['scope']}")
        print(f"  assigned_by: {entry['assigned_by']}")
        print(f"  assigned_at: {entry['assigned_at']}")
        print(f"  confirmed_by_agent: {entry['confirmed_by_agent']}")
        print(f"  confirmed_at: {entry['confirmed_at']}")
        print(f"  last_touched_at: {entry['last_touched_at']}")
        print(f"  inactive_at: {entry['inactive_at']}")
        print(f"  paused_at: {entry['paused_at']}")
        print(f"  mailbox: {entry['mailbox']}")
        print(f"  mailbox_exists: {entry['mailbox_exists']}")
        print(f"  recovery_of: {entry['recovery_of']}")
        print(f"  superseded_by: {entry['superseded_by']}")
        files = entry.get("files", [])
        if files:
            print("  files:")
            for path in files:
                print(f"    - {path}")
        else:
            print("  files: []")


def print_resume_check(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"last_display_id: {data['last_display_id']}")
    print(f"role: {data['role']}")
    print(f"current_status: {data['current_status']}")
    print(f"scope: {data['scope']}")
    print(f"confirmed_by_agent: {data['confirmed_by_agent']}")
    print(f"confirmed_at: {data['confirmed_at']}")
    print(f"last_touched_at: {data['last_touched_at']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"safe_to_resume: {data['safe_to_resume']}")
    print(f"must_recover: {data['must_recover']}")
    print(f"recommended_action: {data['recommended_action']}")
    print(f"reason: {data['reason']}")


def print_recover(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"previous_display_id: {data['previous_display_id']}")
    print(f"recovered_display_id: {data['recovered_display_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"mailbox: {data['mailbox']}")
    print(f"updated_at: {data['updated_at']}")


def print_takeover(data: dict[str, Any]) -> None:
    print(f"stale_agent_uid: {data['stale_agent_uid']}")
    print(f"stale_display_id: {data['stale_display_id']}")
    print(f"replacement_agent_uid: {data['replacement_agent_uid']}")
    print(f"replacement_display_id: {data['replacement_display_id']}")
    print(f"role: {data['replacement_role']}")
    print(f"scope: {data['replacement_scope']}")
    print(f"replacement_mailbox: {data['replacement_mailbox']}")
    print(f"updated_at: {data['updated_at']}")
    print(f"takeover_note: {data['takeover_note']}")
    print("next: read the stale mailbox before resuming tracked work")


def print_touch(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"role: {data['role']}")
    print(f"previous_status: {data['previous_status']}")
    print(f"current_status: {data['current_status']}")
    print(f"last_touched_at: {data['last_touched_at']}")


def print_finish(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"previous_status: {data['previous_status']}")
    print(f"current_status: {data['current_status']}")
    print(f"inactive_at: {data['inactive_at']}")


def print_cleanup(data: dict[str, Any]) -> None:
    print(f"removed_agents: {data['removed_count']}")
    for entry in data["removed_agents"]:
        print(f"  - {entry['agent_uid']} ({entry.get('last_display_id')})")
    print(f"stale_agents: {data['stale_count']}")
    for entry in data["stale_agents"]:
        print(f"  - {entry['agent_uid']} ({entry.get('last_display_id')})")


def print_work_checklist(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"display_id: {data['display_id']}")
    print(f"role: {data['role']}")
    print(f"scope: {data['scope']}")
    print(f"output: {data['output']}")
    print(f"updated_at: {data['updated_at']}")


def print_work_checklist_mark(data: dict[str, Any]) -> None:
    print(f"agent_uid: {data['agent_uid']}")
    print(f"item_id: {data['item_id']}")
    print(f"state: {data['state']}")
    print(f"output: {data['output']}")
    print(f"updated_at: {data['updated_at']}")


def cmd_claim(args: argparse.Namespace) -> int:
    if args.role != "auto" and args.role not in ALLOWED_ROLES:
        raise RegistryError(f"unsupported role: {args.role}")
    if not args.assigned_by.strip():
        raise RegistryError("assigned_by must not be empty")
    if not args.scope.strip():
        raise RegistryError("scope must not be empty")

    registry, _, _ = load_registry_with_cleanup(allow_missing=True)
    role = choose_auto_role(registry) if args.role == "auto" else args.role
    agent_uid = make_new_agent_uid(registry)
    display_id = next_display_id(registry, role)
    mailbox_rel = mailbox_rel_for_uid(agent_uid)
    mailbox_path = resolve_mailbox_path(mailbox_rel)
    ensure_mailbox(mailbox_path, title=agent_uid)

    now = utc_now()
    entry = {
        "agent_uid": agent_uid,
        "role": role,
        "current_display_id": display_id,
        "display_history": [],
        "assigned_by": args.assigned_by,
        "assigned_at": now,
        "confirmed_by_agent": False,
        "confirmed_at": None,
        "status": "paused",
        "scope": args.scope,
        "files": [],
        "mailbox": mailbox_rel,
        "last_touched_at": None,
        "inactive_at": None,
        "paused_at": now,
        "recovery_of": None,
        "superseded_by": None,
    }
    append_display_assignment(entry, display_id, now)
    registry["agents"].append(entry)
    registry["agent_count"] = len(registry["agents"])
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "role": role,
        "scope": args.scope,
        "assigned_by": args.assigned_by,
        "assigned_at": now,
        "mailbox": mailbox_rel,
    }
    if args.json:
        print_json(result)
    else:
        print_claim(result)
    return 0


def cmd_start(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)

    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    display_id = current_display_id(entry)
    if display_id is None:
        raise RegistryError(f"agent {agent_uid} has no active display_id; recover it before start")

    for field in ["role", "assigned_by", "assigned_at", "scope", "mailbox"]:
        require_non_empty_str(entry, field, agent_uid)

    status = require_status(entry, agent_uid)
    if status == "done":
        raise RegistryError(f"agent {agent_uid} cannot start because status is done")
    if status == "blocked":
        raise RegistryError(f"agent {agent_uid} cannot start because status is blocked")

    files = entry.get("files")
    if not isinstance(files, list):
        raise RegistryError(f"agent {agent_uid} is missing required field: files")

    now = utc_now()
    entry["confirmed_by_agent"] = True
    entry["confirmed_at"] = now
    entry["status"] = "active"
    entry["last_touched_at"] = now
    entry["inactive_at"] = None
    entry["paused_at"] = None
    registry["updated_at"] = now

    mailbox_path = resolve_mailbox_path(require_non_empty_str(entry, "mailbox", agent_uid))
    ensure_mailbox(mailbox_path, title=agent_uid)
    mailbox_link = ensure_agent_mailbox_symlink(agent_uid, mailbox_path)
    save_registry(registry)

    bootstrap_path = agents_bootstrap_checklist_path(agent_uid)
    bootstrap_created = False
    if not bootstrap_path.exists():
        checklist_result = materialize_checklist(
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=AGENTS_PATH,
            output_path=bootstrap_path,
            section="bootstrap",
        )
        bootstrap_path = ROOT_DIR / require_non_empty_str(checklist_result, "output", agent_uid)
        bootstrap_created = True

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "role": entry["role"],
        "scope": entry["scope"],
        "mailbox": relative_to_root(mailbox_path),
        "mailbox_link": relative_to_root(mailbox_link),
        "confirmed_at": now,
        "bootstrap_output": relative_to_root(bootstrap_path),
        "bootstrap_created": bootstrap_created,
    }
    if args.json:
        print_json(result)
    else:
        print_start(result)
    return 0


def cmd_stop(args: argparse.Namespace) -> int:
    if args.status not in {"paused", "done"}:
        raise RegistryError(f"unsupported stop status: {args.status}")

    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)
    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    display_id = current_display_id(entry)
    previous_status = require_status(entry, agent_uid)

    now = utc_now()
    entry["status"] = args.status
    entry["inactive_at"] = None
    entry["paused_at"] = now if args.status == "paused" else None
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "previous_status": previous_status,
        "current_status": args.status,
        "updated_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_stop(result)
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    registry, stale_agents, removed_agents = load_registry_with_cleanup()
    selected = registry["agents"]

    if args.agent_ref:
        selected = [resolve_agent_entry(registry, args.agent_ref)]

    result = {
        "status": "ok",
        "registry_path": str(REGISTRY_PATH),
        "version": registry.get("version"),
        "updated_at": normalize_timestamp_string(registry.get("updated_at")),
        "agent_count": len(selected) if args.agent_ref else len(registry["agents"]),
        "cleanup_removed_agents": removed_agents,
        "stale_agents": stale_agents,
        "agents": [normalized_entry(entry) for entry in selected],
    }
    if args.json:
        print_json(result)
    else:
        print_status(result, verbose=args.verbose)
    return 0


def cmd_resume_check(args: argparse.Namespace) -> int:
    registry, stale_agents, removed_agents = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)

    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    role = require_non_empty_str(entry, "role", agent_uid)
    status = require_status(entry, agent_uid)
    scope = require_non_empty_str(entry, "scope", agent_uid)
    mailbox_value = require_non_empty_str(entry, "mailbox", agent_uid)
    confirmed_by_agent = entry.get("confirmed_by_agent", False)
    confirmed_at = entry.get("confirmed_at")
    last_touched_at = entry.get("last_touched_at")
    display_id = current_display_id(entry)
    last_known_display_id = last_display_id(entry)

    mailbox_path = resolve_mailbox_path(mailbox_value)
    reason = "agent is still active and confirmed"
    safe_to_resume = True
    must_recover = False
    recommended_action = "touch"
    exit_code = 0

    if status not in {"active", "inactive"}:
        safe_to_resume = False
        recommended_action = "stop"
        reason = f"agent status is {status}; do not resume tracked work under {agent_uid}"
        exit_code = 2
    elif confirmed_by_agent is not True or not isinstance(confirmed_at, str) or not confirmed_at.strip():
        safe_to_resume = False
        recommended_action = "start"
        reason = f"agent {agent_uid} is not fully confirmed; do not resume tracked work"
        exit_code = 2
    elif display_id is None:
        safe_to_resume = False
        must_recover = True
        recommended_action = "recover"
        reason = f"agent {agent_uid} no longer holds a display slot; recover it before resuming work"
        exit_code = 2
    elif status == "inactive":
        reason = f"agent {agent_uid} is inactive but still holds {display_id}; it may resume by touching its own entry"

    result = {
        "status": "ok" if safe_to_resume else "stop",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "last_display_id": last_known_display_id,
        "role": role,
        "current_status": status,
        "scope": scope,
        "confirmed_by_agent": bool(confirmed_by_agent),
        "confirmed_at": normalize_timestamp_string(confirmed_at),
        "last_touched_at": normalize_timestamp_string(last_touched_at),
        "mailbox": relative_to_root(mailbox_path),
        "safe_to_resume": safe_to_resume,
        "must_recover": must_recover,
        "recommended_action": recommended_action,
        "cleanup_removed_agents": removed_agents,
        "stale_agents": stale_agents,
        "reason": reason,
    }
    if args.json:
        print_json(result)
    else:
        print_resume_check(result)
    return exit_code


def cmd_recover(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)

    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    role = require_non_empty_str(entry, "role", agent_uid)
    if role not in ALLOWED_ROLES:
        raise RegistryError(f"unsupported role in agent entry: {role}")
    if require_status(entry, agent_uid) == "done":
        raise RegistryError(f"agent {agent_uid} cannot be recovered because status is done")
    if current_display_id(entry) is not None:
        raise RegistryError(f"agent {agent_uid} already holds {current_display_id(entry)}; recover is not needed")

    scope = args.scope or require_non_empty_str(entry, "scope", agent_uid)
    previous_display_id = last_display_id(entry)
    recovered_display_id = next_display_id(registry, role)
    now = utc_now()

    append_display_assignment(entry, recovered_display_id, now)
    entry["status"] = "active"
    entry["scope"] = scope
    entry["last_touched_at"] = now
    entry["inactive_at"] = None
    entry["paused_at"] = None
    if entry.get("confirmed_by_agent") is not True:
        entry["confirmed_by_agent"] = True
        entry["confirmed_at"] = now
    registry["updated_at"] = now

    mailbox_path = resolve_mailbox_path(require_non_empty_str(entry, "mailbox", agent_uid))
    ensure_mailbox(mailbox_path, title=agent_uid)
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "previous_display_id": previous_display_id,
        "recovered_display_id": recovered_display_id,
        "role": role,
        "scope": scope,
        "mailbox": relative_to_root(mailbox_path),
        "updated_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_recover(result)
    return 0


def cmd_takeover(args: argparse.Namespace) -> int:
    if not args.assigned_by.strip():
        raise RegistryError("assigned_by must not be empty")

    registry, _, _ = load_registry_with_cleanup()
    stale_entry = resolve_agent_entry(registry, args.stale_agent_ref)

    stale_agent_uid = require_non_empty_str(stale_entry, "agent_uid", args.stale_agent_ref)
    stale_role = require_non_empty_str(stale_entry, "role", stale_agent_uid)
    stale_status = require_status(stale_entry, stale_agent_uid)
    if stale_status == "done":
        raise RegistryError(f"agent {stale_agent_uid} cannot be taken over because status is done")

    scope = args.scope or require_non_empty_str(stale_entry, "scope", stale_agent_uid)
    stale_mailbox_value = require_non_empty_str(stale_entry, "mailbox", stale_agent_uid)
    stale_mailbox_path = resolve_mailbox_path(stale_mailbox_value)
    replacement_agent_uid = make_new_agent_uid(registry)
    replacement_display_id = next_display_id(registry, stale_role)
    replacement_mailbox_rel = mailbox_rel_for_uid(replacement_agent_uid)
    replacement_mailbox_path = resolve_mailbox_path(replacement_mailbox_rel)
    now = utc_now()

    replacement_entry = {
        "agent_uid": replacement_agent_uid,
        "role": stale_role,
        "current_display_id": replacement_display_id,
        "display_history": [],
        "assigned_by": args.assigned_by,
        "assigned_at": now,
        "confirmed_by_agent": True,
        "confirmed_at": now,
        "status": "active",
        "scope": scope,
        "files": list(stale_entry.get("files", [])) if isinstance(stale_entry.get("files"), list) else [],
        "mailbox": replacement_mailbox_rel,
        "last_touched_at": now,
        "inactive_at": None,
        "paused_at": None,
        "recovery_of": stale_agent_uid,
        "superseded_by": None,
    }
    append_display_assignment(replacement_entry, replacement_display_id, now)

    stale_entry["status"] = "paused"
    stale_entry["inactive_at"] = None
    stale_entry["paused_at"] = now
    stale_entry["superseded_by"] = replacement_agent_uid
    registry["agents"].append(replacement_entry)
    registry["agent_count"] = len(registry["agents"])
    registry["updated_at"] = now

    ensure_mailbox(replacement_mailbox_path, title=replacement_agent_uid)
    takeover_note = f"taking over from {stale_agent_uid} after interrupted chat"
    existing_mailbox = replacement_mailbox_path.read_text(encoding="utf-8")
    if takeover_note not in existing_mailbox:
        replacement_mailbox_path.write_text(existing_mailbox + f"- {takeover_note}\n", encoding="utf-8")

    save_registry(registry)

    result = {
        "status": "ok",
        "stale_agent_uid": stale_agent_uid,
        "stale_display_id": current_display_id(stale_entry),
        "stale_status": stale_status,
        "stale_mailbox": relative_to_root(stale_mailbox_path),
        "replacement_agent_uid": replacement_agent_uid,
        "replacement_display_id": replacement_display_id,
        "replacement_role": stale_role,
        "replacement_scope": scope,
        "replacement_mailbox": replacement_mailbox_rel,
        "assigned_by": args.assigned_by,
        "updated_at": now,
        "takeover_note": takeover_note,
    }
    if args.json:
        print_json(result)
    else:
        print_takeover(result)
    return 0


def cmd_touch(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)

    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    role = require_non_empty_str(entry, "role", agent_uid)
    previous_status = require_status(entry, agent_uid)
    display_id = current_display_id(entry)
    if previous_status == "done":
        raise RegistryError(f"agent {agent_uid} cannot be touched because status is done")
    if display_id is None:
        raise RegistryError(f"agent {agent_uid} has no active display_id; recover it before touch")

    confirmed_by_agent = entry.get("confirmed_by_agent", False)
    confirmed_at = entry.get("confirmed_at")
    if confirmed_by_agent is not True or not isinstance(confirmed_at, str) or not confirmed_at.strip():
        raise RegistryError(f"agent {agent_uid} is not fully confirmed; use start before touch")

    now = utc_now()
    entry["status"] = "active"
    entry["last_touched_at"] = now
    entry["inactive_at"] = None
    entry["paused_at"] = None
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "role": role,
        "previous_status": previous_status,
        "current_status": "active",
        "last_touched_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_touch(result)
    return 0


def cmd_finish(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)

    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    display_id = current_display_id(entry)
    previous_status = require_status(entry, agent_uid)
    if previous_status == "done":
        raise RegistryError(f"agent {agent_uid} cannot be finished because status is done")

    now = utc_now()
    entry["status"] = "inactive"
    entry["inactive_at"] = now
    entry["paused_at"] = None
    registry["updated_at"] = now
    save_registry(registry)

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": display_id,
        "previous_status": previous_status,
        "current_status": "inactive",
        "inactive_at": now,
    }
    if args.json:
        print_json(result)
    else:
        print_finish(result)
    return 0


def cmd_cleanup(args: argparse.Namespace) -> int:
    registry, stale_agents, removed_agents = load_registry_with_cleanup()
    result = {
        "status": "ok",
        "removed_count": len(removed_agents),
        "removed_agents": removed_agents,
        "stale_count": len(stale_agents),
        "stale_agents": stale_agents,
        "updated_at": normalize_timestamp_string(registry.get("updated_at")),
    }
    if args.json:
        print_json(result)
    else:
        print_cleanup(result)
    return 0


def cmd_work_checklist(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)
    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    output_path = resolve_agent_local_path(args.output or checklist_rel_for_uid(agent_uid))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    generated_at = utc_now()
    output_path.write_text(render_work_checklist(entry, generated_at=generated_at), encoding="utf-8")

    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "display_id": current_display_id(entry),
        "role": entry.get("role"),
        "scope": entry.get("scope"),
        "output": relative_to_root(output_path),
        "updated_at": generated_at,
    }
    if args.json:
        print_json(result)
    else:
        print_work_checklist(result)
    return 0


def cmd_work_checklist_mark(args: argparse.Namespace) -> int:
    registry, _, _ = load_registry_with_cleanup()
    entry = resolve_agent_entry(registry, args.agent_ref)
    agent_uid = require_non_empty_str(entry, "agent_uid", args.agent_ref)
    checklist_path = (
        resolve_agent_local_path(args.checklist)
        if args.checklist
        else resolve_existing_work_checklist_path(agent_uid)
    )
    if not checklist_path.exists():
        raise RegistryError(f"checklist file not found: {relative_to_root(checklist_path)}")

    lines = checklist_path.read_text(encoding="utf-8").splitlines()
    try:
        updates = apply_updates(lines=lines, updates=[(args.item_id, args.state)], problem_overrides={})
    except ItemIdChecklistMarkError as exc:
        raise RegistryError(str(exc)) from exc

    if updates:
        checklist_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    state = updates[0]["state"]
    result = {
        "status": "ok",
        "agent_uid": agent_uid,
        "item_id": args.item_id,
        "state": state,
        "output": relative_to_root(checklist_path),
        "updated_at": utc_now(),
    }
    if args.json:
        print_json(result)
    else:
        print_work_checklist_mark(result)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_registry.py",
        description="Manage the local agent registry.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    claim = subparsers.add_parser("claim", add_help=False)
    claim.add_argument("role")
    claim.add_argument("--scope", default="pending scope")
    claim.add_argument("--assigned-by", default="user")
    claim.add_argument("--json", action="store_true")
    claim.add_argument("-h", "--help", action="help")
    claim.set_defaults(func=cmd_claim)

    start = subparsers.add_parser("start", add_help=False)
    start.add_argument("agent_ref")
    start.add_argument("--json", action="store_true")
    start.add_argument("-h", "--help", action="help")
    start.set_defaults(func=cmd_start)

    stop = subparsers.add_parser("stop", add_help=False)
    stop.add_argument("agent_ref")
    stop.add_argument("--status", default="paused")
    stop.add_argument("--json", action="store_true")
    stop.add_argument("-h", "--help", action="help")
    stop.set_defaults(func=cmd_stop)

    status = subparsers.add_parser("status", add_help=False)
    status.add_argument("agent_ref", nargs="?")
    status.add_argument("--verbose", action="store_true")
    status.add_argument("--json", action="store_true")
    status.add_argument("-h", "--help", action="help")
    status.set_defaults(func=cmd_status)

    resume_check = subparsers.add_parser("resume-check", add_help=False)
    resume_check.add_argument("agent_ref")
    resume_check.add_argument("--json", action="store_true")
    resume_check.add_argument("-h", "--help", action="help")
    resume_check.set_defaults(func=cmd_resume_check)

    recover = subparsers.add_parser("recover", add_help=False)
    recover.add_argument("agent_ref")
    recover.add_argument("--scope", default="")
    recover.add_argument("--json", action="store_true")
    recover.add_argument("-h", "--help", action="help")
    recover.set_defaults(func=cmd_recover)

    takeover = subparsers.add_parser("takeover", add_help=False)
    takeover.add_argument("stale_agent_ref")
    takeover.add_argument("--scope", default="")
    takeover.add_argument("--assigned-by", default="user")
    takeover.add_argument("--json", action="store_true")
    takeover.add_argument("-h", "--help", action="help")
    takeover.set_defaults(func=cmd_takeover)

    touch = subparsers.add_parser("touch", add_help=False)
    touch.add_argument("agent_ref")
    touch.add_argument("--json", action="store_true")
    touch.add_argument("-h", "--help", action="help")
    touch.set_defaults(func=cmd_touch)

    finish = subparsers.add_parser("finish", add_help=False)
    finish.add_argument("agent_ref")
    finish.add_argument("--json", action="store_true")
    finish.add_argument("-h", "--help", action="help")
    finish.set_defaults(func=cmd_finish)

    cleanup = subparsers.add_parser("cleanup", add_help=False)
    cleanup.add_argument("--json", action="store_true")
    cleanup.add_argument("-h", "--help", action="help")
    cleanup.set_defaults(func=cmd_cleanup)

    work_checklist = subparsers.add_parser("work-checklist", add_help=False)
    work_checklist.add_argument("agent_ref")
    work_checklist.add_argument("--output", default="")
    work_checklist.add_argument("--json", action="store_true")
    work_checklist.add_argument("-h", "--help", action="help")
    work_checklist.set_defaults(func=cmd_work_checklist)

    work_checklist_mark = subparsers.add_parser("work-checklist-mark", add_help=False)
    work_checklist_mark.add_argument("agent_ref")
    work_checklist_mark.add_argument("item_id")
    work_checklist_mark.add_argument("--state", choices=["checked", "unchecked", "toggle"], default="checked")
    work_checklist_mark.add_argument("--checklist", default="")
    work_checklist_mark.add_argument("--json", action="store_true")
    work_checklist_mark.add_argument("-h", "--help", action="help")
    work_checklist_mark.set_defaults(func=cmd_work_checklist_mark)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except RegistryError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
