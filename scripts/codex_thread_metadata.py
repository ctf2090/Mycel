#!/usr/bin/env python3
"""Print model and reasoning effort for the latest Codex thread in a cwd."""

from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path
from typing import Any


CODEX_HOME = Path.home() / ".codex"
DEFAULT_STATE_DB = CODEX_HOME / "state_5.sqlite"
DEFAULT_SESSIONS_DIR = CODEX_HOME / "sessions"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Locate the latest Codex session for a cwd and print model and "
            "reasoning effort."
        )
    )
    parser.add_argument(
        "--cwd",
        default=str(Path.cwd()),
        help="Working directory to match against Codex session turn_context cwd.",
    )
    parser.add_argument(
        "--codex-home",
        default=str(CODEX_HOME),
        help="Codex home directory. Defaults to ~/.codex.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit JSON instead of a human-readable summary.",
    )
    return parser.parse_args()


def thread_id_from_rollout_path(path: Path) -> str:
    name = path.stem
    if not name.startswith("rollout-"):
        raise ValueError(f"Unexpected rollout filename: {path}")
    parts = name.split("-")
    if len(parts) < 6:
        raise ValueError(f"Unexpected rollout filename: {path}")
    return "-".join(parts[-5:])


def load_latest_turn_context(sessions_dir: Path, cwd: str) -> dict[str, Any]:
    latest: dict[str, Any] | None = None
    latest_ts = ""
    for path in sorted(sessions_dir.rglob("rollout-*.jsonl")):
        with path.open("r", encoding="utf-8") as handle:
            for raw_line in handle:
                line = raw_line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if entry.get("type") != "turn_context":
                    continue
                payload = entry.get("payload", {})
                if payload.get("cwd") != cwd:
                    continue
                timestamp = entry.get("timestamp", "")
                if timestamp >= latest_ts:
                    latest_ts = timestamp
                    latest = {
                        "timestamp": timestamp,
                        "session_path": str(path),
                        "thread_id": thread_id_from_rollout_path(path),
                        "turn_id": payload.get("turn_id"),
                        "cwd": payload.get("cwd"),
                        "model": payload.get("model"),
                        "effort": payload.get("effort")
                        or payload.get("collaboration_mode", {})
                        .get("settings", {})
                        .get("reasoning_effort"),
                    }
    if latest is None:
        raise SystemExit(
            f"Could not find a turn_context entry under {sessions_dir} for cwd {cwd!r}."
        )
    return latest


def load_thread_row(state_db: Path, thread_id: str) -> dict[str, Any] | None:
    query = """
        SELECT id, cwd, model, reasoning_effort, updated_at
        FROM threads
        WHERE id = ?
    """
    with sqlite3.connect(state_db) as conn:
        conn.row_factory = sqlite3.Row
        row = conn.execute(query, (thread_id,)).fetchone()
    return dict(row) if row is not None else None


def main() -> int:
    args = parse_args()
    codex_home = Path(args.codex_home).expanduser()
    sessions_dir = codex_home / "sessions"
    state_db = codex_home / "state_5.sqlite"
    turn = load_latest_turn_context(sessions_dir, args.cwd)
    thread = load_thread_row(state_db, turn["thread_id"])

    result = {
        "cwd": args.cwd,
        "thread_id": turn["thread_id"],
        "turn_id": turn["turn_id"],
        "session_path": turn["session_path"],
        "session_timestamp": turn["timestamp"],
        "session_model": turn["model"],
        "session_effort": turn["effort"],
        "thread_model": None if thread is None else thread.get("model"),
        "thread_reasoning_effort": None
        if thread is None
        else thread.get("reasoning_effort"),
    }

    if args.json:
        print(json.dumps(result, ensure_ascii=True, indent=2))
        return 0

    print(f"cwd: {result['cwd']}")
    print(f"thread_id: {result['thread_id']}")
    print(f"turn_id: {result['turn_id']}")
    print(f"session_path: {result['session_path']}")
    print(f"session_timestamp: {result['session_timestamp']}")
    print(f"session_model: {result['session_model']}")
    print(f"session_effort: {result['session_effort']}")
    print(f"thread_model: {result['thread_model']}")
    print(f"thread_reasoning_effort: {result['thread_reasoning_effort']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
