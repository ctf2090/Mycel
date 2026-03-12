# Repo Working Agreements

## Git workflow
- Current repo convention: the shared integration branch is `origin/main`.
- Push to `origin main` after every code change.
- Multiple chats may work on the same project at the same time; each chat agent should just push its own commits to `origin/main`.
- Commit and push must run serially; only push after the commit has completed successfully, and do not run commit/push in parallel.
- If `git push origin main` is rejected because `origin/main` moved, run `git fetch origin`, `git rebase origin/main`, resolve any conflicts, and retry the push; do not force-push to bypass other chats' commits.
- During rebase conflicts, preserve user changes first, then already-pushed `origin/main` changes from other chats, and then re-apply this chat's work on top; if the conflict cannot be resolved confidently, stop and ask the user instead of guessing.
- On every 20 commits to `origin/main`, check [`docs/PLANNING-SYNC-PLAN.md`](docs/PLANNING-SYNC-PLAN.md) as the single planning-sync entry point and follow it to refresh the current planning surfaces.
- Use `scripts/check-doc-refresh.sh` to check that 20-commit planning-sync cadence before or after a work batch; if it reports `due`, use [`docs/PLANNING-SYNC-PLAN.md`](docs/PLANNING-SYNC-PLAN.md) as the single entry point for the next docs sync.
- Do not check CI immediately after each push, since the workflow may still be running. Before starting the next piece of work, check the latest completed CI status for the previous push and report any failures.

## Git identity (User vs Agent)
- User commits should keep the user's normal local git identity.
- Agent commits should use a distinct agent identity instead of the user's identity.
- On each new chat, the agent should determine the current `<model_family>:<agent_identity>` string before making commits and use it as the per-commit `user.name` value.
- Preferred setup: keep repo `user.name/user.email` for the user; the agent overrides per commit:
  - `git -c user.name='<agent-name>' -c user.email='<agent-email>' commit --no-gpg-sign -m "..."`

## Local overlays
- If [`AGENTS-LOCAL.md`](./AGENTS-LOCAL.md) exists, apply it together with this file for repo-local or user-local communication, language, timezone, or workflow overlays.

## Communication
- When replying, assume the agent and user are on the same team; use “we/our” phrasing where appropriate.
- If the user misuses a technical or product term, correct it plainly and continue the answer using the correct term; do not mirror the incorrect term in the response except when briefly identifying the mistake.
- When the user asks for a solution or recommendation, provide multiple viable options by default, not just a single “best” answer.
- For each option, include a one-line tradeoff (cost/time/risk/complexity) so the user can choose.
- As the project grows, proactively suggest mature modules, libraries, or frameworks at the right time when they would clearly reduce maintenance risk or simplify the design. Do not wait for the user to ask if the need is becoming obvious (for example, suggesting `clap` once CLI argument parsing becomes complex).
- When code or tests start showing large repeated patterns, actively consider whether a mature module or tool should replace the repetition with a clearer structure (for example, `rstest` cases for repeated validation matrices).
- When suggesting a mature module, library, or framework, explain why now is the right time, what problem it solves, and the main tradeoff of adopting it.
- After completing a piece of work, end with a short evaluation of valuable next-stage work and, by default, offer multiple concrete options for the user to choose from.
- In final next-stage recommendations, put the highest-value option first and mark it as `(最有價值)`.
- When a next-stage option maps to a roadmap milestone, phase, or named track, include that roadmap location so the user can see where it fits in the plan.
- If the right choice depends on unknown constraints, ask 1–2 short clarifying questions, but still provide a best-effort set of options based on common assumptions.

## New chat bootstrap
- Run `git status -sb` to understand the repo state.
- Scan the repo layout with `ls` and prefer `rg --files` for fast file discovery.
- For a fresh local or Codespaces environment, ensure `gh` and `rg` are installed; install them if missing.
- For multi-agent startup and role assignment, read [`docs/AGENT-REGISTRY.md`](docs/AGENT-REGISTRY.md) first, then read the local registry file `.agent-local/agents.json`, and use `scripts/agent-claim.sh`, `scripts/agent-start.sh`, `scripts/agent-status.sh`, `scripts/agent-resume-check.sh`, `scripts/agent-stop.sh`, and `scripts/agent-recover.sh` as defined there.
- If the user did not assign a role for the new chat, use `scripts/agent-claim.sh auto`: it takes `coding` when there is no active `coding` agent, takes `doc` when active `coding >= 1` and active `doc == 0`, and takes `coding` when active `coding >= 1` and active `doc >= 1`.
- If a task needs an additional tool or module, the agent should install it directly unless the user explicitly says not to.
- Reply with a short plan and the current repo status before making changes.

## .md
- Read .md from the root folder and its sub-folders, if it exists.

## Scripts
- Do not inline Python code inside `scripts/*.sh`.
- If a script job is better expressed in Python, implement it as a real `scripts/*.py` file and keep any `.sh` entrypoint as a thin wrapper.

## Feature policy
- For new features, default to no backward compatibility unless the user explicitly requests compatibility support.
