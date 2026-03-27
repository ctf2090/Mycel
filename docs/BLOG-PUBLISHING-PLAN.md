# Mycel Blog Publishing Plan

Status: draft v1 publishing structure for the public Mycel blog

This note defines the recommended v1 structure for publishing a public Mycel
blog on top of the existing repository and Pages workflow.

It exists to answer a narrower question than the blog-series draft:

- where blog content should live
- how the public blog URLs should be shaped
- which metadata each article should carry
- which public pages should exist
- when we should keep the simple static model versus upgrade hosting

The companion content-strategy note remains:

- [`docs/AGENT-COORDINATION-BLOG-SERIES-DRAFT.md`](./AGENT-COORDINATION-BLOG-SERIES-DRAFT.md)

## Recommendation

Recommended v1:

- keep the Mycel blog on the current static site model
- publish the site through the existing GitHub Pages flow
- keep article source-of-truth in repo-local Markdown
- treat generated HTML pages as presentation surfaces, not authoring surfaces

Why this is the right v1:

- the current blog plan is a low-frequency series, not a newsroom
- the repo already has a public static Pages surface under `pages/`
- Mycel already treats Markdown as the canonical planning and docs layer
- this keeps publishing simple while leaving room for later upgrades

## v1 Goals

The first blog version should optimize for:

- low publishing friction
- repo-native review and version history
- stable public URLs
- clear linking from the existing Mycel landing pages
- lightweight generation without introducing a full CMS

The first blog version should not optimize for:

- rich editorial workflows
- multi-author dashboard tooling
- comments
- advanced personalization
- dynamic search infrastructure

## Source-of-Truth Model

Use this authority order:

1. blog source Markdown in the repository
2. generated public HTML pages under `pages/`
3. social-share and landing-page summaries that link to the blog

Interpretation:

- the Markdown files own article title, summary, date, series position, and body
- the public HTML owns presentation, navigation, and metadata tags
- landing-page teasers must summarize the article state, not invent it

## Recommended Content Locations

Recommended authoring location:

- `docs/blog/`

Recommended generated output locations:

- `pages/blog/index.html`
- `pages/blog/<slug>.html`

Recommended shared assets:

- `pages/assets/blog.css`
- `pages/assets/blog-index.css`
- `pages/assets/blog-post.css`

Recommended optional helper inputs:

- `pages/assets/blog-social/` for per-post preview images
- `scripts/` generation helpers if we later automate Markdown-to-HTML conversion

Why `docs/blog/` first:

- it matches the repo's existing habit of keeping authoritative text in Markdown
- it keeps blog drafts near related design notes and public docs
- it avoids inventing a parallel content system before we need one

## URL Structure

Recommended public URLs:

- index: `/Mycel/blog/`
- article: `/Mycel/blog/<slug>.html`

Recommended slug format:

- short, lowercase, hyphenated
- based on article concept, not publication date

Examples:

- `/Mycel/blog/multi-agent-coding-is-coordination.html`
- `/Mycel/blog/chat-memory-is-not-a-coordination-system.html`
- `/Mycel/blog/handoffs-should-be-first-class-artifacts.html`

Do not put the date in the URL for v1 unless we expect multiple posts with near-identical titles. Stable concept URLs are easier to keep alive if publication timing changes.

## Recommended File Naming

Recommended Markdown source naming:

- `docs/blog/YYYY-MM-DD.<slug>.md`

Examples:

- `docs/blog/2026-04-03.multi-agent-coding-is-coordination.md`
- `docs/blog/2026-04-10.chat-memory-is-not-a-coordination-system.md`

Why include the date in the source file but not the public URL:

- the file name stays sortable in git
- the public URL stays shorter and more durable

## Article Metadata

Each article should carry a small metadata block at the top.

Recommended fields:

- `title`
- `slug`
- `published`
- `status`
- `summary`
- `series`
- `part`
- `author`
- `canonical_url`
- `lang`
- `translation_of` when localized
- `tags`

Recommended v1 status values:

