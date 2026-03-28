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
ROLE_DEFAULT_ITEMS: dict[str, list[dict[str, str]]] = {
    "coding": [
        {
            "text": "review ROADMAP.md and identify the highest-value next coding work",
            "tradeoff": "best roadmap alignment, but it spends a little time on prioritization before implementation",
            "roadmap": "ROADMAP.md / next coding slice",
        },
        {
            "text": "review the latest CQH issue and identify high-value work items",
            "tradeoff": "usually cheaper to land quickly, but it may be less directly tied to the main roadmap lane",
        },
    ],
    "delivery": [
        {
            "text": "review the latest completed CI result before choosing the next delivery follow-up",
            "tradeoff": "safest delivery baseline, but it may delay action if CI context needs re-reading",
        },
        {
            "text": "review the current delivery workflow or process follow-up with the freshest CI evidence",
            "tradeoff": "good for stabilizing delivery flow, but it is less directly product-facing than coding work",
        },
    ],
    "doc": [
        {
            "text": "review the freshest planning or documentation follow-up before choosing the next doc item",
            "tradeoff": "keeps doc work aligned with current repo state, but it adds a short review step first",
        },
        {
            "text": "check whether planning-sync or issue-sync follow-up is due before writing the next doc update",
            "tradeoff": "helps avoid drift in planning surfaces, but it may defer narrower writing work briefly",
        },
    ],
}


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


def parse_optional_role(payload: dict[str, object]) -> str | None:
    role = parse_optional_string(payload, "role")
    if role is None:
        return None
    if role not in ROLE_DEFAULT_ITEMS:
        raise NextWorkItemsError(
            f"'role' must be one of: {', '.join(sorted(ROLE_DEFAULT_ITEMS))}"
        )
    return role


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


def role_default_items(role: str | None) -> list[dict[str, str]]:
    if role is None:
        return []
    return [dict(entry) for entry in ROLE_DEFAULT_ITEMS[role]]


def build_items(payload: dict[str, object]) -> list[dict[str, str]]:
    role = parse_optional_role(payload)
    items = role_default_items(role)
    items.extend(parse_items(payload))
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


def render_payload(payload: dict[str, object]) -> str:
    return render_items(build_items(payload))


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
        rendered = render_payload(payload)
    except NextWorkItemsError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
