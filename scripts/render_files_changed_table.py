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
    parser.add_argument(
        "--diff-key",
        help=(
            "stable diff bucket key for rendering clickable delta links in --stdin "
            "mode"
        ),
    )
    return parser.parse_args()


def is_range_ref(git_ref: str) -> bool:
    return ".." in git_ref


def load_numstat(git_ref: str, from_stdin: bool) -> str:
    if from_stdin:
        return sys.stdin.read()

    command = (
        ["git", "diff", "--numstat", git_ref]
        if is_range_ref(git_ref)
        else ["git", "show", "--numstat", "--format=", git_ref]
    )
    proc = subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        action = "git diff" if is_range_ref(git_ref) else "git show"
        raise FilesChangedError(proc.stderr.strip() or f"{action} failed for {git_ref}")
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


def require_notes_for_all_rows(
    rows: list[tuple[str, str, str]], note_overrides: dict[str, str]
) -> None:
    missing = [path for path, _, _ in rows if path not in note_overrides]
    if not missing:
        return
    missing_display = ", ".join(missing)
    raise FilesChangedError(
        "missing required --note entries for: "
        f"{missing_display}; provide --note PATH=TEXT for every changed file"
    )


def render_count(value: str, prefix: str) -> str:
    if value == "-":
        return f"{prefix}n/a"
    return f"{prefix}{value}"


def render_delta(added: str, removed: str, *, stdin_mode: bool) -> str:
    if stdin_mode and added == "0" and removed == "0":
        return "tracked artifact"
    return f"{render_count(added, '+')} / {render_count(removed, '-')}"


def render_file_cell(path: str) -> str:
    resolved = (ROOT_DIR / path).resolve()
    if resolved.exists():
        return f"[{path}]({resolved})"
    return path


def write_diff_file(git_ref: str, path: str) -> Path:
    output_dir = diff_output_dir(git_ref)
    diff_path = output_dir / f"{path}.diff"
    diff_path.parent.mkdir(parents=True, exist_ok=True)

    command = (
        ["git", "diff", "--no-ext-diff", "--binary", git_ref, "--", path]
        if is_range_ref(git_ref)
        else ["git", "diff", "--no-ext-diff", "--binary", f"{git_ref}^!", "--", path]
    )
    proc = subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        raise FilesChangedError(proc.stderr.strip() or f"git diff failed for {git_ref} -- {path}")

    diff_path.write_text(proc.stdout, encoding="utf-8")
    return diff_path


def write_stdin_diff_file(diff_key: str, path: str) -> Path:
    output_dir = diff_output_dir(diff_key)
    diff_path = output_dir / f"{path}.diff"
    diff_path.parent.mkdir(parents=True, exist_ok=True)

    target = ROOT_DIR / path
    if not target.exists():
        raise FilesChangedError(f"stdin diff path does not exist: {path}")

    proc = subprocess.run(
        ["git", "diff", "--no-ext-diff", "--binary", "--no-index", "--", "/dev/null", path],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode not in (0, 1):
        raise FilesChangedError(
            proc.stderr.strip() or f"git diff --no-index failed for {path}"
        )

    diff_path.write_text(proc.stdout, encoding="utf-8")
    return diff_path


def render_table(
    rows: list[tuple[str, str, str]],
    note_overrides: dict[str, str],
    *,
    git_ref: str | None = None,
    stdin_diff_key: str | None = None,
) -> str:
    lines = [
        "| File | +/- | One-line note |",
        "|---|---:|---|",
    ]
    diff_bucket_key = git_ref if git_ref is not None else stdin_diff_key
    if diff_bucket_key is not None:
        clear_other_diff_output_dirs(diff_bucket_key)
        clear_diff_output_dir(diff_bucket_key)
    for path, added, removed in rows:
        delta = render_delta(added, removed, stdin_mode=stdin_diff_key is not None)
        if git_ref is not None:
            diff_path = write_diff_file(git_ref, path)
            delta = f"[{delta}]({diff_path})"
        elif stdin_diff_key is not None:
            diff_path = write_stdin_diff_file(stdin_diff_key, path)
            delta = f"[{delta}]({diff_path})"
        note = note_overrides[path]
        lines.append(f"| {render_file_cell(path)} | {delta} | {note} |")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    try:
        note_overrides = parse_note_overrides(args.note)
        rows = parse_numstat(load_numstat(args.git_ref, args.stdin))
        require_notes_for_all_rows(rows, note_overrides)
        output = render_table(
            rows,
            note_overrides,
            git_ref=None if args.stdin else args.git_ref,
            stdin_diff_key=args.diff_key if args.stdin else None,
        )
    except FilesChangedError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if not rows:
        print("No file changes found.", file=sys.stderr)
        return 1

    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
