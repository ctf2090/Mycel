#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from codex_token_usage_summary import load_latest_usage_snapshot


ROOT_DIR = Path(__file__).resolve().parent.parent
AGENT_LOCAL_DIR = ROOT_DIR / ".agent-local" / "agents"


class AgentSafeCommitError(Exception):
    pass


TOKEN_USAGE_FILENAME_RE = re.compile(r"token-usage-(\d+)\.json$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_safe_commit.py",
        description=(
            "Stage an explicit path allowlist and create an agent commit only if "
            "the git index contains exactly those paths."
        ),
    )
    parser.add_argument("--name", required=True, help="git user.name override for the commit")
    parser.add_argument("--email", required=True, help="git user.email override for the commit")
    parser.add_argument("--agent-id", required=True, help="Agent-Id trailer value for the commit")
    parser.add_argument(
        "--model-id",
        help="deprecated compatibility flag; commit trailers now read Model from codex_thread_metadata.py",
    )
    parser.add_argument(
        "--state-db",
        help="optional explicit state_*.sqlite path passed through to codex_thread_metadata.py",
    )
    parser.add_argument("-m", "--message", required=True, help="commit message")
    parser.add_argument(
        "--allow-empty",
        action="store_true",
        help="allow creating an empty commit when the allowlist produces no staged diff",
    )
    parser.add_argument(
        "paths",
        nargs="+",
        help="explicit repo-relative file paths to stage and allow in the commit",
    )
    return parser.parse_args()


