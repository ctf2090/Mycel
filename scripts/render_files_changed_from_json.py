#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import render_files_changed_table as table


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/render_files_changed_from_json.py",
        description=(
            "Render a Markdown 'Files changed' table from a JSON spec without "
            "shell-quoting note text."
        ),
    )
    parser.add_argument(
        "--diff-key",
        required=True,
        help="stable diff bucket key used for clickable delta links",
    )
    parser.add_argument(
        "spec_path",
        nargs="?",
        default="-",
        help="JSON spec path, or '-' to read the spec from stdin",
    )
    return parser.parse_args()


def load_spec(spec_path: str) -> dict[str, object]:
    if spec_path == "-":
        raw = sys.stdin.read()
    else:
        raw = Path(spec_path).read_text(encoding="utf-8")
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise table.FilesChangedError(f"invalid JSON spec: {exc.msg}") from exc
    if not isinstance(data, dict):
        raise table.FilesChangedError("JSON spec must be an object")
    return data


def parse_rows(data: dict[str, object]) -> list[tuple[str, str, str]]:
    raw_rows = data.get("rows")
    if not isinstance(raw_rows, list) or not raw_rows:
        raise table.FilesChangedError("JSON spec must contain a non-empty 'rows' array")

    rows: list[tuple[str, str, str]] = []
    for index, entry in enumerate(raw_rows, start=1):
        if not isinstance(entry, dict):
            raise table.FilesChangedError(f"row {index} must be an object")
        path = entry.get("path")
        added = entry.get("added")
        removed = entry.get("removed")
        note = entry.get("note")
        if not all(isinstance(value, str) and value.strip() for value in (path, added, removed, note)):
            raise table.FilesChangedError(
                f"row {index} must provide non-empty string values for path, added, removed, and note"
            )
        rows.append((path.strip(), added.strip(), removed.strip()))
    return rows


def parse_notes(data: dict[str, object]) -> dict[str, str]:
    raw_rows = data["rows"]
    notes: dict[str, str] = {}
    for entry in raw_rows:
        assert isinstance(entry, dict)
        path = str(entry["path"]).strip()
        note = str(entry["note"]).strip()
        notes[path] = note
    return notes


def main() -> int:
    args = parse_args()
    try:
        data = load_spec(args.spec_path)
        rows = parse_rows(data)
        notes = parse_notes(data)
        table.require_notes_for_all_rows(rows, notes)
    except table.FilesChangedError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print(table.render_table(rows, notes, stdin_diff_key=args.diff_key))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
