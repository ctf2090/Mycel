#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path


class ContextUsageError(Exception):
    pass


CALIBRATION_SHORTCUTS: dict[str, dict[str, int | str]] = {
    "doc-sync-plan": {
        "mode": "additive",
        "estimated_tokens": 37000,
        "observed_tokens": 122000,
    }
}


@dataclass(frozen=True)
class UsageEstimate:
    used_tokens: int
    raw_used_tokens: int
    context_window: int
    warn_threshold: float
    rotate_threshold: float
    source: str
    calibration_summary: str | None = None

    @property
    def used_ratio(self) -> float:
        return self.used_tokens / self.context_window

    @property
    def used_percent(self) -> float:
        return self.used_ratio * 100

    @property
    def remaining_tokens(self) -> int:
        return max(self.context_window - self.used_tokens, 0)

    @property
    def remaining_percent(self) -> float:
        return max(100.0 - self.used_percent, 0.0)

    @property
    def status(self) -> str:
        if self.used_percent >= self.rotate_threshold:
            return "rotate_chat"
        if self.used_percent >= self.warn_threshold:
            return "prepare_handoff"
        return "ok"

    @property
    def recommendation(self) -> str:
        if self.status == "rotate_chat":
            return "Open a fresh chat now and continue from a concise handoff note."
        if self.status == "prepare_handoff":
            return "Prepare a handoff note now so the next chat can resume cleanly."
        return "Keep working in the current chat."


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/estimate_context_window_usage.py",
        description=(
            "Estimate current active context-window usage from a small JSON snapshot "
            "or an append-only token ledger."
        ),
    )
    parser.add_argument(
        "spec_path",
        nargs="?",
        default="-",
        help="JSON spec path, or '-' to read the spec from stdin",
    )
    parser.add_argument(
        "--warn-threshold",
        type=float,
        default=60.0,
        help="percent used at which to recommend preparing a handoff (default: 60)",
    )
    parser.add_argument(
        "--rotate-threshold",
        type=float,
        default=75.0,
        help="percent used at which to recommend opening a fresh chat (default: 75)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit machine-readable JSON instead of a text summary",
    )
    parser.add_argument(
        "--calibration-mode",
        choices=("additive", "multiplicative"),
        help="apply a calibration sample without embedding a calibration object in the JSON spec",
    )
    parser.add_argument(
        "--calibrate-estimated-tokens",
        type=int,
        help="estimated token count from a prior comparable round for CLI calibration",
    )
    parser.add_argument(
        "--calibrate-observed-tokens",
        type=int,
        help="observed token count from a prior comparable round for CLI calibration",
    )
    parser.add_argument(
        "--calibration-shortcut",
        choices=tuple(sorted(CALIBRATION_SHORTCUTS)),
        help="apply a named calibration sample without repeating raw token values",
    )
    return parser.parse_args()


def load_spec(spec_path: str) -> dict[str, object]:
    if spec_path == "-":
        raw = sys.stdin.read()
    else:
        raw = Path(spec_path).read_text(encoding="utf-8")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ContextUsageError(f"invalid JSON spec: {exc.msg}") from exc
    if not isinstance(payload, dict):
        raise ContextUsageError("JSON spec must be an object")
    return payload


def parse_required_positive_int(payload: dict[str, object], key: str) -> int:
    value = parse_positive_int(payload, key)
    if value is None:
        raise ContextUsageError(f"{key} must be provided as a positive integer")
    return value


def parse_positive_int(payload: dict[str, object], key: str) -> int | None:
    value = payload.get(key)
    if value is None:
        return None
    if not isinstance(value, int) or value <= 0:
        raise ContextUsageError(f"{key} must be a positive integer")
    return value


def parse_non_negative_int(entry: dict[str, object], key: str) -> int:
    value = entry.get(key, 0)
    if not isinstance(value, int) or value < 0:
        raise ContextUsageError(f"{key} must be a non-negative integer")
    return value


def validate_thresholds(args: argparse.Namespace) -> None:
    if args.warn_threshold <= 0 or args.warn_threshold >= 100:
        raise ContextUsageError("warn threshold must be greater than 0 and less than 100")
    if args.rotate_threshold <= 0 or args.rotate_threshold > 100:
        raise ContextUsageError("rotate threshold must be greater than 0 and at most 100")
    if args.rotate_threshold <= args.warn_threshold:
        raise ContextUsageError("rotate threshold must be greater than warn threshold")


def inject_cli_calibration(payload: dict[str, object], args: argparse.Namespace) -> dict[str, object]:
    estimated = args.calibrate_estimated_tokens
    observed = args.calibrate_observed_tokens
    mode = args.calibration_mode
    shortcut = args.calibration_shortcut

    if shortcut is not None and any(value is not None for value in (estimated, observed, mode)):
        raise ContextUsageError(
            "use either --calibration-shortcut or the explicit CLI calibration flags, not both"
        )
    if shortcut is None and estimated is None and observed is None and mode is None:
        return payload
    if "calibration" in payload:
        raise ContextUsageError(
            "spec already contains calibration; use either JSON calibration or CLI calibration"
        )
    if shortcut is not None:
        updated = dict(payload)
        updated["calibration"] = dict(CALIBRATION_SHORTCUTS[shortcut])
        return updated
    if estimated is None or observed is None or mode is None:
        raise ContextUsageError(
            "CLI calibration requires --calibration-mode, --calibrate-estimated-tokens, "
            "and --calibrate-observed-tokens together"
        )
    if estimated <= 0 or observed <= 0:
        raise ContextUsageError("CLI calibration token values must be positive integers")

    updated = dict(payload)
    updated["calibration"] = {
        "mode": mode,
        "estimated_tokens": estimated,
        "observed_tokens": observed,
    }
    return updated


