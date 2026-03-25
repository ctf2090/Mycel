#!/usr/bin/env python3
"""Helpers for repo-tracked GitHub labels."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path


class LabelToolError(Exception):
    pass


def require_cmd(cmd: str) -> None:
    if not shutil.which(cmd):
        raise LabelToolError(f"{cmd} is required")


def parse_scalar(raw: str) -> str:
    value = raw.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def load_tracked_labels(labels_path: Path) -> list[dict[str, str]]:
    if not labels_path.is_file():
        raise LabelToolError(f"labels file not found: {labels_path}")

    labels: list[dict[str, str]] = []
    current: dict[str, str] | None = None
    in_labels = False

    for raw_line in labels_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line == "labels:":
            in_labels = True
            continue
        if not in_labels:
            continue
        if line.startswith("- "):
            if current is not None:
                labels.append(current)
            current = {}
            line = line[2:].strip()
        if current is None:
            raise LabelToolError("invalid labels file: expected a list item under labels:")
        if ":" not in line:
            raise LabelToolError(f"invalid labels file line: {raw_line}")
        key, value = line.split(":", 1)
        current[key.strip()] = parse_scalar(value)

    if current is not None:
        labels.append(current)

    normalized: list[dict[str, str]] = []
    for entry in labels:
        try:
            normalized.append(
                {
                    "name": entry["name"],
                    "color": entry["color"].lower(),
                    "description": entry.get("description", ""),
                }
            )
        except KeyError as exc:
            raise LabelToolError(f"invalid labels file entry missing {exc.args[0]!r}") from exc
    return normalized


def gh_json(repo: str | None, *args: str) -> list[dict[str, object]]:
    cmd = ["gh", *args]
    if repo:
        cmd.extend(["--repo", repo])
    proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "gh command failed"
        raise LabelToolError(detail)
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise LabelToolError(f"failed to parse gh output: {exc}") from exc
    if not isinstance(payload, list):
        raise LabelToolError("expected gh JSON output to be a list")
    return payload


def run_gh(repo: str | None, *args: str) -> None:
    cmd = ["gh", *args]
    if repo:
        cmd.extend(["--repo", repo])
    proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "gh command failed"
        raise LabelToolError(detail)
