#!/usr/bin/env python3
"""Inspect and optionally reclaim common Codespaces storage hot spots."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class TargetSpec:
    key: str
    label: str
    root: str
    relative_path: str


TARGET_SPECS = (
    TargetSpec("repo-target", "Cargo build output", "workspace", "target"),
    TargetSpec("repo-tmp", "Workspace tmp scratch data", "workspace", "tmp"),
    TargetSpec("repo-pytest-cache", "Pytest cache", "workspace", ".pytest_cache"),
    TargetSpec("repo-node-cache", "node_modules cache", "workspace", "node_modules/.cache"),
    TargetSpec("cargo-registry-cache", "Cargo registry crate archives", "home", ".cargo/registry/cache"),
    TargetSpec("cargo-git-db", "Cargo git checkout cache", "home", ".cargo/git/db"),
    TargetSpec("npm-cache", "npm package cache", "home", ".npm/_cacache"),
    TargetSpec("pip-cache", "pip cache", "home", ".cache/pip"),
)
TARGET_KEYS = tuple(spec.key for spec in TARGET_SPECS)
HOME_TARGET_KEYS = {spec.key for spec in TARGET_SPECS if spec.root == "home"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inspect or reclaim common rebuildable storage hot spots in a Codespaces workspace."
    )
    parser.add_argument(
        "--workspace",
        default=".",
        help="workspace root to inspect; defaults to the current directory",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="delete the selected targets instead of reporting a dry-run plan",
    )
    parser.add_argument(
        "--include-home-caches",
        action="store_true",
        help="include home-directory caches such as ~/.cargo and ~/.npm",
    )
    parser.add_argument(
        "--target",
        action="append",
        default=[],
        choices=TARGET_KEYS,
        help="limit the run to one target key; may be passed multiple times",
    )
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    return parser.parse_args()


def format_bytes(size: int) -> str:
    units = ("B", "KiB", "MiB", "GiB", "TiB")
    value = float(size)
    for unit in units:
        if value < 1024.0 or unit == units[-1]:
            if unit == "B":
                return f"{int(value)} {unit}"
            return f"{value:.1f} {unit}"
        value /= 1024.0
    return f"{size} B"


def resolve_workspace(path_value: str) -> Path:
    return Path(path_value).expanduser().resolve()


def iter_selected_specs(args: argparse.Namespace) -> list[TargetSpec]:
    selected_keys = list(dict.fromkeys(args.target))
    if not selected_keys:
        selected_keys = [spec.key for spec in TARGET_SPECS if spec.root == "workspace"]
        if args.include_home_caches:
            selected_keys.extend(spec.key for spec in TARGET_SPECS if spec.root == "home")
    elif args.include_home_caches:
        # Keep explicit selections stable; the flag only widens the default set.
        pass

    return [spec for spec in TARGET_SPECS if spec.key in selected_keys]


def path_size_bytes(path: Path) -> int:
    if path.is_symlink():
        return 0
    if path.is_file():
        return path.stat().st_size
    total = 0
    stack = [path]
    while stack:
        current = stack.pop()
        try:
            with os.scandir(current) as entries:
                for entry in entries:
                    try:
                        if entry.is_symlink():
                            continue
                        if entry.is_dir(follow_symlinks=False):
                            stack.append(Path(entry.path))
                        else:
                            total += entry.stat(follow_symlinks=False).st_size
                    except FileNotFoundError:
                        continue
        except FileNotFoundError:
            continue
    return total


def resolve_target_path(spec: TargetSpec, workspace: Path, home_dir: Path) -> Path:
    base = workspace if spec.root == "workspace" else home_dir
    return (base / spec.relative_path).resolve()


def target_status(path: Path, allowed_root: Path) -> str:
    if not path.exists():
        return "missing"
    try:
        path.relative_to(allowed_root)
    except ValueError:
        return "outside-root"
    if path.is_symlink():
        return "symlink-skipped"
    if not path.is_dir():
        return "not-directory"
    return "present"


def collect_targets(selected_specs: list[TargetSpec], workspace: Path, home_dir: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for spec in selected_specs:
        allowed_root = workspace if spec.root == "workspace" else home_dir
        path = resolve_target_path(spec, workspace, home_dir)
        status = target_status(path, allowed_root)
        size_bytes = path_size_bytes(path) if status == "present" else 0
        rows.append(
            {
                "key": spec.key,
                "label": spec.label,
                "scope": spec.root,
                "path": str(path),
                "status": status,
                "size_bytes": size_bytes,
            }
        )
    return rows


def apply_gc(rows: list[dict[str, object]]) -> None:
    for row in rows:
        if row["status"] != "present":
            row["action"] = "skipped"
            row["reclaimed_bytes"] = 0
            continue
        path = Path(str(row["path"]))
        reclaimed_bytes = int(row["size_bytes"])
        shutil.rmtree(path)
        row["action"] = "removed"
        row["reclaimed_bytes"] = reclaimed_bytes
        row["status"] = "removed"


def disk_snapshot(path: Path) -> dict[str, int]:
    usage = shutil.disk_usage(path)
    return {"total_bytes": usage.total, "used_bytes": usage.used, "free_bytes": usage.free}


def render_text(
    *,
    workspace: Path,
    mode: str,
    rows: list[dict[str, object]],
    before_disk: dict[str, int],
    after_disk: dict[str, int] | None,
) -> str:
    lines = [
        f"mode: {mode}",
        f"workspace: {workspace}",
        f"disk_free_before: {format_bytes(before_disk['free_bytes'])}",
        f"targets_selected: {len(rows)}",
    ]
    for row in rows:
        action = str(row.get("action", "planned" if row["status"] == "present" else "skipped"))
        size_label = format_bytes(int(row["size_bytes"]))
        lines.append(
            "\t".join(
                [
                    str(row["key"]),
                    str(row["status"]),
                    action,
                    size_label,
                    str(row["path"]),
                ]
            )
        )

    reclaimable = sum(int(row["size_bytes"]) for row in rows if row["status"] in {"present", "removed"})
    lines.append(f"estimated_reclaimable: {format_bytes(reclaimable)}")
    if after_disk is not None:
        lines.append(f"disk_free_after: {format_bytes(after_disk['free_bytes'])}")
        lines.append(
            f"reported_free_delta: {format_bytes(after_disk['free_bytes'] - before_disk['free_bytes'])}"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    workspace = resolve_workspace(args.workspace)
    if not workspace.exists():
        print(f"workspace does not exist: {workspace}", file=sys.stderr)
        return 2

    home_dir = Path.home().resolve()
    selected_specs = iter_selected_specs(args)
    before_disk = disk_snapshot(workspace)
    rows = collect_targets(selected_specs, workspace, home_dir)

    after_disk: dict[str, int] | None = None
    mode = "dry-run"
    if args.apply:
        apply_gc(rows)
        after_disk = disk_snapshot(workspace)
        mode = "apply"

    payload = {
        "mode": mode,
        "workspace": str(workspace),
        "before_disk": before_disk,
        "after_disk": after_disk,
        "targets": rows,
        "estimated_reclaimable_bytes": sum(
            int(row["size_bytes"]) for row in rows if row["status"] in {"present", "removed"}
        ),
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(render_text(workspace=workspace, mode=mode, rows=rows, before_disk=before_disk, after_disk=after_disk), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
