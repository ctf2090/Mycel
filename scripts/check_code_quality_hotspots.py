#!/usr/bin/env python3

from __future__ import annotations

import argparse
import ast
import io
import math
import re
import sys
import tokenize
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_DIRS = ("apps", "crates", "scripts")
SUPPORTED_EXTENSIONS = {".py", ".rs"}
EXCLUDED_PARTS = {".git", ".agent-local", "target", "node_modules", "__pycache__"}
DEFAULT_FILE_LINES = 800
DEFAULT_FUNCTION_LINES = 100
DEFAULT_LITERAL_REPEATS = 3
DEFAULT_NUMERIC_REPEATS = 3
MIN_LITERAL_LENGTH = 12
IGNORED_NUMERIC_LITERALS = {-1, 0, 1, 2}


@dataclass(frozen=True)
class Finding:
    kind: str
    path: str
    line: int
    message: str


@dataclass(frozen=True)
class FunctionHotspot:
    name: str
    line: int
    line_count: int


@dataclass(frozen=True)
class LiteralHotspot:
    line: int
    count: int
    preview: str


@dataclass(frozen=True)
class NumericHotspot:
    line: int
    count: int
    value: int


@dataclass(frozen=True)
class FileSummary:
    path: str
    line_count: int
    functions: tuple[FunctionHotspot, ...]
    literals: tuple[LiteralHotspot, ...]
    numeric_literals: tuple[NumericHotspot, ...]

    def score(self, args: argparse.Namespace) -> int:
        points = 0
        if self.line_count > args.file_lines:
            points += 1 + score_excess(
                self.line_count - args.file_lines,
                max(args.file_lines // 2, 1),
            )
        for hotspot in self.functions:
            points += 2 + score_excess(
                hotspot.line_count - args.function_lines,
                max(args.function_lines // 2, 1),
            )
        for hotspot in self.literals:
            points += 1 + max(0, hotspot.count - args.literal_repeats)
        for hotspot in self.numeric_literals:
            points += 1 + max(0, hotspot.count - args.numeric_repeats)
        return points


@dataclass(frozen=True)
class ScanResult:
    findings: tuple[Finding, ...]
    summary: FileSummary | None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/check_code_quality_hotspots.py",
        description=(
            "Scan source files for large-file, large-function, and repeated-literal "
            "code-quality hotspots."
        ),
    )
    parser.add_argument(
        "paths",
        nargs="*",
        help="repo-relative directories or files to scan; defaults to apps crates scripts",
    )
    parser.add_argument(
        "--file-lines",
        type=int,
        default=DEFAULT_FILE_LINES,
        help=f"warn when a file exceeds this many lines (default: {DEFAULT_FILE_LINES})",
    )
    parser.add_argument(
        "--function-lines",
        type=int,
        default=DEFAULT_FUNCTION_LINES,
        help=(
            f"warn when a function exceeds this many lines "
            f"(default: {DEFAULT_FUNCTION_LINES})"
        ),
    )
    parser.add_argument(
        "--literal-repeats",
        type=int,
        default=DEFAULT_LITERAL_REPEATS,
        help=(
            "warn when the same non-trivial literal appears at least this many times "
            f"(default: {DEFAULT_LITERAL_REPEATS})"
        ),
    )
    parser.add_argument(
        "--numeric-repeats",
        type=int,
        default=DEFAULT_NUMERIC_REPEATS,
        help=(
            "warn when the same non-trivial integer literal appears at least this many "
            f"times (default: {DEFAULT_NUMERIC_REPEATS})"
        ),
    )
    parser.add_argument(
        "--github-warning",
        action="store_true",
        help="emit GitHub Actions warning annotations for each finding",
    )
    parser.add_argument(
        "--fail-on-findings",
        action="store_true",
        help="exit non-zero when findings are present",
    )
    return parser.parse_args()


def iter_source_files(raw_paths: list[str]) -> list[Path]:
    scan_roots = raw_paths or list(DEFAULT_DIRS)
    files: list[Path] = []
    for raw in scan_roots:
        path = (ROOT / raw).resolve()
        if not path.exists():
            continue
        if path.is_file():
            if is_supported(path):
                files.append(path)
            continue
        for candidate in path.rglob("*"):
            if any(part in EXCLUDED_PARTS for part in candidate.parts):
                continue
            if candidate.is_file() and is_supported(candidate):
                files.append(candidate)
    return sorted(set(files))


def is_supported(path: Path) -> bool:
    return path.suffix in SUPPORTED_EXTENSIONS


def relative_path(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def file_line_count(text: str) -> int:
    return text.count("\n") + (0 if not text else 1)


def score_excess(excess: int, bucket_size: int) -> int:
    if excess <= 0:
        return 0
    return math.ceil(excess / max(bucket_size, 1))


def scan_file(path: Path, args: argparse.Namespace) -> ScanResult:
    text = path.read_text(encoding="utf-8")
    findings: list[Finding] = []
    line_count = file_line_count(text)
    rel = relative_path(path)
    if line_count > args.file_lines:
        findings.append(
            Finding(
                kind="file-size",
                path=rel,
                line=1,
                message=(
                    f"file has {line_count} lines (warning threshold: {args.file_lines}); "
                    "consider splitting by concern"
                ),
            )
        )

    literal_scan_allowed = "/tests/" not in f"/{rel}/"
    functions: list[FunctionHotspot] = []
    literals: list[LiteralHotspot] = []
    numeric_literals: list[NumericHotspot] = []

    if path.suffix == ".py":
        function_findings, functions = scan_python_functions(rel, text, args.function_lines)
        findings.extend(function_findings)
        if literal_scan_allowed:
            literal_findings_list, literals = scan_python_literals(rel, text, args.literal_repeats)
            findings.extend(literal_findings_list)
            numeric_findings_list, numeric_literals = scan_python_numeric_literals(
                rel, text, args.numeric_repeats
            )
            findings.extend(numeric_findings_list)
    elif path.suffix == ".rs":
        function_findings, functions = scan_rust_functions(rel, text, args.function_lines)
        findings.extend(function_findings)
        if literal_scan_allowed:
            literal_findings_list, literals = scan_rust_literals(rel, text, args.literal_repeats)
            findings.extend(literal_findings_list)
            numeric_findings_list, numeric_literals = scan_rust_numeric_literals(
                rel, text, args.numeric_repeats
            )
            findings.extend(numeric_findings_list)
    summary = None
    if findings:
        summary = FileSummary(
            path=rel,
            line_count=line_count,
            functions=tuple(functions),
            literals=tuple(literals),
            numeric_literals=tuple(numeric_literals),
        )
    return ScanResult(findings=tuple(findings), summary=summary)


def scan_python_functions(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[FunctionHotspot]]:
    tree = ast.parse(text)
    findings: list[Finding] = []
    hotspots: list[FunctionHotspot] = []
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        end_lineno = getattr(node, "end_lineno", node.lineno)
        line_count = end_lineno - node.lineno + 1
        if line_count > threshold:
            hotspots.append(
                FunctionHotspot(name=node.name, line=node.lineno, line_count=line_count)
            )
            findings.append(
                Finding(
                    kind="function-size",
                    path=rel,
                    line=node.lineno,
                    message=(
                        f"function `{node.name}` spans {line_count} lines "
                        f"(warning threshold: {threshold})"
                    ),
                )
            )
    return findings, hotspots


RUST_FN_PATTERN = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)


def scan_rust_functions(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[FunctionHotspot]]:
    findings: list[Finding] = []
    hotspots: list[FunctionHotspot] = []
    lines = text.splitlines()
    line_total = len(lines)
    idx = 0
    while idx < line_total:
        line = lines[idx]
        match = RUST_FN_PATTERN.match(line)
        if not match:
            idx += 1
            continue
        name = match.group(1)
        start = idx
        brace_depth = 0
        opened = False
        end = idx
        while end < line_total:
            current = lines[end]
            brace_depth += current.count("{")
            if current.count("{") > 0:
                opened = True
            brace_depth -= current.count("}")
            if opened and brace_depth <= 0:
                break
            end += 1
        line_count = end - start + 1
        if opened and line_count > threshold:
            hotspots.append(
                FunctionHotspot(name=name, line=start + 1, line_count=line_count)
            )
            findings.append(
                Finding(
                    kind="function-size",
                    path=rel,
                    line=start + 1,
                    message=(
                        f"function `{name}` spans {line_count} lines "
                        f"(warning threshold: {threshold})"
                    ),
                )
            )
        idx = max(end + 1, idx + 1)
    return findings, hotspots


def scan_python_literals(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[LiteralHotspot]]:
    occurrences: dict[str, list[int]] = {}
    token_stream = tokenize.generate_tokens(io.StringIO(text).readline)
    for token in token_stream:
        if token.type != tokenize.STRING:
            continue
        try:
            value = ast.literal_eval(token.string)
        except Exception:
            continue
        if not isinstance(value, str):
            continue
        if not is_non_trivial_literal(value):
            continue
        occurrences.setdefault(value, []).append(token.start[0])
    return literal_findings(rel, occurrences, threshold)


def scan_python_numeric_literals(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[NumericHotspot]]:
    tree = ast.parse(text)
    parents = {child: node for node in ast.walk(tree) for child in ast.iter_child_nodes(node)}
    occurrences: dict[int, list[int]] = {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, int) and not isinstance(node.value, bool):
            parent = parents.get(node)
            if (
                isinstance(parent, ast.UnaryOp)
                and isinstance(parent.op, ast.USub)
                and parent.operand is node
            ):
                continue
            value = node.value
            line = node.lineno
        elif (
            isinstance(node, ast.UnaryOp)
            and isinstance(node.op, ast.USub)
            and isinstance(node.operand, ast.Constant)
            and isinstance(node.operand.value, int)
            and not isinstance(node.operand.value, bool)
        ):
            value = -node.operand.value
            line = node.lineno
        else:
            continue
        if value in IGNORED_NUMERIC_LITERALS:
            continue
        occurrences.setdefault(value, []).append(line)
    return numeric_literal_findings(rel, occurrences, threshold)


RUST_LITERAL_PATTERN = re.compile(r'"((?:\\.|[^"\\])*)"')
RUST_NUMERIC_LITERAL_PATTERN = re.compile(r"(?<![A-Za-z0-9_])-?\d[\d_]*(?![A-Za-z0-9_])")


def scan_rust_literals(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[LiteralHotspot]]:
    occurrences: dict[str, list[int]] = {}
    for lineno, line in enumerate(text.splitlines(), start=1):
        for match in RUST_LITERAL_PATTERN.finditer(line):
            value = bytes(match.group(1), "utf-8").decode("unicode_escape")
            if not is_non_trivial_literal(value):
                continue
            occurrences.setdefault(value, []).append(lineno)
    return literal_findings(rel, occurrences, threshold)


def scan_rust_numeric_literals(
    rel: str, text: str, threshold: int
) -> tuple[list[Finding], list[NumericHotspot]]:
    occurrences: dict[int, list[int]] = {}
    for lineno, line in enumerate(text.splitlines(), start=1):
        scan_line = strip_rust_strings_and_line_comments(line)
        for match in RUST_NUMERIC_LITERAL_PATTERN.finditer(scan_line):
            start, end = match.span()
            before = scan_line[start - 1] if start > 0 else ""
            after = scan_line[end] if end < len(scan_line) else ""
            if before == "." or after == ".":
                continue
            value = int(match.group(0).replace("_", ""))
            if value in IGNORED_NUMERIC_LITERALS:
                continue
            occurrences.setdefault(value, []).append(lineno)
    return numeric_literal_findings(rel, occurrences, threshold)


def strip_rust_strings_and_line_comments(line: str) -> str:
    line_without_comments = re.sub(r"//.*", "", line)
    return RUST_LITERAL_PATTERN.sub('""', line_without_comments)


def is_non_trivial_literal(value: str) -> bool:
    stripped = value.strip()
    if len(stripped) < MIN_LITERAL_LENGTH:
        return False
    if "\n" in stripped:
        return False
    if not any(ch.isalpha() for ch in stripped):
        return False
    if stripped.startswith("http://") or stripped.startswith("https://"):
        return False
    if re.fullmatch(r"[A-Za-z0-9_./:#-]+", stripped):
        return False
    return True


def literal_findings(
    rel: str, occurrences: dict[str, list[int]], threshold: int
) -> tuple[list[Finding], list[LiteralHotspot]]:
    findings: list[Finding] = []
    hotspots: list[LiteralHotspot] = []
    for literal, lines in sorted(occurrences.items(), key=lambda item: (-len(item[1]), item[0])):
        if len(lines) < threshold:
            continue
        preview = literal if len(literal) <= 48 else literal[:45] + "..."
        hotspots.append(
            LiteralHotspot(line=lines[0], count=len(lines), preview=preview)
        )
        findings.append(
            Finding(
                kind="literal-repeat",
                path=rel,
                line=lines[0],
                message=(
                    f"non-trivial literal repeats {len(lines)} times "
                    f"(warning threshold: {threshold}): {preview!r}"
                ),
            )
        )
    return findings, hotspots


def numeric_literal_findings(
    rel: str, occurrences: dict[int, list[int]], threshold: int
) -> tuple[list[Finding], list[NumericHotspot]]:
    findings: list[Finding] = []
    hotspots: list[NumericHotspot] = []
    for value, lines in sorted(occurrences.items(), key=lambda item: (-len(item[1]), item[0])):
        if len(lines) < threshold:
            continue
        hotspots.append(NumericHotspot(line=lines[0], count=len(lines), value=value))
        findings.append(
            Finding(
                kind="numeric-literal-repeat",
                path=rel,
                line=lines[0],
                message=(
                    f"non-trivial integer literal repeats {len(lines)} times "
                    f"(warning threshold: {threshold}): {value}"
                ),
            )
        )
    return findings, hotspots


def render_ranked_candidates(summaries: list[FileSummary], args: argparse.Namespace) -> None:
    ranked = sorted(
        summaries,
        key=lambda summary: (
            -summary.score(args),
            -len(summary.functions),
            -len(summary.literals),
            -len(summary.numeric_literals),
            -summary.line_count,
            summary.path,
        ),
    )
    print("Ranked split candidates:")
    for index, summary in enumerate(ranked, start=1):
        file_note = (
            f"file {summary.line_count} lines"
            if summary.line_count > args.file_lines
            else f"file {summary.line_count} lines (under file threshold)"
        )
        function_note = (
            ", ".join(
                f"{hotspot.name}@L{hotspot.line}={hotspot.line_count}"
                for hotspot in summary.functions[:2]
            )
            if summary.functions
            else "none"
        )
        literal_note = (
            ", ".join(
                f"L{hotspot.line} x{hotspot.count}"
                for hotspot in summary.literals[:2]
            )
            if summary.literals
            else "none"
        )
        numeric_note = (
            ", ".join(
                f"{hotspot.value}@L{hotspot.line} x{hotspot.count}"
                for hotspot in summary.numeric_literals[:2]
            )
            if summary.numeric_literals
            else "none"
        )
        print(
            f"{index}. score={summary.score(args)} {summary.path} | "
            f"{file_note}; long functions={len(summary.functions)} [{function_note}]; "
            f"repeated literals={len(summary.literals)} [{literal_note}]; "
            f"numeric literals={len(summary.numeric_literals)} [{numeric_note}]"
        )


def render(findings: list[Finding], summaries: list[FileSummary], args: argparse.Namespace) -> int:
    if not findings:
        print(
            "No code-quality hotspots found "
            f"(file>{args.file_lines}, function>{args.function_lines}, "
            f"literal repeats>={args.literal_repeats}, "
            f"numeric repeats>={args.numeric_repeats})."
        )
        return 0

    counter = Counter(finding.kind for finding in findings)
    print(
        "Code-quality hotspot warnings "
        f"(file>{args.file_lines}, function>{args.function_lines}, "
        f"literal repeats>={args.literal_repeats}, "
        f"numeric repeats>={args.numeric_repeats}):"
    )
    for finding in findings:
        print(f"- [{finding.kind}] {finding.path}:{finding.line} {finding.message}")
        if args.github_warning:
            message = finding.message.replace("%", "%25").replace("\n", "%0A").replace("\r", "%0D")
            print(f"::warning file={finding.path},line={finding.line}::{message}")
    print(
        "Summary: "
        + ", ".join(f"{counter[key]} {key}" for key in sorted(counter))
    )
    print()
    render_ranked_candidates(summaries, args)
    return 1 if args.fail_on_findings else 0


def main() -> int:
    args = parse_args()
    findings: list[Finding] = []
    summaries: list[FileSummary] = []
    for path in iter_source_files(args.paths):
        result = scan_file(path, args)
        findings.extend(result.findings)
        if result.summary is not None:
            summaries.append(result.summary)
    return render(findings, summaries, args)


if __name__ == "__main__":
    raise SystemExit(main())
