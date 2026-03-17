#!/usr/bin/env python3

from __future__ import annotations

import html
import re
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import quote


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "pages" / "docs.html"
GITHUB_BLOB_BASE = "https://github.com/ctf2090/Mycel/blob/main/"

LANGUAGE_ORDER = {
    "default": 0,
    "en": 1,
    "zh-TW": 2,
    "zh-CN": 3,
}


@dataclass(frozen=True)
class Category:
    key: str
    title: str
    description: str
    patterns: tuple[str, ...]


CATEGORIES: tuple[Category, ...] = (
    Category(
        key="project-docs",
        title="Project Docs",
        description="High-level project overviews, roadmap materials, and public-facing reference notes.",
        patterns=(
            "README*.md",
            "PROJECT-INTENT*.md",
            "ROADMAP*.md",
            "IMPLEMENTATION-CHECKLIST*.md",
            "RUST-WORKSPACE.md",
            "SECURITY.md",
            "SPEC-ISSUES.md",
            "docs/DEV-SETUP*.md",
            "docs/FEATURE-REVIEW-CHECKLIST*.md",
            "docs/MYCEL-*.md",
            "docs/PROJECT-NAMING*.md",
            "docs/PROGRESS.md",
            "docs/TERMINOLOGY*.md",
        ),
    ),
    Category(
        key="protocol",
        title="Protocol",
        description="Core protocol and wire-level reference documents.",
        patterns=("PROTOCOL*.md", "WIRE-PROTOCOL*.md"),
    ),
    Category(
        key="profiles",
        title="Profiles",
        description="Profile documents that define governed interpretations and application-specific shapes.",
        patterns=("PROFILE.*.md",),
    ),
    Category(
        key="design-notes",
        title="Design Notes",
        description="Deeper design explorations for protocol, app-layer, and governance concepts.",
        patterns=("docs/design-notes/*.md",),
    ),
)


@dataclass
class DocEntry:
    category: Category
    path: str
    title: str
    summary: str
    status: str | None
    language: str

    @property
    def github_url(self) -> str:
        return GITHUB_BLOB_BASE + quote(self.path)


def iter_category_paths(category: Category) -> list[Path]:
    results: list[Path] = []
    seen: set[Path] = set()
    for pattern in category.patterns:
        for path in ROOT.glob(pattern):
            if not path.is_file():
                continue
            resolved = path.resolve()
            if resolved in seen:
                continue
            seen.add(resolved)
            results.append(path)
    return results


def detect_language(path: Path) -> str:
    name = path.name
    for suffix in (".zh-TW.md", ".zh-CN.md", ".en.md"):
        if name.endswith(suffix):
            return suffix.removeprefix(".").removesuffix(".md")
    return "default"


def cleanup_markdown_inline(text: str) -> str:
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)
    text = re.sub(r"`([^`]+)`", r"\1", text)
    text = re.sub(r"[*_~]", "", text)
    text = re.sub(r"\s+", " ", text)
    return text.strip()


def extract_title(lines: list[str], path: Path) -> str:
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("# "):
            return stripped[2:].strip()
    return path.stem


def extract_status(lines: list[str]) -> str | None:
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("Status:"):
            return cleanup_markdown_inline(stripped.partition(":")[2].strip())
    return None


def is_paragraph_break(stripped: str) -> bool:
    return (
        stripped.startswith("#")
        or stripped.startswith("- ")
        or stripped.startswith("* ")
        or stripped.startswith("> ")
        or stripped.startswith("|")
        or stripped.startswith("<")
        or re.match(r"\d+\.\s", stripped) is not None
    )


def extract_summary(lines: list[str]) -> str:
    title_seen = False
    in_code = False
    paragraph: list[str] = []

    for line in lines:
        stripped = line.strip()

        if stripped.startswith("```"):
            in_code = not in_code
            if paragraph:
                break
            continue
        if in_code:
            continue

        if not title_seen:
            if stripped.startswith("# "):
                title_seen = True
            continue

        if not stripped:
            if paragraph:
                break
            continue

        if stripped.startswith("Language:") or stripped.startswith("Status:"):
            continue

        if is_paragraph_break(stripped):
            if paragraph:
                break
            continue

        paragraph.append(stripped)

    summary = cleanup_markdown_inline(" ".join(paragraph))
    if not summary:
        return "No opening summary found in the source Markdown yet."
    if len(summary) <= 220:
        return summary
    return summary[:217].rstrip() + "..."


def parse_doc(path: Path, category: Category) -> DocEntry:
    lines = path.read_text(encoding="utf-8").splitlines()
    return DocEntry(
        category=category,
        path=path.relative_to(ROOT).as_posix(),
        title=extract_title(lines, path),
        summary=extract_summary(lines),
        status=extract_status(lines),
        language=detect_language(path),
    )


def include_language(language: str) -> bool:
    return language in {"default", "en"}


