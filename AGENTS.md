# Repo Working Agreements

## Git workflow
- Current repo convention: the shared integration branch is `origin/main`.
- Push to `origin main` after every code change.
- Multiple chats may work on the same project at the same time; each chat agent should just push its own commits to `origin/main`.
- Commit and push must run serially; only push after the commit has completed successfully, and do not run commit/push in parallel.
- If `git push origin main` is rejected because `origin/main` moved, run `git fetch origin`, `git rebase origin/main`, resolve any conflicts, and retry the push; do not force-push to bypass other chats' commits.
- During rebase conflicts, preserve user changes first, then already-pushed `origin/main` changes from other chats, and then re-apply this chat's work on top; if the conflict cannot be resolved confidently, stop and ask the user instead of guessing.
- Use `scripts/check-plan-refresh.sh` to manage planning cadence: `sync doc` is due after 10 commits, `sync issue` is due after 10 commits, and `sync web` is due after 20 commits.
- The `doc` role owns `scripts/check-plan-refresh.sh` and must run it after each completed work item while preparing next items; if it reports `due`, include the due planning surfaces in the next items. `coding` agents must not run it.
- When `coding` work produces roadmap, checklist, progress-page, or issue-triage material that may affect planning sync, hand that material to `doc` through the registry mailbox instead of running `scripts/check-plan-refresh.sh` directly.
- Before starting `sync doc` or `sync web`, `doc` must scan the relevant handoff mailboxes for recent open or unresolved planning notes and use them as collection input for the sync batch.
- If `scripts/check-plan-refresh.sh` reports `due`, use [`docs/PLANNING-SYNC-PLAN.md`](docs/PLANNING-SYNC-PLAN.md) as the single entry point for the next planning-sync batch.
- Do not check CI immediately after each push, since the workflow may still be running.
- Only the `coding` role checks the latest completed CI status for the previous push before starting the next piece of work and reports any failures.
- The `doc` role does not check CI.

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
- When suggesting next-stage work or options, use numeric item indexes (`1. 2. 3.`), not dot bullets.
- When a next-stage option maps to a roadmap milestone, phase, or named track, include that roadmap location so the user can see where it fits in the plan.
- If the right choice depends on unknown constraints, ask 1–2 short clarifying questions, but still provide a best-effort set of options based on common assumptions.

## New chat bootstrap
- Scan the repo layout with `ls` and prefer `rg --files` for fast file discovery. <!-- item-id: bootstrap.repo-layout -->
- Before repeating environment checks, read `.agent-local/dev-setup-status.md` if it exists. <!-- item-id: bootstrap.read-dev-setup-status -->
- If `.agent-local/dev-setup-status.md` says `Status: ready` and records the required tool/setup checks for this workspace, a new chat does not need to re-check dev setup during bootstrap. <!-- item-id: bootstrap.skip-dev-setup-when-ready -->
- If `.agent-local/dev-setup-status.md` is missing or does not say `Status: ready`, read [`docs/DEV-SETUP.md`](docs/DEV-SETUP.md), ensure the required setup items are satisfied, and update `.agent-local/dev-setup-status.md` with the detailed tool/setup check results, preferably via `scripts/update-dev-setup-status.py --actor <role-id>`. <!-- item-id: bootstrap.refresh-dev-setup-when-needed -->
- Use [`.agent-local/DEV-SETUP-STATUS.example.md`](.agent-local/DEV-SETUP-STATUS.example.md) as the template for the local status file. <!-- item-id: bootstrap.dev-setup-template -->
- For multi-agent startup and role assignment, read [`docs/AGENT-REGISTRY.md`](docs/AGENT-REGISTRY.md) first, then read the local registry file `.agent-local/agents.json`, and use `scripts/agent_registry.py` subcommands as defined there. <!-- item-id: bootstrap.read-agent-registry -->
- If the user did not assign a role for the new chat, use `scripts/agent_registry.py claim auto`. <!-- item-id: bootstrap.claim-auto -->
- After claiming a role for the chat, tell the user which role was claimed before moving on to task work. <!-- item-id: bootstrap.announce-claimed-role -->

## Work Cycle Workflow
- Run `git status -sb` to understand the repo state. <!-- item-id: bootstrap.git-status -->
- For each user command work cycle, touch the active agent entry before working and mark it inactive after the work for that command finishes. <!-- item-id: workflow.touch-finish-work-cycle -->
- For each user command work cycle, post a short human-facing commentary line with a timestamp before work starts and after work ends. The timestamp must be visible in user-facing commentary, not only in terminal output or tool logs. Use the exact line format emitted by `scripts/agent_work_cycle.py begin|end <agent-ref> [--scope <scope-label>]`; do not hand-write, paraphrase, or replace it with dual-timezone text. Outside those canonical before/after lines, normal progress updates should not add hand-written date or time prefixes. `scripts/agent_timestamp.py` remains available only when a standalone timestamp line is needed and should keep the same single-line `UTC+8` format. <!-- item-id: workflow.timestamped-commentary -->
- When using `scripts/agent_work_cycle.py begin|end`, do not immediately follow it with a manual `scripts/agent_registry.py touch|finish` for the same work cycle; `begin` already performs `touch`, and `end` already performs `finish`. <!-- item-id: workflow.no-double-touch-finish -->
- If a task needs an additional tool or module, the agent should install it directly unless the user explicitly says not to. <!-- item-id: workflow.install-needed-tools -->
- Reply with a short plan and the current repo status before making changes. <!-- item-id: workflow.reply-with-plan-and-status -->

## Item-ID Checklists
- When an agent reads a Markdown file that carries `item-id` annotations, treat the tracked file as the canonical instruction source; do not use the tracked file itself as the personal work log.
- Before self-tracking progress, the agent should create its own copy under `.agent-local/agents/<agent_uid>/checklists/`, preferably with `python3 scripts/item_id_checklist.py <agent-ref> <source-md>`.
- In that agent-local copy, every `item-id` line should use checklist-style prefixes such as `- [ ]`, `- [X]`, and `- [!]` so the agent can mark work in place without changing the tracked source file.
- Use these meanings consistently in the agent-local copy: `- [ ]` means not checked yet, `- [X]` means checked and completed without problems, and `- [!]` means checked but problems were found.
- When an item is marked `- [!]`, the agent should add an indented subitem immediately below it explaining the problem.
- Agents may update their own checklist copy with `python3 scripts/item_id_checklist_mark.py <checklist-md> <item-id> --state checked|unchecked|problem [--problem "..."]`.
- Agents should update `[ ]` / `[X]` / `[!]` state only in their own checklist copy unless the source instructions themselves are being intentionally edited.

## .md
- Read .md from the root folder and its sub-folders, if it exists.

## Scripts
- Do not inline Python code inside `scripts/*.sh`.
- If a script job is better expressed in Python, implement it as a real `scripts/*.py` file and use the `.py` file itself as the entrypoint instead of adding a `.sh` wrapper.

## Feature policy
- For new features, default to no backward compatibility unless the user explicitly requests compatibility support.
