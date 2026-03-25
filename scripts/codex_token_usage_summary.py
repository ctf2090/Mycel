#!/usr/bin/env python3
"""Summarize per-turn Codex token usage from session JSONL files."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


CODEX_HOME = Path.home() / ".codex"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Read the latest matching Codex session JSONL for a cwd and print "
            "per-turn token usage."
        )
    )
    parser.add_argument(
        "--cwd",
        default=str(Path.cwd()),
        help="Working directory to match against Codex turn_context cwd.",
    )
    parser.add_argument(
        "--codex-home",
        default=str(CODEX_HOME),
        help="Codex home directory. Defaults to ~/.codex.",
    )
    parser.add_argument(
        "--thread-id",
        help="Specific thread_id to inspect. If omitted, use the latest thread matching --cwd.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of recent token_count rows to print.",
    )
    parser.add_argument(
        "--last-turn",
        action="store_true",
        help=(
            "Emit only the latest completed token_count row for the selected "
            "thread. When --thread-id is omitted, resolve the current thread "
            "from the latest rollout matching --cwd."
        ),
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit JSON instead of a table.",
    )
    parser.add_argument(
        "--full-numbers",
        action="store_true",
        help="Render full token counts instead of compact K units.",
    )
    parser.add_argument(
        "--last-turn-total-only",
        action="store_true",
        help="Emit only the latest row's last-turn total token count as an integer.",
    )
    return parser.parse_args()


def find_latest_rollout_path(sessions_dir: Path, cwd: str) -> Path:
    latest_path: Path | None = None
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
                    latest_path = path
    if latest_path is None:
        raise SystemExit(
            f"Could not find any rollout JSONL under {sessions_dir} for cwd {cwd!r}."
        )
    return latest_path


def find_rollout_path_by_thread_id(sessions_dir: Path, thread_id: str) -> Path:
    matches = sorted(sessions_dir.rglob(f"rollout-*{thread_id}.jsonl"))
    if not matches:
        raise SystemExit(
            f"Could not find any rollout JSONL under {sessions_dir} for thread_id {thread_id!r}."
        )
    return matches[-1]


def thread_id_from_rollout_path(rollout_path: Path) -> str:
    return "-".join(rollout_path.stem.rsplit("-", 5)[-5:])


def load_usage_rows(rollout_path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for raw_line in rollout_path.open("r", encoding="utf-8"):
        line = raw_line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        payload = entry.get("payload", {})
        if entry.get("type") != "event_msg" or payload.get("type") != "token_count":
            continue
        info = payload.get("info")
        if not info:
            continue
        last = info.get("last_token_usage")
        total = info.get("total_token_usage")
        if not last or not total:
            continue
        rows.append(
            {
                "timestamp": entry.get("timestamp"),
                "input_tokens": int(last.get("input_tokens", 0)),
                "cached_input_tokens": int(last.get("cached_input_tokens", 0)),
                "output_tokens": int(last.get("output_tokens", 0)),
                "reasoning_output_tokens": int(last.get("reasoning_output_tokens", 0)),
                "total_tokens": int(last.get("total_tokens", 0)),
                "cumulative_total_tokens": int(total.get("total_tokens", 0)),
                "model_context_window": int(info.get("model_context_window", 0)),
            }
        )
    return rows


def resolve_rollout_path(
    sessions_dir: Path, *, cwd: str, thread_id: str | None = None
) -> Path:
    return (
        find_rollout_path_by_thread_id(sessions_dir, thread_id)
        if thread_id
        else find_latest_rollout_path(sessions_dir, cwd)
    )


def load_latest_usage_snapshot(
    *, cwd: str, codex_home: Path | None = None, thread_id: str | None = None
) -> dict[str, Any] | None:
    home = (codex_home or CODEX_HOME).expanduser()
    sessions_dir = home / "sessions"
    try:
        rollout_path = resolve_rollout_path(sessions_dir, cwd=cwd, thread_id=thread_id)
    except SystemExit:
        return None
    rows = load_usage_rows(rollout_path)
    if not rows:
        return None
    row = rows[-1]
    return {
        "cwd": cwd,
        "thread_id": thread_id or thread_id_from_rollout_path(rollout_path),
        "rollout_path": str(rollout_path),
        "timestamp": row["timestamp"],
        "last_turn_total_tokens": int(row["total_tokens"]),
        "cumulative_total_tokens": int(row["cumulative_total_tokens"]),
        "input_tokens": int(row["input_tokens"]),
        "cached_input_tokens": int(row["cached_input_tokens"]),
        "output_tokens": int(row["output_tokens"]),
        "reasoning_output_tokens": int(row["reasoning_output_tokens"]),
        "model_context_window": int(row["model_context_window"]),
    }


def select_rows(
    rows: list[dict[str, Any]], limit: int, last_turn: bool
) -> list[dict[str, Any]]:
    if last_turn:
        return rows[-1:] if rows else []
    if limit > 0:
        return rows[-limit:]
    return rows


def format_int(value: int) -> str:
    return f"{value:,}"


def format_compact_k(value: int) -> str:
    return f"{value / 1000:.1f}K"


def format_value(value: int, full_numbers: bool) -> str:
    if full_numbers:
        return format_int(value)
    return format_compact_k(value)


def render_table(rows: list[dict[str, Any]], rollout_path: Path, full_numbers: bool) -> str:
    headers = [
        "timestamp",
        "input",
        "cached",
        "output",
        "reasoning",
        "turn_total",
        "cum_total",
    ]
    body: list[list[str]] = []
    for row in rows:
        body.append(
            [
                str(row["timestamp"]),
                format_value(row["input_tokens"], full_numbers),
                format_value(row["cached_input_tokens"], full_numbers),
                format_value(row["output_tokens"], full_numbers),
                format_value(row["reasoning_output_tokens"], full_numbers),
                format_value(row["total_tokens"], full_numbers),
                format_value(row["cumulative_total_tokens"], full_numbers),
            ]
        )

    widths = [len(header) for header in headers]
    for line in body:
        for idx, cell in enumerate(line):
            widths[idx] = max(widths[idx], len(cell))

    lines = [f"rollout_path: {rollout_path}", ""]
    lines.append("  ".join(header.ljust(widths[idx]) for idx, header in enumerate(headers)))
    lines.append("  ".join("-" * widths[idx] for idx in range(len(headers))))
    for line in body:
        lines.append("  ".join(cell.ljust(widths[idx]) for idx, cell in enumerate(line)))
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    codex_home = Path(args.codex_home).expanduser()
    sessions_dir = codex_home / "sessions"
    rollout_path = resolve_rollout_path(
        sessions_dir, cwd=args.cwd, thread_id=args.thread_id
    )
    rows = load_usage_rows(rollout_path)
    rows = select_rows(rows, args.limit, args.last_turn)

    result = {
        "cwd": args.cwd,
        "thread_id": (
            args.thread_id if args.thread_id else thread_id_from_rollout_path(rollout_path)
        ),
        "rollout_path": str(rollout_path),
        "last_turn": args.last_turn,
        "row_count": len(rows),
        "rows": rows,
    }

    if args.last_turn_total_only:
        if not rows:
            raise SystemExit("No token_count rows found for the selected thread.")
        print(int(rows[-1]["total_tokens"]))
        return 0

    if args.json:
        print(json.dumps(result, ensure_ascii=True, indent=2))
        return 0

    print(render_table(rows, rollout_path, args.full_numbers))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
