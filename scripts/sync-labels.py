#!/usr/bin/env python3
"""Sync repo-tracked GitHub labels from .github/labels.yml."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from github_labels_lib import LabelToolError, load_tracked_labels, require_cmd, run_gh


ROOT_DIR = Path(__file__).resolve().parent.parent
LABELS_FILE = ROOT_DIR / ".github" / "labels.yml"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Sync repo-tracked GitHub labels from .github/labels.yml."
    )
    parser.add_argument("--repo", "-R", default="MycelLayer/Mycel")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        require_cmd("gh")
        labels = load_tracked_labels(LABELS_FILE)
        for label in labels:
            run_gh(
                args.repo,
                "label",
                "create",
                label["name"],
                "--color",
                label["color"],
                "--description",
                label["description"],
                "--force",
            )
            print(f"synced label: {label['name']}")
    except LabelToolError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(f"synced {len(labels)} labels from .github/labels.yml")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
