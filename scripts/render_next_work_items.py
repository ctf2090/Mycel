#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


DEFAULT_COMPACTION_MESSAGE = "compaction detected, we better open a new chat."
DEFAULT_COMPACTION_TRADEOFF = (
    "safest follow-up after compaction, but it pauses immediate work until a fresh chat is open."
)


class NextWorkItemsError(Exception):
    """Raised when the next-work-items spec is invalid."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/render_next_work_items.py",
        description="Render Markdown next-work-item options from a JSON spec.",
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
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise NextWorkItemsError(f"invalid JSON spec: {exc.msg}") from exc
    if not isinstance(payload, dict):
        raise NextWorkItemsError("JSON spec must be an object")
    return payload


def require_string(entry: dict[str, object], key: str, *, item_index: int) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value.strip():
        raise NextWorkItemsError(f"item {item_index} must provide a non-empty string '{key}'")
    return value.strip()


def parse_bool(payload: dict[str, object], key: str) -> bool:
    value = payload.get(key, False)
    if not isinstance(value, bool):
        raise NextWorkItemsError(f"'{key}' must be a boolean when provided")
    return value


def parse_optional_string(payload: dict[str, object], key: str) -> str | None:
    value = payload.get(key)
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise NextWorkItemsError(f"'{key}' must be a non-empty string when provided")
    return value.strip()


def parse_items(payload: dict[str, object]) -> list[dict[str, str]]:
    raw_items = payload.get("items", [])
    if not isinstance(raw_items, list):
        raise NextWorkItemsError("'items' must be an array when provided")

    items: list[dict[str, str]] = []
    for index, entry in enumerate(raw_items, start=1):
        if not isinstance(entry, dict):
            raise NextWorkItemsError(f"item {index} must be an object")
        item = {
            "text": require_string(entry, "text", item_index=index),
            "tradeoff": require_string(entry, "tradeoff", item_index=index),
        }
        roadmap = entry.get("roadmap")
        if roadmap is not None:
            if not isinstance(roadmap, str) or not roadmap.strip():
                raise NextWorkItemsError(f"item {index} has an invalid 'roadmap' value")
            item["roadmap"] = roadmap.strip()
        items.append(item)
    return items


def build_items(payload: dict[str, object]) -> list[dict[str, str]]:
    items = parse_items(payload)
    compaction_detected = parse_bool(payload, "compaction_detected")
    if compaction_detected:
        compaction_item = {
            "text": parse_optional_string(payload, "compaction_message") or DEFAULT_COMPACTION_MESSAGE,
            "tradeoff": parse_optional_string(payload, "compaction_tradeoff")
            or DEFAULT_COMPACTION_TRADEOFF,
        }
        items.insert(0, compaction_item)
    if not items:
        raise NextWorkItemsError("spec must provide at least one item or set compaction_detected=true")
    return items


def render_items(items: list[dict[str, str]]) -> str:
    lines: list[str] = []
    for index, item in enumerate(items, start=1):
        prefix = f"{index}. "
        if index == 1:
            prefix += "(最有價值) "
        line = f"{prefix}{item['text']} Tradeoff: {item['tradeoff']}"
        roadmap = item.get("roadmap")
        if roadmap:
            line += f" Roadmap: {roadmap}"
        lines.append(line)
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    try:
        payload = load_spec(args.spec_path)
        items = build_items(payload)
    except NextWorkItemsError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(render_items(items), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
