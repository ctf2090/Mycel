#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parent.parent


class FilesChangedError(Exception):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/render_files_changed_table.py",
        description="Render a Markdown 'Files changed' table from git numstat output.",
    )
    parser.add_argument(
        "git_ref",
        nargs="?",
        default="HEAD",
        help="git revision, commit, or range to inspect (default: HEAD)",
    )
    parser.add_argument(
        "--note",
        action="append",
        default=[],
        metavar="PATH=TEXT",
        help="one-line note override for a specific file path",
    )
    parser.add_argument(
        "--stdin",
        action="store_true",
        help="read numstat content from stdin instead of invoking git",
    )
    return parser.parse_args()


def load_numstat(git_ref: str, from_stdin: bool) -> str:
    if from_stdin:
        return sys.stdin.read()

    proc = subprocess.run(
        ["git", "show", "--numstat", "--format=", git_ref],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise FilesChangedError(proc.stderr.strip() or f"git show failed for {git_ref}")
    return proc.stdout


def diff_output_dir(git_ref: str) -> Path:
    digest = hashlib.sha1(git_ref.encode("utf-8")).hexdigest()[:12]
    return ROOT_DIR / ".agent-local" / "rendered-diffs" / digest


def clear_other_diff_output_dirs(git_ref: str) -> None:
    rendered_diffs_root = ROOT_DIR / ".agent-local" / "rendered-diffs"
    current_output_dir = diff_output_dir(git_ref)
    if not rendered_diffs_root.exists():
        return
    for path in rendered_diffs_root.iterdir():
        if path == current_output_dir or not path.is_dir():
            continue
        for nested in sorted(path.rglob("*"), reverse=True):
            if nested.is_file() or nested.is_symlink():
                nested.unlink()
            elif nested.is_dir():
                nested.rmdir()
        path.rmdir()


def clear_diff_output_dir(git_ref: str) -> Path:
    output_dir = diff_output_dir(git_ref)
    if output_dir.exists():
        for diff_path in output_dir.rglob("*.diff"):
            diff_path.unlink()
    return output_dir


def parse_note_overrides(raw_notes: list[str]) -> dict[str, str]:
    notes: dict[str, str] = {}
    for raw in raw_notes:
        if "=" not in raw:
            raise FilesChangedError(f"invalid --note value: {raw!r}; expected PATH=TEXT")
        path, note = raw.split("=", 1)
        path = path.strip()
        note = note.strip()
        if not path or not note:
            raise FilesChangedError(f"invalid --note value: {raw!r}; expected PATH=TEXT")
        notes[path] = note
    return notes


def parse_numstat(text: str) -> list[tuple[str, str, str]]:
    rows: list[tuple[str, str, str]] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        parts = line.split("\t", 2)
        if len(parts) != 3:
            continue
        added, removed, path = parts
        rows.append((path, added, removed))
    return rows


def render_count(value: str, prefix: str) -> str:
    if value == "-":
        return f"{prefix}n/a"
    return f"{prefix}{value}"


def render_file_cell(path: str) -> str:
    resolved = (ROOT_DIR / path).resolve()
    if resolved.exists():
        return f"[{path}]({resolved})"
    return path


def write_diff_file(git_ref: str, path: str) -> Path:
    output_dir = diff_output_dir(git_ref)
    diff_path = output_dir / f"{path}.diff"
    diff_path.parent.mkdir(parents=True, exist_ok=True)

    proc = subprocess.run(
        ["git", "diff", "--no-ext-diff", "--binary", f"{git_ref}^!", "--", path],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise FilesChangedError(proc.stderr.strip() or f"git diff failed for {git_ref} -- {path}")

    diff_path.write_text(proc.stdout, encoding="utf-8")
    return diff_path


def semantic_note_from_path(path: str) -> str | None:
    path_obj = Path(path)
    name = path_obj.name
    parts = path_obj.parts

    if name.startswith("ROADMAP."):
        return "Refresh roadmap status and milestone wording."
    if name.startswith("IMPLEMENTATION-CHECKLIST."):
        return "Update checklist closure state and follow-up tracking."
    if path == "docs/PROGRESS.md" or name == "progress.html":
        return "Sync public progress summary with current planning state."
    if name == "index.html" and "pages" in parts:
        return "Refresh landing-page contributor entry and planning summary copy."
    if path == "AGENTS.md" or name == "AGENTS.md":
        return "Clarify agent workflow instructions."
    if "scripts" in parts and path_obj.suffix in {".py", ".sh"}:
        return "Adjust repo tooling behavior and command output."
    if "tests" in parts or name.startswith("test_") or name.endswith("_test.rs"):
        return "Expand regression coverage for this area."
    if path_obj.suffix == ".rs":
        return f"Update {path_obj.stem} implementation behavior."
    if path_obj.suffix == ".md":
        return f"Refresh {name} documentation wording."
    if path_obj.suffix == ".html":
        return f"Update {name} page content."
    if path_obj.suffix == ".css":
        return f"Adjust {name} styling."
    return None


def default_note(path: str, added: str, removed: str) -> str:
    semantic_note = semantic_note_from_path(path)
    if semantic_note is not None:
        return semantic_note
    if added == "-" or removed == "-":
        return "Binary or non-line diff in this commit."
    added_n = int(added)
    removed_n = int(removed)
    if added_n > 0 and removed_n == 0:
        return "Added content in this commit."
    if removed_n > 0 and added_n == 0:
        return "Removed content in this commit."
    return "Updated content in this commit."


def render_table(rows: list[tuple[str, str, str]], note_overrides: dict[str, str], *, git_ref: str | None = None) -> str:
    lines = [
        "| File | +/- | One-line note |",
        "|---|---:|---|",
    ]
    if git_ref is not None:
        clear_other_diff_output_dirs(git_ref)
        clear_diff_output_dir(git_ref)
    for path, added, removed in rows:
        delta = f"{render_count(added, '+')} / {render_count(removed, '-')}"
        if git_ref is not None:
            diff_path = write_diff_file(git_ref, path)
            delta = f"[{delta}]({diff_path})"
        note = note_overrides.get(path, default_note(path, added, removed))
        lines.append(f"| {render_file_cell(path)} | {delta} | {note} |")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    try:
        note_overrides = parse_note_overrides(args.note)
        rows = parse_numstat(load_numstat(args.git_ref, args.stdin))
    except FilesChangedError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if not rows:
        print("No file changes found.", file=sys.stderr)
        return 1

    print(render_table(rows, note_overrides, git_ref=None if args.stdin else args.git_ref))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
