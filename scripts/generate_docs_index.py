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

LANGUAGE_LABELS = {
    "default": "Default",
    "en": "English",
    "zh-TW": "繁體中文",
    "zh-CN": "简体中文",
}

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

    @property
    def language_label(self) -> str:
        return LANGUAGE_LABELS.get(self.language, self.language)


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


def collect_entries() -> list[DocEntry]:
    entries: list[DocEntry] = []
    for category in CATEGORIES:
        for path in iter_category_paths(category):
            entries.append(parse_doc(path, category))
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


def render_doc(entry: DocEntry) -> str:
    return f"""
          <article class="doc-card">
            <div class="doc-card-top">
              <div class="meta-row">
                <span class="meta-pill">{html.escape(entry.language_label)}</span>
                {render_status(entry.status)}
              </div>
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
      content="Generated index of Mycel project docs, protocol references, profiles, and design notes."
    >
    <meta name="theme-color" content="#0d6b57">
    <style>
      :root {{
        --landing-body-font: "IBM Plex Sans", "Segoe UI", sans-serif;
        --landing-heading-font: "IBM Plex Serif", Georgia, serif;
        --landing-heading-line-height: 1.04;
        --landing-card-line-height: 1.62;
        --landing-section-line-height: 1.72;
        --bg: #f3efe6;
        --surface: rgba(255, 250, 242, 0.9);
        --surface-strong: #fffaf1;
        --text: #1d2a26;
        --muted: #5a6a62;
        --accent: #0d6b57;
        --accent-strong: #084d3f;
        --line: rgba(29, 42, 38, 0.12);
        --shadow: 0 24px 80px rgba(20, 32, 27, 0.12);
      }}

      .docs-page .hero {{
        display: grid;
        grid-template-columns: minmax(0, 1.4fr) minmax(280px, 0.8fr);
        gap: 22px;
      }}

      .docs-page .hero-copy,
      .docs-page .hero-sidebar {{
        padding: 34px;
      }}

      .docs-page .eyebrow,
      .docs-page .section-kicker {{
        margin: 0 0 10px;
        font-size: 0.84rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--accent-strong);
      }}

      .docs-page h1 {{
        font-size: clamp(2.8rem, 6vw, 5rem);
        margin-bottom: 18px;
      }}

      .docs-page .hero-copy p,
      .docs-page .section-copy,
      .docs-page .hero-sidebar p {{
        margin: 0;
        color: var(--muted);
        line-height: 1.72;
      }}

      .docs-page .hero-copy p + p {{
        margin-top: 14px;
      }}

      .docs-page .stats-list {{
        display: grid;
        gap: 14px;
      }}

      .docs-page .stats-card {{
        padding: 18px 20px;
        border-radius: 20px;
        background: var(--surface-strong);
        border: 1px solid var(--line);
      }}

      .docs-page .stats-card strong {{
        display: block;
        margin-bottom: 8px;
        color: var(--accent-strong);
        letter-spacing: 0.05em;
        text-transform: uppercase;
        font-size: 0.88rem;
      }}

      .docs-page .section-heading {{
        display: grid;
        grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
        gap: 22px;
        align-items: start;
        margin-bottom: 22px;
      }}

      .docs-page .section-heading h2 {{
        font-size: clamp(2rem, 4vw, 3rem);
      }}

      .docs-page .docs-grid {{
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 18px;
      }}

      .docs-page .doc-card {{
        display: flex;
        flex-direction: column;
        gap: 14px;
        padding: 24px;
        border-radius: 22px;
        background: var(--surface-strong);
        border: 1px solid var(--line);
      }}

      .docs-page .doc-card-top {{
        display: flex;
        flex-direction: column;
        gap: 10px;
      }}

      .docs-page .doc-card h3 {{
        font-size: 1.5rem;
      }}

      .docs-page .meta-row {{
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }}

      .docs-page .meta-pill {{
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 7px 11px;
        border-radius: 999px;
        border: 1px solid var(--line);
        background: rgba(13, 107, 87, 0.08);
        color: var(--accent-strong);
        font-size: 0.82rem;
        font-weight: 600;
      }}

      .docs-page .status-pill {{
        background: rgba(175, 126, 53, 0.14);
        color: #815314;
      }}

      .docs-page .doc-path {{
        margin: 0;
        color: var(--muted);
      }}

      .docs-page .doc-summary {{
        margin: 0;
        color: var(--muted);
        line-height: var(--landing-card-line-height);
        flex: 1;
      }}

      .docs-page .doc-actions {{
        display: flex;
        gap: 12px;
        margin-top: auto;
      }}

      .docs-page footer {{
        margin-top: 22px;
        padding: 22px 4px 0;
        color: var(--muted);
        font-size: 0.95rem;
      }}

      @media (max-width: 980px) {{
        .docs-page .hero,
        .docs-page .section-heading,
        .docs-page .docs-grid {{
          grid-template-columns: 1fr;
        }}
      }}
    </style>
    <link rel="stylesheet" href="/Mycel/assets/landing-common.css">
  </head>
  <body class="landing-page docs-page">
    <div class="page">
      <nav class="nav">
        <div class="brand">Mycel</div>
        <div class="nav-links">
          <a href="/Mycel/">Home</a>
          <a href="/Mycel/docs.html" aria-current="page">Docs</a>
          <a href="/Mycel/progress.html">Progress</a>
          <a href="https://github.com/ctf2090/Mycel">GitHub</a>
        </div>
      </nav>

      <section class="hero">
        <div class="panel hero-copy">
          <p class="eyebrow">Generated Document Index</p>
          <h1>Read the Mycel docs by intent, not by directory.</h1>
          <p>
            This page is generated from selected Markdown sources in the repository. It lists public
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
              <p>{total_docs} generated entries across project docs, protocol references, profiles, and design notes.</p>
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