def run_git(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        ["git", *args],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or f"git {' '.join(args)} failed"
        raise AgentSafeCommitError(message)
    return proc


def normalize_paths(raw_paths: list[str]) -> list[str]:
    normalized: list[str] = []
    seen: set[str] = set()
    for raw in raw_paths:
        candidate = raw.strip()
        if not candidate:
            raise AgentSafeCommitError("empty path is not allowed")
        if Path(candidate).is_absolute():
            raise AgentSafeCommitError(f"path must be repo-relative, not absolute: {raw}")
        path = Path(candidate)
        if ".." in path.parts:
            raise AgentSafeCommitError(f"path must stay inside the repo: {raw}")
        normalized_path = path.as_posix()
        if normalized_path not in seen:
            normalized.append(normalized_path)
            seen.add(normalized_path)
    return normalized


def verify_paths_exist(paths: list[str]) -> None:
    missing: list[str] = []
    for path in paths:
        if (ROOT_DIR / path).exists():
            continue
        tracked_in_head = run_git("cat-file", "-e", f"HEAD:{path}", check=False)
        if tracked_in_head.returncode == 0:
            continue
        missing.append(path)
    if missing:
        raise AgentSafeCommitError(
            "cannot stage missing paths: " + ", ".join(sorted(missing))
        )


def path_tracked_in_head(path: str) -> bool:
    return run_git("cat-file", "-e", f"HEAD:{path}", check=False).returncode == 0


def stage_paths(paths: list[str]) -> None:
    for path in paths:
        if (ROOT_DIR / path).exists():
            run_git("add", "-A", "--", path)
            continue
        if path_tracked_in_head(path):
            run_git("rm", "-f", "--ignore-unmatch", "--", path)
            continue
        raise AgentSafeCommitError(f"cannot stage missing paths: {path}")


def staged_paths() -> list[str]:
    proc = run_git("diff", "--cached", "--name-only", "--diff-filter=ACMRD")
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def load_codex_metadata(args: argparse.Namespace) -> tuple[str, str]:
    cmd = [
        sys.executable,
        str(ROOT_DIR / "scripts" / "codex_thread_metadata.py"),
        "--cwd",
        str(ROOT_DIR),
        "--shell",
    ]
    if args.state_db:
        cmd.extend(["--state-db", args.state_db])
    proc = subprocess.run(
        cmd,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or "codex_thread_metadata.py failed"
        raise AgentSafeCommitError(
            "could not load commit metadata from codex_thread_metadata.py: " + message
        )

    values: dict[str, str] = {}
    for line in proc.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        try:
            parsed = json.loads(value)
        except json.JSONDecodeError:
            parsed = value
        values[key] = str(parsed)

    model = values.get("MODEL")
    effort = values.get("EFFORT")
    if not model or not effort:
        raise AgentSafeCommitError(
            "codex_thread_metadata.py did not return MODEL and EFFORT in --shell mode"
        )
    return model, effort


def workcycle_dir(agent_id: str) -> Path:
    return AGENT_LOCAL_DIR / agent_id / "workcycles"


def latest_workcycle_batch_num(agent_id: str) -> int | None:
    directory = workcycle_dir(agent_id)
    if not directory.exists():
        return None
    latest: int | None = None
    for path in directory.iterdir():
        match = TOKEN_USAGE_FILENAME_RE.fullmatch(path.name)
        if match is None:
            continue
        batch = int(match.group(1))
        if latest is None or batch > latest:
            latest = batch
    return latest


def load_json_dict(path: Path) -> dict[str, object] | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    return payload if isinstance(payload, dict) else None


def token_snapshot_path(agent_id: str, batch_num: int) -> Path:
    return workcycle_dir(agent_id) / f"token-usage-{batch_num}.json"


def end_token_snapshot_path(agent_id: str, batch_num: int) -> Path:
    return workcycle_dir(agent_id) / f"token-usage-end-{batch_num}.json"


def estimate_token_spent(agent_id: str) -> int | None:
    batch_num = latest_workcycle_batch_num(agent_id)
    if batch_num is None:
        return None

    start_snapshot = load_json_dict(token_snapshot_path(agent_id, batch_num))
    if start_snapshot is None:
        return None

    thread_id = start_snapshot.get("thread_id")
    start_total = start_snapshot.get("input_tokens")
    if not isinstance(thread_id, str) or not thread_id.strip() or not isinstance(start_total, int):
        return None

    end_snapshot = load_json_dict(end_token_snapshot_path(agent_id, batch_num))
    if end_snapshot is None:
        end_snapshot = load_latest_usage_snapshot(cwd=str(ROOT_DIR), thread_id=thread_id)
    if end_snapshot is None:
        return None

    end_thread = end_snapshot.get("thread_id")
    end_total = end_snapshot.get("input_tokens")
    if not isinstance(end_thread, str) or end_thread != thread_id or not isinstance(end_total, int):
        return None
    if end_total < start_total:
        return None
    return end_total - start_total


def format_token_spent(value: int) -> str:
    if value < 1000:
        return str(value)
    return f"{round(value / 1000):,}K"


def create_commit(args: argparse.Namespace, allowed_paths: list[str]) -> str:
    model, effort = load_codex_metadata(args)
    token_spent = estimate_token_spent(args.agent_id)
    token_spent_trailer = (
        f"Token-Spent: {format_token_spent(token_spent)}\n" if token_spent is not None else ""
    )
    commit_message = (
        args.message.rstrip()
        + (
            f"\n\nAgent-Id: {args.agent_id}\n"
            f"Model: {model}\n"
            f"Reasoning-Effort: {effort}\n"
            f"{token_spent_trailer}"
        )
    )
    commit_args = [
        "-c",
        f"user.name={args.name}",
        "-c",
        f"user.email={args.email}",
        "commit",
        "--no-gpg-sign",
    ]
    if args.allow_empty:
        commit_args.append("--allow-empty")
    commit_args.extend(["-m", commit_message])
    proc = run_git(*commit_args)
    return proc.stdout.strip()


def main() -> int:
    args = parse_args()

    try:
        allowed_paths = normalize_paths(args.paths)
        verify_paths_exist(allowed_paths)

        stage_paths(allowed_paths)

        actual_staged = staged_paths()
        expected = set(allowed_paths)
        actual = set(actual_staged)

        extra = sorted(actual - expected)
        missing = sorted(expected - actual)
        if extra or missing:
            parts: list[str] = [
                "refusing to commit because the staged index does not match the explicit allowlist."
            ]
            if extra:
                parts.append("extra staged paths: " + ", ".join(extra))
            if missing:
                parts.append("allowed paths missing from staged diff: " + ", ".join(missing))
            parts.append(
                "review `git diff --cached --name-only` and unstage unrelated files before retrying."
            )
            raise AgentSafeCommitError(" ".join(parts))

        if not actual_staged and not args.allow_empty:
            raise AgentSafeCommitError(
                "no staged changes remain after filtering to the explicit allowlist; "
                "nothing to commit"
            )

        print(create_commit(args, allowed_paths))
        return 0
    except AgentSafeCommitError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
