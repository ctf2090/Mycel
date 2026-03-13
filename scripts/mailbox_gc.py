#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Iterable
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
MAILBOX_DIR = ROOT_DIR / ".agent-local" / "mailboxes"
LEGACY_ARCHIVE_DIR = MAILBOX_DIR / "archive"
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))
DEFAULT_DELETE_AGE_DAYS = 3


class MailboxGcError(Exception):
    pass


def format_timestamp(dt: datetime) -> str:
    return dt.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def load_registry() -> dict[str, Any]:
    try:
        payload = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise MailboxGcError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise MailboxGcError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise MailboxGcError("invalid registry: top-level JSON value must be an object")
    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise MailboxGcError("invalid registry: agents must be an array")
    return payload


def registry_mailboxes(registry: dict[str, Any]) -> dict[str, dict[str, Any]]:
    mapping: dict[str, dict[str, Any]] = {}
    for entry in registry["agents"]:
        if not isinstance(entry, dict):
            continue
        mailbox = entry.get("mailbox")
        if not isinstance(mailbox, str) or not mailbox.strip():
            continue
        mailbox_path = ROOT_DIR / mailbox
        mapping[str(mailbox_path.resolve())] = {
            "agent_uid": entry.get("agent_uid"),
            "status": entry.get("status"),
            "mailbox": mailbox,
        }
    return mapping


def is_tracked_example(path: Path) -> bool:
    return path.name.startswith("EXAMPLE-")


def live_mailbox_files() -> list[Path]:
    if not MAILBOX_DIR.exists():
        return []
    return sorted(
        path
        for path in MAILBOX_DIR.glob("*.md")
        if path.is_file() and not is_tracked_example(path)
    )


def legacy_archived_mailbox_files() -> list[Path]:
    if not LEGACY_ARCHIVE_DIR.exists():
        return []
    return sorted(path for path in LEGACY_ARCHIVE_DIR.rglob("*.md") if path.is_file())