- `draft`
- `scheduled`
- `published`
- `archived`

Recommended minimal example:

```md
# Multi-Agent Coding Is a Coordination Problem, Not Just an Orchestration Problem

Status: published
Slug: multi-agent-coding-is-coordination
Published: 2026-04-03
Series: Agent Coordination Field Notes
Part: 1
Author: Mycel
Summary: Why real multi-agent coding failures come from ownership, handoffs, and stale state more often than from task routing.
Tags: multi-agent, coordination, git, handoff
Canonical URL: https://mycellayer.github.io/Mycel/blog/multi-agent-coding-is-coordination.html
Language: en
```

## Blog Index Fields

The blog index page should show only the fields needed to scan the series quickly.

Recommended index card fields:

- title
- one-sentence summary
- publication date
- series name
- part number
- tags
- reading-time estimate
- status when not yet published

Recommended optional index fields:

- hero eyebrow such as `Field Notes`
- one highlighted quote line
- translation availability

## Public Page Set

Recommended v1 public pages:

- `pages/blog/index.html`
- one HTML file per published article

Recommended index-page sections:

- hero with the blog promise
- latest post
- series overview
- all published posts
- optional upcoming posts section for `scheduled` entries

Recommended per-article page sections:

- article header
- metadata row
- article body
- series navigation
- related docs or source links
- footer CTA back to docs, roadmap, or repository

## Navigation Entry Points

Recommended primary navigation additions:

- add `Blog` to `pages/index.html`
- add `Blog` to `pages/docs.html`
- add localized `Blog` links to `pages/zh-TW/index.html` and `pages/zh-CN/index.html`

Recommended secondary entry points:

- add one blog card or teaser on the landing page
- add a docs-index card for the blog publishing plan or series draft when appropriate
- link from future progress or support surfaces only when the blog materially supports those pages

Recommended nav order after adding `Blog`:

- `README`
- `Docs`
- `Blog`
- `Progress`
- `Support`

## Language Strategy

Recommended v1 language strategy:

- publish English first
- add Traditional Chinese versions only for posts with durable strategic value
- do not promise same-day translation for every post in v1

If a localized post is published, use a dedicated localized URL family:

- `/Mycel/zh-TW/blog/<slug>.html`
- `/Mycel/zh-CN/blog/<slug>.html`

If localized blog pages are added later, the index pages should stay semantically aligned across maintained languages even if the prose differs.

## Generation Model

Recommended v1 generation model:

- start with a manual or lightly scripted Markdown-to-HTML flow
- keep the HTML templates simple and static
- avoid introducing a framework-specific app runtime for the first release

Reasonable v1 implementation shapes:

1. manual HTML authoring with Markdown source kept beside it
2. a small repo-local script that renders article Markdown into static HTML
3. a lightweight static-site generator only if we clearly outgrow the first two

## Social and SEO Baseline

Each published article page should include:

- canonical URL
- page title
- meta description
- Open Graph title, description, URL, and image
- Twitter card image

Recommended image strategy:

- use one default blog social image for v1
- add per-post images only when an article becomes an important outward-facing reference

## Upgrade Triggers

Stay on GitHub Pages for blog v1 unless one of these becomes important:

- pull-request preview deploys are now part of the editorial workflow
- we want branch previews for blog review by non-maintainers
- the blog needs redirects, rewrites, or richer routing rules
- analytics and experimentation become a real product need
- the team wants a dashboard-driven publishing workflow

If those triggers appear:

1. move to Cloudflare Pages when we still want static-first publishing with better deployment flexibility
2. move to Vercel when preview environments and app-style routing become first-class needs

## Proposed First Implementation Slice

Recommended first implementation slice:

1. create `docs/blog/`
2. create `pages/blog/index.html`
3. publish one pilot article page
4. add `Blog` to existing nav bars
5. keep the article body English-only for the pilot

This is enough to validate:

- whether the publishing shape feels coherent
- whether the blog belongs on the main site
- whether we need automation immediately or can wait