def collect_entries() -> list[DocEntry]:
    entries: list[DocEntry] = []
    for category in CATEGORIES:
        for path in iter_category_paths(category):
            entry = parse_doc(path, category)
            if not include_language(entry.language):
                continue
            entries.append(entry)
    entries.sort(
        key=lambda entry: (
            entry.category.title,
            entry.title.lower(),
            LANGUAGE_ORDER.get(entry.language, 99),
            entry.path,
        )
    )
    return entries


def render_status(status: str | None) -> str:
    if not status:
        return ""
    return f'<span class="meta-pill status-pill">{html.escape(status)}</span>'


def render_meta_row(entry: DocEntry) -> str:
    status = render_status(entry.status)
    if not status:
        return ""
    return f"""
              <div class="meta-row">
                {status}
              </div>
    """.rstrip()


def render_doc(entry: DocEntry) -> str:
    return f"""
          <article class="doc-card">
            <div class="doc-card-top">
{render_meta_row(entry)}
              <p class="doc-path"><code>{html.escape(entry.path)}</code></p>
            </div>
            <h3>{html.escape(entry.title)}</h3>
            <p class="doc-summary">{html.escape(entry.summary)}</p>
            <div class="doc-actions">
              <a class="button secondary" href="{html.escape(entry.github_url)}">Open Markdown</a>
            </div>
          </article>
    """.strip()


def render_category(category: Category, entries: list[DocEntry]) -> str:
    cards = "\n".join(render_doc(entry) for entry in entries)
    return f"""
      <section class="panel section-panel category-panel" id="{html.escape(category.key)}">
        <div class="section-heading">
          <div>
            <p class="section-kicker">{len(entries)} documents</p>
            <h2>{html.escape(category.title)}</h2>
          </div>
          <p class="section-copy">{html.escape(category.description)}</p>
        </div>
        <div class="docs-grid">
{cards}
        </div>
      </section>
    """.rstrip()


def render_page(entries: list[DocEntry]) -> str:
    category_sections = []
    for category in CATEGORIES:
        category_entries = [entry for entry in entries if entry.category.key == category.key]
        if not category_entries:
            continue
        category_sections.append(render_category(category, category_entries))

    section_html = "\n\n".join(category_sections)
    total_docs = len(entries)

    return f"""<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Mycel Docs Index</title>
    <meta
      name="description"
      content="Generated index of Mycel English project docs, protocol references, profiles, and design notes."
    >
    <meta name="theme-color" content="#0d6b57">
    <link rel="stylesheet" href="/Mycel/assets/landing-common.css">
    <link rel="stylesheet" href="/Mycel/assets/docs-index.css">
  </head>
  <body class="landing-page docs-page">
    <div class="page">
      <nav class="nav">
        <div class="brand">Mycel</div>
        <div class="nav-links">
          <a href="https://ctf2090.github.io/Mycel/zh-TW/">繁體中文</a>
          <a href="https://ctf2090.github.io/Mycel/zh-CN/">简体中文</a>
          <a href="https://github.com/ctf2090/Mycel">GitHub</a>
          <a href="https://github.com/ctf2090/Mycel/blob/main/README.md">README</a>
          <a href="/Mycel/docs.html" aria-current="page">Docs</a>
          <a href="/Mycel/progress.html">Progress</a>
          <a href="/Mycel/support.html">Support</a>
        </div>
      </nav>

      <section class="hero">
        <div class="panel hero-copy">
          <p class="eyebrow">Generated English Docs Index</p>
          <h1>Read the English Mycel docs by intent, not by directory.</h1>
          <p>
            This page is generated from selected English Markdown sources in the repository. It lists
            project docs, protocol references, profiles, and design notes with a short opening note
            taken from each file's heading section.
          </p>
          <p>
            The short note prefers a leading <code>Status:</code> line when one exists, otherwise it
            falls back to the first summary paragraph under the document title.
          </p>
          <div class="actions">
            <a class="button primary" href="https://github.com/ctf2090/Mycel">Open Repository</a>
            <a class="button secondary" href="https://github.com/ctf2090/Mycel/tree/main/docs/design-notes">Browse Design Notes</a>
          </div>
        </div>

        <aside class="panel hero-sidebar">
          <div class="stats-list">
            <div class="stats-card">
              <strong>Coverage</strong>
              <p>{total_docs} generated English entries across project docs, protocol references, profiles, and design notes.</p>
            </div>
            <div class="stats-card">
              <strong>Generation Rule</strong>
              <p>Built from repository Markdown before GitHub Pages deploy, so the public site stays static while the index stays current.</p>
            </div>
            <div class="stats-card">
              <strong>Source of Truth</strong>
              <p>The Markdown files remain authoritative. This page is a navigation layer, not a second source of content truth.</p>
            </div>
          </div>
        </aside>
      </section>

{section_html}

      <footer>
        Generated by <code>scripts/generate_docs_index.py</code> from tracked Markdown sources.
      </footer>
    </div>
  </body>
</html>
"""


def main() -> int:
    entries = collect_entries()
    rendered = render_page(entries)
    normalized = "\n".join(line.rstrip() for line in rendered.splitlines()) + "\n"
    OUTPUT.write_text(normalized, encoding="utf-8")
    print(f"generated {len(entries)} entries into {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