def mailbox_record(path: Path, *, now: datetime, extra: dict[str, Any] | None = None) -> dict[str, Any]:
    stat = path.stat()
    modified_at = datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc)
    age_days = int((now - modified_at).total_seconds() // 86400)
    record = {
        "path": relative_to_root(path),
        "mtime": format_timestamp(modified_at),
        "age_days": age_days,
        "size_bytes": stat.st_size,
    }
    if extra:
        record.update(extra)
    return record


def deletion_candidates(records: list[dict[str, Any]], *, min_age_days: int) -> list[dict[str, Any]]:
    return [record for record in records if record["age_days"] >= min_age_days]


def scan_mailboxes(*, delete_age_days: int = DEFAULT_DELETE_AGE_DAYS) -> dict[str, Any]:
    registry = load_registry()
    referenced = registry_mailboxes(registry)
    live_files = live_mailbox_files()
    legacy_archived = legacy_archived_mailbox_files()
    now = datetime.now(timezone.utc)

    referenced_existing: list[dict[str, Any]] = []
    missing_referenced: list[dict[str, Any]] = []
    orphaned: list[dict[str, Any]] = []
    legacy_archived_records = [mailbox_record(path, now=now) for path in legacy_archived]

    live_by_resolved = {str(path.resolve()): path for path in live_files}

    for resolved_path, entry in sorted(referenced.items(), key=lambda item: item[1]["mailbox"]):
        live_path = live_by_resolved.get(resolved_path)
        if live_path is None:
            missing_referenced.append(
                {
                    "mailbox": entry["mailbox"],
                    "agent_uid": entry["agent_uid"],
                    "status": entry["status"],
                }
            )
            continue
        referenced_existing.append(
            mailbox_record(
                live_path,
                now=now,
                extra={
                    "agent_uid": entry["agent_uid"],
                    "status": entry["status"],
                },
            )
        )

    for path in live_files:
        if str(path.resolve()) in referenced:
            continue
        orphaned.append(mailbox_record(path, now=now))

    delete_candidates = deletion_candidates(orphaned + legacy_archived_records, min_age_days=delete_age_days)

    return {
        "status": "ok",
        "mailbox_dir": relative_to_root(MAILBOX_DIR),
        "legacy_archive_dir": relative_to_root(LEGACY_ARCHIVE_DIR),
        "referenced_count": len(referenced_existing),
        "missing_referenced_count": len(missing_referenced),
        "orphaned_count": len(orphaned),
        "legacy_archived_count": len(legacy_archived_records),
        "delete_candidate_count": len(delete_candidates),
        "delete_age_days": delete_age_days,
        "referenced": referenced_existing,
        "missing_referenced": missing_referenced,
        "orphaned": orphaned,
        "legacy_archived": legacy_archived_records,
        "delete_candidates": delete_candidates,
    }


def delete_stale_mailboxes(*, dry_run: bool, min_age_days: int) -> dict[str, Any]:
    scan = scan_mailboxes(delete_age_days=min_age_days)
    deleted: list[dict[str, Any]] = []

    for record in scan["delete_candidates"]:
        path = ROOT_DIR / record["path"]
        deleted.append({"path": record["path"], "age_days": record["age_days"]})
        if dry_run or not path.exists():
            continue
        path.unlink()

    return {
        "status": "ok",
        "dry_run": dry_run,
        "min_age_days": min_age_days,
        "deleted_count": len(deleted),
        "deleted": deleted,
    }


def print_scan(data: dict[str, Any]) -> None:
    print(f"mailbox_dir: {data['mailbox_dir']}")
    print(f"legacy_archive_dir: {data['legacy_archive_dir']}")
    print(f"referenced_mailboxes: {data['referenced_count']}")
    print(f"missing_referenced_mailboxes: {data['missing_referenced_count']}")
    print(f"orphaned_mailboxes: {data['orphaned_count']}")
    print(f"legacy_archived_mailboxes: {data['legacy_archived_count']}")
    print(f"delete_candidates: {data['delete_candidate_count']}")
    print(f"delete_age_days: {data['delete_age_days']}")
    if data["missing_referenced"]:
        print("missing_referenced:")
        for record in data["missing_referenced"]:
            print(f"  - {record['mailbox']} ({record['agent_uid']}, {record['status']})")
    if data["orphaned"]:
        print("orphaned:")
        for record in data["orphaned"]:
            print(f"  - {record['path']} ({record['age_days']} days)")
    if data["legacy_archived"]:
        print("legacy_archived:")
        for record in data["legacy_archived"]:
            print(f"  - {record['path']} ({record['age_days']} days)")
    if data["delete_candidates"]:
        print("delete_candidates:")
        for record in data["delete_candidates"]:
            print(f"  - {record['path']} ({record['age_days']} days)")


def print_prune(data: dict[str, Any]) -> None:
    print(f"dry_run: {data['dry_run']}")
    print(f"min_age_days: {data['min_age_days']}")
    print(f"deleted_mailboxes: {data['deleted_count']}")
    for record in data["deleted"]:
        print(f"- {record['path']} ({record['age_days']} days)")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/mailbox_gc.py",
        description="Inspect and delete unreferenced uid-based agent mailboxes.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    scan = subparsers.add_parser("scan", add_help=False)
    scan.add_argument("--delete-age-days", type=int, default=DEFAULT_DELETE_AGE_DAYS)
    scan.add_argument("--json", action="store_true")
    scan.add_argument("-h", "--help", action="help")
    scan.set_defaults(func=cmd_scan)

    prune = subparsers.add_parser("prune", add_help=False)
    prune.add_argument("--dry-run", action="store_true")
    prune.add_argument("--min-age-days", type=int, default=DEFAULT_DELETE_AGE_DAYS)
    prune.add_argument("--json", action="store_true")
    prune.add_argument("-h", "--help", action="help")
    prune.set_defaults(func=cmd_prune)

    return parser


def cmd_scan(args: argparse.Namespace) -> int:
    result = scan_mailboxes(delete_age_days=args.delete_age_days)
    if args.json:
        print(json.dumps(result))
    else:
        print_scan(result)
    return 0


def cmd_prune(args: argparse.Namespace) -> int:
    result = delete_stale_mailboxes(dry_run=args.dry_run, min_age_days=args.min_age_days)
    if args.json:
        print(json.dumps(result))
    else:
        print_prune(result)
    return 0


def main(argv: Iterable[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return args.func(args)
    except MailboxGcError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