def apply_calibration(raw_used_tokens: int, payload: dict[str, object]) -> tuple[int, str | None]:
    raw_calibration = payload.get("calibration")
    if raw_calibration is None:
        return raw_used_tokens, None
    if not isinstance(raw_calibration, dict):
        raise ContextUsageError("calibration must be an object when provided")

    mode = raw_calibration.get("mode", "additive")
    if mode not in {"additive", "multiplicative"}:
        raise ContextUsageError("calibration mode must be 'additive' or 'multiplicative'")

    estimated_tokens = parse_required_positive_int(raw_calibration, "estimated_tokens")
    observed_tokens = parse_required_positive_int(raw_calibration, "observed_tokens")

    if mode == "additive":
        delta = observed_tokens - estimated_tokens
        adjusted = max(raw_used_tokens + delta, 0)
        summary = (
            f"additive calibration (+{delta:,} tokens) from observed {observed_tokens:,} "
            f"vs estimated {estimated_tokens:,}"
        )
        return adjusted, summary

    adjusted = max(round(raw_used_tokens * (observed_tokens / estimated_tokens)), 0)
    summary = (
        f"multiplicative calibration (x{observed_tokens / estimated_tokens:.2f}) "
        f"from observed {observed_tokens:,} vs estimated {estimated_tokens:,}"
    )
    return adjusted, summary


def estimate_from_snapshot(payload: dict[str, object]) -> tuple[int, str] | None:
    current_input = parse_positive_int(payload, "current_input_tokens")
    last_output = parse_non_negative_int(payload, "last_output_tokens")
    if current_input is None:
        return None
    return current_input + last_output, "snapshot"


def estimate_from_ledger(payload: dict[str, object]) -> tuple[int, str] | None:
    raw_turns = payload.get("turns")
    if raw_turns is None:
        return None
    if not isinstance(raw_turns, list) or not raw_turns:
        raise ContextUsageError("turns must be a non-empty array when provided")
    total = 0
    for index, raw_turn in enumerate(raw_turns, start=1):
        if not isinstance(raw_turn, dict):
            raise ContextUsageError(f"turn {index} must be an object")
        added_tokens = raw_turn.get("added_tokens")
        if not isinstance(added_tokens, int) or added_tokens < 0:
            raise ContextUsageError(f"turn {index} added_tokens must be a non-negative integer")
        total += added_tokens
    return total, "ledger"


def build_estimate(payload: dict[str, object], args: argparse.Namespace) -> UsageEstimate:
    validate_thresholds(args)
    context_window = parse_positive_int(payload, "context_window")
    if context_window is None:
        raise ContextUsageError("context_window must be provided as a positive integer")

    source_estimate = estimate_from_snapshot(payload)
    if source_estimate is None:
        source_estimate = estimate_from_ledger(payload)
    if source_estimate is None:
        raise ContextUsageError(
            "spec must provide either current_input_tokens or a non-empty turns array"
        )

    raw_used_tokens, source = source_estimate
    used_tokens, calibration_summary = apply_calibration(raw_used_tokens, payload)
    return UsageEstimate(
        used_tokens=used_tokens,
        raw_used_tokens=raw_used_tokens,
        context_window=context_window,
        warn_threshold=args.warn_threshold,
        rotate_threshold=args.rotate_threshold,
        source=source,
        calibration_summary=calibration_summary,
    )


def render_text(estimate: UsageEstimate) -> str:
    lines = [
        "Context usage estimate",
        (
            f"- Estimated active context: {estimate.used_tokens:,} / "
            f"{estimate.context_window:,} tokens"
        ),
    ]
    if estimate.calibration_summary is not None:
        lines.append(f"- Raw estimate before calibration: {estimate.raw_used_tokens:,}")
        lines.append(f"- Calibration: {estimate.calibration_summary}")
    lines.extend(
        [
            (
                f"- Percent used: {estimate.used_percent:.1f}% "
                f"({estimate.remaining_percent:.1f}% left)"
            ),
            f"- Remaining tokens: {estimate.remaining_tokens:,}",
            f"- Status: {estimate.status}",
            f"- Source: {estimate.source}",
            f"- Recommendation: {estimate.recommendation}",
        ]
    )
    return "\n".join(lines)


def render_json(estimate: UsageEstimate) -> str:
    payload = {
        "used_tokens": estimate.used_tokens,
        "raw_used_tokens": estimate.raw_used_tokens,
        "context_window": estimate.context_window,
        "used_percent": round(estimate.used_percent, 1),
        "remaining_tokens": estimate.remaining_tokens,
        "remaining_percent": round(estimate.remaining_percent, 1),
        "status": estimate.status,
        "source": estimate.source,
        "recommendation": estimate.recommendation,
    }
    if estimate.calibration_summary is not None:
        payload["calibration"] = estimate.calibration_summary
    return json.dumps(payload, indent=2, sort_keys=True)


def main() -> int:
    args = parse_args()
    try:
        spec = inject_cli_calibration(load_spec(args.spec_path), args)
        estimate = build_estimate(spec, args)
    except ContextUsageError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if args.json:
        print(render_json(estimate))
    else:
        print(render_text(estimate))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
