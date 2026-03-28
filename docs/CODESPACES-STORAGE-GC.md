# Codespaces Storage GC

This guide describes a safe, repeatable storage garbage-collection plan for the Mycel Codespaces workspace.

Use it when:

- the Codespace warns that disk space is low
- `/workspaces` is filling up from rebuildable artifacts
- we want a standard first-response playbook instead of ad-hoc cleanup

## Goals

- reclaim space without touching tracked source files
- prefer rebuildable outputs and caches over user worktree content
- make the cleanup plan visible before deletion

## Tool

Use `scripts/codespaces_storage_gc.py`.

Default behavior is a dry run that reports reclaimable paths under the current workspace.

```bash
python3 scripts/codespaces_storage_gc.py
```

To include common home-directory caches such as Cargo and npm:

```bash
python3 scripts/codespaces_storage_gc.py --include-home-caches
```

To actually remove the selected targets:

```bash
python3 scripts/codespaces_storage_gc.py --apply
python3 scripts/codespaces_storage_gc.py --apply --include-home-caches
```

To limit cleanup to specific targets:

```bash
python3 scripts/codespaces_storage_gc.py --target repo-target
python3 scripts/codespaces_storage_gc.py --apply --target cargo-registry-cache --target npm-cache
```

To integrate with other tooling:

```bash
python3 scripts/codespaces_storage_gc.py --json
```

## Default GC Plan

Run the plan in this order:

1. Dry-run the workspace-only targets and confirm the largest reclaimable path.
2. Remove workspace build outputs first, especially `target/`.
3. Re-check free space.
4. Only if pressure remains, include home-directory caches with `--include-home-caches`.
5. Rebuild or re-fetch caches as needed after cleanup.

## Cadence

Run the Codespaces storage GC review at least once every `400` commits.

As of `2026-03-26`, the current Mycel commit cadence in this clone is:

- `1334` commits between `2026-03-08 17:54:36 UTC+8` and `2026-03-26 17:18:27 UTC+8`
- about `74.21 commits/day`
- about `5.39 days` for every `400` commits, which is roughly `5 days 9 hours`

Treat the `400`-commit rule as the stable trigger and the day estimate as a moving operational reference that should be recalculated when the project cadence changes.

## Current Target Classes

- `repo-target`: Cargo build output under `target/`
- `repo-tmp`: workspace scratch data under `tmp/`
- `repo-pytest-cache`: `.pytest_cache/`
- `repo-node-cache`: `node_modules/.cache/`
- `cargo-registry-cache`: `~/.cargo/registry/cache/`
- `cargo-git-db`: `~/.cargo/git/db/`
- `npm-cache`: `~/.npm/_cacache/`
- `pip-cache`: `~/.cache/pip/`

The tool only operates on this allowlist and skips symlinks, missing paths, and non-directory targets.

## Backup Note

Some Codespace state lives outside the repository and should be included in a manual backup when we want to preserve the local shell or agent setup.

Back up these files explicitly:

- `/home/codespace/.codex/skills/boot-agent/agents/openai.yaml`
- `/home/codespace/.bashrc`

## Operational Notes

- Workspace targets are the default because they are the least surprising and usually reclaim the most space.
- Home caches are optional because they can slow down later installs or builds.
- The tool reports disk-free space before cleanup and after cleanup when `--apply` is used.
- If the workspace still runs low on storage after the allowlisted cleanup, inspect large nonstandard directories manually before deleting anything else.
