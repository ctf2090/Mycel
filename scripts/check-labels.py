#!/usr/bin/env python3
"""Check whether GitHub host labels match .github/labels.yml."""

from __future__ import annotations

import argparse
from pathlib import Path

from github_labels_lib import LabelToolError, gh_json, load_tracked_labels, require_cmd


ROOT_DIR = Path(__file__).resolve().parent.parent
LABELS_FILE = ROOT_DIR / ".github" / "labels.yml"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check whether GitHub host labels match .github/labels.yml."
    )
    parser.add_argument("--repo", "-R", default="MycelLayer/Mycel")
    parser.add_argument("--strict", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        require_cmd("gh")
        expected_labels = load_tracked_labels(LABELS_FILE)
        actual_payload = gh_json(args.repo, "label", "list", "--limit", "200", "--json", "name,color,description")
    except LabelToolError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    actual_map: dict[str, tuple[str, str]] = {}
    for entry in actual_payload:
        name = str(entry.get("name", "")).strip()
        if not name:
            continue
        actual_map[name] = (
            str(entry.get("color", "")).strip().lower(),
            str(entry.get("description", "")).strip(),
        )

    missing = False
    mismatch = False
    for label in expected_labels:
        expected_name = label["name"]
        expected_color = label["color"]
        expected_description = label["description"].strip()
        actual = actual_map.get(expected_name)
        if actual is None:
            print(f"missing label on GitHub: {expected_name}")
            missing = True
            continue
        actual_color, actual_description = actual
        if actual_color != expected_color or actual_description != expected_description:
            print(f"mismatched label: {expected_name}")
            print(f"  expected: {expected_color}\t{expected_description}")
            print(f"  actual:   {actual_color}\t{actual_description}")
            mismatch = True

    extra = False
    if args.strict:
        expected_names = {label["name"] for label in expected_labels}
        for actual_name in sorted(actual_map):
            if actual_name not in expected_names:
                print(f"extra label on GitHub (not tracked): {actual_name}")
                extra = True

    if missing or mismatch or extra:
        return 1

    count = len(expected_labels)
    if args.strict:
        print(f"labels are in strict sync for {count} tracked labels")
    else:
        print(f"tracked labels are in sync for {count} labels")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(main())
