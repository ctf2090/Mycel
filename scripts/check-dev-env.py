#!/usr/bin/env python3
"""Check whether the local machine is ready for Mycel development."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check whether the local machine is ready for Mycel development."
    )
    parser.add_argument(
        "--full",
        action="store_true",
        help="Also run the first-pass validation commands from docs/DEV-SETUP.md.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit machine-readable JSON instead of human-oriented log lines.",
    )
    return parser.parse_args()


class CheckFailure(Exception):
    pass


def run_command(*args: str) -> str:
    try:
        return subprocess.check_output(args, text=True, stderr=subprocess.DEVNULL).splitlines()[0]
    except subprocess.CalledProcessError:
        return ""


def emit_text(enabled: bool, line: str) -> None:
    if enabled:
        print(line)


def load_toolchain_metadata() -> tuple[str, str]:
    rust_toolchain_path = ROOT_DIR / "rust-toolchain.toml"
    cargo_toml_path = ROOT_DIR / "Cargo.toml"
    if not rust_toolchain_path.is_file():
        raise CheckFailure("missing rust-toolchain.toml in repo root")
    if not cargo_toml_path.is_file():
        raise CheckFailure("missing Cargo.toml in repo root")

    with rust_toolchain_path.open("rb") as handle:
        toolchain_doc = tomllib.load(handle)
    with cargo_toml_path.open("rb") as handle:
        cargo_doc = tomllib.load(handle)

    toolchain_channel = str(toolchain_doc.get("toolchain", {}).get("channel", "unknown"))
    minimum_rust = str(cargo_doc.get("workspace", {}).get("package", {}).get("rust-version", "unknown"))
    return toolchain_channel, minimum_rust


def require_cmd(
    cmd: str,
    results: list[dict[str, str]],
    *,
    text_mode: bool,
    version_arg: str = "--version",
) -> None:
    resolved = shutil.which(cmd)
    if not resolved:
        results.append({"kind": "command", "name": cmd, "status": "missing", "detail": ""})
        raise CheckFailure(f"missing required command: {cmd}")

    version = run_command(cmd, version_arg)
    results.append({"kind": "command", "name": cmd, "status": "found", "detail": version})
    emit_text(text_mode, f"found {cmd:<8} {version}".rstrip())


def require_component(
    toolchain: str,
    component: str,
    results: list[dict[str, str]],
    *,
    text_mode: bool,
) -> None:
    proc = subprocess.run(
        ["rustup", "component", "list", "--toolchain", toolchain, "--installed"],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    installed = {line.strip().split("-", 1)[0] for line in proc.stdout.splitlines() if line.strip()}
    if component not in installed:
        results.append(
            {"kind": "component", "name": component, "status": "missing", "detail": toolchain}
        )
        raise CheckFailure(f"missing required Rust component on {toolchain}: {component}")

    results.append({"kind": "component", "name": component, "status": "found", "detail": toolchain})
    emit_text(text_mode, f"found component {component} on {toolchain}")


def run_check(
    label: str,
    command: list[str],
    results: list[dict[str, str]],
    *,
    text_mode: bool,
) -> None:
    emit_text(text_mode, f"running {label:<18} {' '.join(command)}")
    proc = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    output = proc.stdout + proc.stderr
    if proc.returncode == 0:
        results.append(
            {"kind": "validation", "name": label, "status": "passed", "detail": " ".join(command)}
        )
        if text_mode and output.strip():
            print(output.rstrip())
        return

    results.append(
        {"kind": "validation", "name": label, "status": "failed", "detail": " ".join(command)}
    )
    if text_mode and output.strip():
        print(output.rstrip())
    raise CheckFailure(f"validation step failed: {label}")


def emit_payload(
    *,
    status: str,
    mode: str,
    toolchain_channel: str,
    minimum_rust: str,
    results: list[dict[str, str]],
    json_mode: bool,
    error: str = "",
) -> None:
    if json_mode:
        payload: dict[str, object] = {
            "status": status,
            "mode": mode,
            "repo_root": str(ROOT_DIR),
            "required_toolchain_channel": toolchain_channel,
            "minimum_rust": minimum_rust,
            "checks": results,
        }
        if error:
            payload["error"] = error
        print(json.dumps(payload, separators=(",", ":")))
    elif status == "passed":
        print("dev environment check passed")
    else:
        print(error, file=sys.stderr)


def main() -> int:
    args = parse_args()
    mode = "full" if args.full else "quick"
    results: list[dict[str, str]] = []
    toolchain_channel = "unknown"
    minimum_rust = "unknown"

    try:
        toolchain_channel, minimum_rust = load_toolchain_metadata()
        emit_text(not args.json, "checking Mycel development environment")
        emit_text(not args.json, f"repo root: {ROOT_DIR}")
        emit_text(not args.json, f"required toolchain channel: {toolchain_channel}")
        emit_text(not args.json, f"workspace minimum Rust: {minimum_rust}")

        require_cmd("cargo", results, text_mode=not args.json)
        require_cmd("rustup", results, text_mode=not args.json)
        require_cmd("rustc", results, text_mode=not args.json)
        require_cmd("gh", results, text_mode=not args.json)
        require_cmd("rg", results, text_mode=not args.json)

        if toolchain_channel:
            require_component(toolchain_channel, "rustfmt", results, text_mode=not args.json)
            require_component(toolchain_channel, "clippy", results, text_mode=not args.json)

        if args.full:
            emit_text(not args.json, "running full validation pass")
            run_check("fmt", ["cargo", "fmt", "--all", "--check"], results, text_mode=not args.json)
            run_check("core-tests", ["cargo", "test", "-p", "mycel-core"], results, text_mode=not args.json)
            run_check("cli-tests", ["cargo", "test", "-p", "mycel-cli"], results, text_mode=not args.json)
            run_check("cli-info", ["cargo", "run", "-p", "mycel-cli", "--", "info"], results, text_mode=not args.json)
            run_check(
                "fixture-validate",
                [
                    "cargo",
                    "run",
                    "-p",
                    "mycel-cli",
                    "--",
                    "validate",
                    "fixtures/object-sets/minimal-valid/fixture.json",
                    "--json",
                ],
                results,
                text_mode=not args.json,
            )
            run_check(
                "sim-smoke",
                ["./sim/negative-validation/smoke.sh", "--summary-only"],
                results,
                text_mode=not args.json,
            )

        emit_payload(
            status="passed",
            mode=mode,
            toolchain_channel=toolchain_channel,
            minimum_rust=minimum_rust,
            results=results,
            json_mode=args.json,
        )
        return 0
    except CheckFailure as exc:
        emit_payload(
            status="failed",
            mode=mode,
            toolchain_channel=toolchain_channel,
            minimum_rust=minimum_rust,
            results=results,
            json_mode=args.json,
            error=str(exc),
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
