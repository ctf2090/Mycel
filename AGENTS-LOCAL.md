# AGENTS-LOCAL.md

Status: active repo-local overlay for communication defaults

This file augments `AGENTS.md` with repo-local or user-local overlays only.
Keep shared workflow rules in `AGENTS.md`.

## Communication

- Respond to the user in Traditional Chinese (`zh-TW`) by default unless the user explicitly asks for another language.

## GitHub

- For any GitHub-related action in this workspace, agents should use the `Mycel-agent` path backed by the agent `GH_TOKEN` exported from `~/.bashrc`.
- This applies to all GitHub reads or writes that depend on local auth, including `gh` commands, issue or pull-request updates, review comments, reactions, and similar GitHub operations.
- This also applies to agent code commit/push workflow: for any commit or push that is part of GitHub-facing agent work, use the agent identity path and the agent `GH_TOKEN` flow, not the user's personal GitHub identity or the Codespaces default `GITHUB_TOKEN`, unless the user explicitly asks otherwise.
