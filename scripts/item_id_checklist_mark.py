#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
CHECKLIST_DIR = ROOT_DIR / ".agent-local" / "checklists"
ITEM_LINE_RE = re.compile(
    r"^(?P<prefix>\s*-\s\[(?P<mark>[X! ])\]\s.*?)(?P<suffix>\s*<!-- item-id: (?P<item_id>.*?) -->\s*)$"
)
PROBLEM_SUBITEM_RE = re.compile(r"^\s{2,}-\sProblem:\s.*$")


class ItemIdChecklistMarkError(Exception):
    pass


def relative_to_root(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT_DIR))
    except ValueError:
        return str(path)


def resolve_checklist_path(path_value: str) -> Path:
    candidate = Path(path_value)
    if not candidate.is_absolute():
        candidate = ROOT_DIR / candidate
    resolved = candidate.resolve()
    try:
        resolved.relative_to(CHECKLIST_DIR.resolve())
    except ValueError as exc:
        raise ItemIdChecklistMarkError("checklist path must live under .agent-local/checklists/") from exc
    if not resolved.exists():
        raise ItemIdChecklistMarkError(f"checklist file not found: {relative_to_root(resolved)}")
    if not resolved.is_file():
        raise ItemIdChecklistMarkError(f"checklist path is not a file: {relative_to_root(resolved)}")
    return resolved


def find_item(lines: list[str], item_id: str) -> tuple[int, str]:
    for index, line in enumerate(lines):
        match = ITEM_LINE_RE.match(line)
        if match is None:
            continue
        if match.group("item_id").strip() == item_id:
            return index, match.group("mark")
    raise ItemIdChecklistMarkError(f"checklist item not found: {item_id}")


def mark_to_state(mark: str) -> str:
    if mark == "X":
        return "checked"
    if mark == "!":
        return "problem"
    return "unchecked"


def remove_problem_subitem(lines: list[str], item_index: int) -> None:
    next_index = item_index + 1
    if next_index < len(lines) and PROBLEM_SUBITEM_RE.match(lines[next_index]):
        del lines[next_index]


def set_problem_subitem(lines: list[str], item_index: int, problem: str) -> None:
    subitem = f"  - Problem: {problem}"
    next_index = item_index + 1
    if next_index < len(lines) and PROBLEM_SUBITEM_RE.match(lines[next_index]):
        lines[next_index] = subitem
        return
    lines.insert(next_index, subitem)


def apply_state(lines: list[str], item_index: int, current_mark: str, state: str, problem: str) -> str:
    if state == "toggle":
        next_mark = " " if current_mark in {"X", "!"} else "X"
    elif state == "checked":
        next_mark = "X"
    elif state == "problem":
        next_mark = "!"
    else:
        next_mark = " "

    if next_mark != current_mark:
        lines[item_index] = lines[item_index].replace(f"[{current_mark}]", f"[{next_mark}]", 1)

    if next_mark == "!":
        if not problem.strip():
            raise ItemIdChecklistMarkError("problem state requires --problem with a short explanation")
        set_problem_subitem(lines, item_index, problem.strip())
    else:
        remove_problem_subitem(lines, item_index)

    return mark_to_state(next_mark)


def print_human(data: dict[str, Any]) -> None:
    print(f"path: {data['path']}")
    print(f"item_id: {data['item_id']}")
    print(f"state: {data['state']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/item_id_checklist_mark.py",
        description="Update one item-id checklist line in an agent-local checklist copy.",
    )
    parser.add_argument("checklist_md")
    parser.add_argument("item_id")
    parser.add_argument("--state", choices=["checked", "unchecked", "problem", "toggle"], default="checked")
    parser.add_argument("--problem", default="")
    parser.add_argument("--json", action="store_true")
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        checklist_path = resolve_checklist_path(args.checklist_md)
        lines = checklist_path.read_text(encoding="utf-8").splitlines()
        item_index, current_mark = find_item(lines, args.item_id)
        state = apply_state(lines, item_index, current_mark, args.state, args.problem)
        checklist_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        result = {
            "status": "ok",
            "path": relative_to_root(checklist_path),
            "item_id": args.item_id,
            "state": state,
        }
    except ItemIdChecklistMarkError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(result))
    else:
        print_human(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
