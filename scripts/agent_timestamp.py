#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone


TAIPEI_TIMEZONE = timezone(timedelta(hours=8))


def format_now(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%d %H:%M:%S UTC+8")


def format_agent_label(agent: str | None, agent_uid: str | None) -> str | None:
    if agent and agent_uid and agent != agent_uid:
        return f"{agent} ({agent_uid})"
    return agent or agent_uid


def build_message(
    stage: str,
    *,
    agent: str | None,
    agent_uid: str | None = None,
    scope: str | None,
    now: datetime | None = None,
) -> str:
    label = "Before work" if stage == "before" else "After work"
    message = f"[{format_now(now)}] {label}"
    agent_label = format_agent_label(agent, agent_uid)
    if agent_label:
        message += f" | {agent_label}"
    if scope:
        message += f" | {scope}"
    return message


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print a human-readable timestamp line for agent work-cycle updates."
    )
    parser.add_argument("stage", choices=["before", "after"], help="whether the line is for the start or end of work")
    parser.add_argument("--agent", help="display id or agent uid to include in the message")
    parser.add_argument("--agent-uid", help="agent uid to pair with the display id in the message")
    parser.add_argument("--scope", help="scope label to include in the message")
    parser.add_argument(
        "--now",
        help="override the current time with an ISO 8601 timestamp; intended for tests",
    )
    return parser.parse_args()


def parse_now(raw: str | None) -> datetime | None:
    if raw is None:
        return None

    normalized = raw.strip()
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"
    elif len(normalized) >= 5 and normalized[-5] in {"+", "-"} and normalized[-3] != ":":
        normalized = f"{normalized[:-2]}:{normalized[-2:]}"
    return datetime.fromisoformat(normalized)


def main() -> int:
    args = parse_args()
    print(
        build_message(
            args.stage,
            agent=args.agent,
            agent_uid=args.agent_uid,
            scope=args.scope,
            now=parse_now(args.now),
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
