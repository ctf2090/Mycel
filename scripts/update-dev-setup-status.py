#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
CHECK_DEV_ENV = ROOT_DIR / "scripts" / "check-dev-env.sh"
DEFAULT_OUTPUT = ROOT_DIR / ".agent-local" / "dev-setup-status.md"
AGENT_LOCAL_DIR = (ROOT_DIR / ".agent-local").resolve()
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))


class DevSetupStatusError(Exception):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Write the local dev-setup readiness file from scripts/check-dev-env.sh results."
    )
    parser.add_argument(
        "--actor",
        default="unknown",
        help="actor label to record in the status file, for example doc-6 or coding-2",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT),
        help="output Markdown file path; defaults to .agent-local/dev-setup-status.md",
    )
    parser.add_argument("--json", action="store_true", help="print a machine-readable summary")
    return parser.parse_args()


def run_check(*, full: bool) -> dict[str, Any]:
    command = [str(CHECK_DEV_ENV), "--json"]
    command_label = "scripts/check-dev-env.sh --json"
    if full:
        command = [str(CHECK_DEV_ENV), "--full", "--json"]
        command_label = "scripts/check-dev-env.sh --full --json"

    proc = subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True, check=False)
    stdout = proc.stdout.strip()
    if not stdout:
        detail = proc.stderr.strip() or "no JSON output received"
        raise DevSetupStatusError(f"{command_label} failed to produce JSON: {detail}")

    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise DevSetupStatusError(f"{command_label} produced invalid JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise DevSetupStatusError(f"{command_label} produced a non-object JSON payload")

    payload["_command"] = command_label
    payload["_exit_code"] = proc.returncode
    return payload


def format_checked_at(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%d %H:%M:%S UTC+8")


def tool_rows(payload: dict[str, Any], kind: str) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for entry in payload.get("checks", []):
        if not isinstance(entry, dict) or entry.get("kind") != kind:
            continue
        rows.append(
            {
                "name": str(entry.get("name", "")),
                "status": str(entry.get("status", "")),
                "detail": str(entry.get("detail", "")),
            }
        )
    return rows


def overall_status(quick: dict[str, Any], full: dict[str, Any] | None) -> str:
    if quick.get("status") == "passed" and full and full.get("status") == "passed":
        return "ready"
    return "not-ready"


def render_table(headers: list[str], rows: list[list[str]]) -> list[str]:
    lines = ["| " + " | ".join(headers) + " |", "|---" * len(headers) + "|"]
    for row in rows:
        lines.append("| " + " | ".join(row) + " |")
    return lines


def render_markdown(*, actor: str, quick: dict[str, Any], full: dict[str, Any] | None, now: datetime | None = None) -> str:
    status = overall_status(quick, full)
    repo_root = str(quick.get("repo_root", ROOT_DIR))
    command_rows = tool_rows(quick, "command")
    component_rows = tool_rows(quick, "component")
    validation_rows = tool_rows(full, "validation") if full else []
    full_run_status = "passed" if full and full.get("status") == "passed" else "failed" if full else "skipped"

    lines = [
        "# Dev Setup Status",
        "",
        f"- Status: {status}",
        f"- Checked at: {format_checked_at(now)}",
        f"- Checked by: {actor}",
        f"- Workspace: {repo_root}",
        "- Evidence source:",
        f"  - `{quick['_command']}` ({quick.get('status', 'unknown')})",
    ]
    if full:
        lines.append(f"  - `{full['_command']}` ({full.get('status', 'unknown')})")
    else:
        lines.append("  - full validation was skipped because the quick environment check did not pass")

    lines.extend(
        [
            "- Notes:",
            "  - This file is generated from the repo-local dev environment checks.",
            "  - New chats may skip bootstrap dev-setup checks only when this file says `Status: ready`.",
            "",
            "## Tool Checks",
            "",
        ]
    )

    lines.extend(
        render_table(
            ["Item", "Status", "Detail"],
            [[f"`{row['name']}`", row["status"], f"`{row['detail']}`" if row["detail"] else ""] for row in command_rows],
        )
    )
    lines.extend(["", "## Rust Components", ""])
    lines.extend(
        render_table(
            ["Item", "Status", "Detail"],
            [[f"`{row['name']}`", row["status"], f"`{row['detail']}`" if row["detail"] else ""] for row in component_rows],
        )
    )
    lines.extend(["", "## Repo Validation", "", f"- Full validation run: {'yes' if full else 'no'}", f"- Full validation status: {full_run_status}"])

    if validation_rows:
        lines.append("")
        lines.extend(
            render_table(
                ["Check", "Status", "Command"],
                [[row["name"], row["status"], f"`{row['detail']}`" if row["detail"] else ""] for row in validation_rows],
            )
        )
    else:
        lines.extend(["", "- No repo validation commands were recorded."])

    errors: list[str] = []
    if quick.get("error"):
        errors.append(f"quick check: {quick['error']}")
    if full and full.get("error"):
        errors.append(f"full check: {full['error']}")

    if errors:
        lines.extend(["", "## Diagnostics", ""])
        for error in errors:
            lines.append(f"- {error}")

    return "\n".join(lines) + "\n"


def write_status_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def resolve_output_path(path_value: str) -> Path:
    candidate = Path(path_value)
    if not candidate.is_absolute():
        candidate = ROOT_DIR / candidate
    resolved = candidate.resolve()
    try:
        resolved.relative_to(AGENT_LOCAL_DIR)
    except ValueError as exc:
        raise DevSetupStatusError("output path must live under .agent-local/") from exc
    return resolved


def main() -> int:
    args = parse_args()

    try:
        output_path = resolve_output_path(args.output)
        quick = run_check(full=False)
        full = run_check(full=True) if quick.get("status") == "passed" else None
        content = render_markdown(actor=args.actor, quick=quick, full=full)
        write_status_file(output_path, content)
        summary = {
            "status": overall_status(quick, full),
            "output": str(output_path),
            "quick_status": quick.get("status"),
            "full_status": full.get("status") if full else "skipped",
        }
        if args.json:
            print(json.dumps(summary))
        else:
            print(f"wrote {output_path} ({summary['status']})")
        return 0 if summary["status"] == "ready" else 1
    except DevSetupStatusError as exc:
        if args.json:
            print(json.dumps({"status": "failed", "output": str(output_path), "error": str(exc)}))
        else:
            print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
