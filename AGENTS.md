# Repo Working Agreements

## Git workflow
- Current repo convention: the shared integration branch is `origin/main`.
- Push each chat agent's own commits to `origin/main` after every code change, even when multiple chats are working in parallel. <!-- item-id: git.push-after-own-change -->
- Commit and push must run serially; only push after the commit has completed successfully, and do not run commit/push in parallel.
- If `git push origin main` is rejected because `origin/main` moved, run `git fetch origin`, `git rebase origin/main`, resolve any conflicts, and retry the push; do not force-push to bypass other chats' commits.
- During rebase conflicts, preserve user changes first, then already-pushed `origin/main` changes from other chats, and then re-apply this chat's work on top; if the conflict cannot be resolved confidently, stop and ask the user instead of guessing.
- Use `scripts/check-plan-refresh.sh` to manage planning cadence: `sync doc` is due after 10 commits, `sync issue` is due after 10 commits, and `sync web` is due after 20 commits.
- The `doc` role owns `scripts/check-plan-refresh.sh` and must run it after each completed work item while preparing next items; if it reports `due`, include the due planning surfaces in the next items. `coding` and `delivery` agents must not run it.
- When `coding` or `delivery` work produces roadmap, checklist, progress-page, issue-triage, or process-status material that may affect planning sync, hand that material to `doc` through the registry mailbox instead of running `scripts/check-plan-refresh.sh` directly.
- Before starting `sync doc` or `sync web`, `doc` must scan the relevant handoff mailboxes for recent open or unresolved planning notes and use them as collection input for the sync batch.
- If the planning-refresh cadence checker reports `due`, use [`docs/PLANNING-SYNC-PLAN.md`](docs/PLANNING-SYNC-PLAN.md) as the single entry point for the next planning-sync batch.
- Do not check CI immediately after each push, since the workflow may still be running.
- Only the `coding` and `delivery` roles check the latest completed CI status for the previous push before starting the next piece of work and report any failures.
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
- Respect the user's preferred response language, including locale-specific variants when the user or repo overlay specifies them. <!-- item-id: communication.preferred-response-language -->
- When the user asks for a solution or recommendation, provide multiple viable options by default, not just a single “best” answer.
- For each option, include a one-line tradeoff (cost/time/risk/complexity) so the user can choose.
- As the project grows, proactively suggest mature modules, libraries, or frameworks at the right time when they would clearly reduce maintenance risk or simplify the design. Do not wait for the user to ask if the need is becoming obvious (for example, suggesting `clap` once CLI argument parsing becomes complex).
- When code or tests start showing large repeated patterns, actively consider whether a mature module or tool should replace the repetition with a clearer structure (for example, `rstest` cases for repeated validation matrices).
- When suggesting a mature module, library, or framework, explain why now is the right time, what problem it solves, and the main tradeoff of adopting it.

## New chat bootstrap
- Bootstrap fast path for a fresh chat:
  - scan the repo root with `ls`
  - read `AGENTS-LOCAL.md` if it exists, then read `.agent-local/dev-setup-status.md`
  - read [`docs/ROLE-CHECKLISTS/README.md`](docs/ROLE-CHECKLISTS/README.md), then read [`docs/AGENT-REGISTRY.md`](docs/AGENT-REGISTRY.md) and `.agent-local/agents.json`
  - run `scripts/agent_bootstrap.py <role> --model-id <model_id>` or `scripts/agent_bootstrap.py auto --model-id <model_id>`
  - if the role is `coding` or `delivery`, check the latest completed CI result for the previous push before starting implementation or delivery follow-up
- The fast path is the default startup flow. Defer broad context gathering until task work begins unless recovery, takeover, or an explicit user request requires it sooner.
- Default deferred reads after bootstrap:
  - `ROADMAP.md` and other broad planning docs
  - full mailbox scans unless the chat is resuming, taking over, or doing `doc` planning-sync work
  - full registry dumps beyond confirming active peers and the claimed agent state
  - broad `.md` sweeps outside the task area
- Scan the repo root with `ls` for a quick top-level layout before using narrower file discovery commands when needed. <!-- item-id: bootstrap.repo-layout -->
- Dev setup:
  - Before repeating environment checks, read `.agent-local/dev-setup-status.md` if it exists. <!-- item-id: bootstrap.read-dev-setup-status -->
  - If `.agent-local/dev-setup-status.md` says `Status: ready` and records the required tool/setup checks for this workspace, a new chat does not need to re-check dev setup during bootstrap. <!-- item-id: bootstrap.skip-dev-setup-when-ready -->
  - If `.agent-local/dev-setup-status.md` is missing or does not say `Status: ready`, read [`docs/DEV-SETUP.md`](docs/DEV-SETUP.md), ensure the required setup items are satisfied, and refresh `.agent-local/dev-setup-status.md` with the repo-local dev-setup status tool. <!-- item-id: bootstrap.refresh-dev-setup-when-needed -->
  - Use [`.agent-local/DEV-SETUP-STATUS.example.md`](.agent-local/DEV-SETUP-STATUS.example.md) as the template for the local status file. <!-- item-id: bootstrap.dev-setup-template -->
- Agent startup:
  - Before starting role-specific checklist work, read [`docs/ROLE-CHECKLISTS/README.md`](docs/ROLE-CHECKLISTS/README.md) as the entry point for canonical role checklist sources and per-agent checklist copy locations. <!-- item-id: bootstrap.read-role-checklists -->
  - For multi-agent startup and role assignment, read [`docs/AGENT-REGISTRY.md`](docs/AGENT-REGISTRY.md) first, then read the local registry file `.agent-local/agents.json`, and use the registry tool for role assignment and startup state. <!-- item-id: bootstrap.read-agent-registry -->
  - Preferred fast path after reading the startup instructions: `scripts/agent_bootstrap.py <role> --model-id <model_id>` to perform the repo-standard bootstrap flow without front-loading broader task research.
  - If the user did not assign a role for the new chat, use the registry tool to auto-claim a role, then tell the user which role was claimed before moving on to task work. <!-- item-id: bootstrap.claim-auto -->
  - A new chat should claim a fresh agent for itself, even when the role matches an older inactive agent. Only use `resume-check` or `recover` when the same returning chat is resuming its own existing `agent_uid`. <!-- item-id: bootstrap.claim-fresh-agent-for-new-chat -->
  - The registry tool generates the agent's bootstrap checklist template at `.agent-local/agents/<agent_uid>/checklists/AGENTS-bootstrap-checklist.md` if it does not exist yet.

## Work Cycle Workflow
- Before work in each user command cycle, use `scripts/agent_work_cycle.py`; it handles the active registry transition for the cycle and generates the next agent-specific workcycle checklist copy. <!-- item-id: workflow.touch-work-cycle -->
- Run `git status -sb` to understand the repo state. <!-- item-id: bootstrap.git-status -->
- If a task needs an additional tool or module, the agent should install it directly unless the user explicitly says not to. <!-- item-id: workflow.install-needed-tools -->
- Reply with a short plan and the current repo status before making changes. <!-- item-id: workflow.reply-with-plan-and-status -->
- Use the exact before/after timestamp line emitted by `scripts/agent_work_cycle.py`; do not hand-write replacements or swap in a different timestamp format. <!-- item-id: workflow.timestamped-commentary -->
- Do not immediately follow `scripts/agent_work_cycle.py` with a separate manual registry lifecycle change for the same work cycle. <!-- item-id: workflow.no-double-touch-finish -->
- Before ending each completed user command work cycle after bootstrap batch 1, append or update one same-role handoff entry in the agent's declared mailbox so the mailbox records the latest state for that cycle. Cross-role handoff entries are optional. At `scripts/agent_work_cycle.py end`, the mailbox must be left with exactly one open same-role handoff and at most one open cross-role handoff. If the new entry replaces an older open handoff in the same slot, mark the older one `superseded` first. Prefer `scripts/mailbox_handoff.py` so the tool renders the tracked template and automatically supersedes older open handoffs in the same slot before appending the new one. Bootstrap batch 1 should skip mailbox handoff and treat this item as not needed. <!-- item-id: workflow.mailbox-handoff-each-cycle -->
- When a work cycle produces any source changes, include a files-changed summary in the final reply before the next-stage options. Use `git show --stat HEAD` (or the relevant commit range) to generate it and render it as a Markdown table with columns: File, +/-, and a one-line note per file. <!-- item-id: workflow.files-changed-summary -->
- After work in each completed user command cycle, use `scripts/agent_work_cycle.py`; it handles the inactive registry transition for the cycle and checks for unchecked items in the agent-specific workcycle checklist. For batch 1 bootstrap work, it also checks the bootstrap checklist before ending cleanly. If the work-cycle tool reports a non-zero status, the cycle is not cleanly closed yet: fix the reported issue, then rerun it before reporting the cycle complete. <!-- item-id: workflow.finish-work-cycle -->
- In the final answer, when a work cycle has been closed with `scripts/agent_work_cycle.py end`, place the exact emitted `After work` timestamp line immediately before the next-stage options section. <!-- item-id: workflow.final-after-work-line-before-next-items -->
- After completing a piece of work, end with a short evaluation of valuable next-stage work and, by default, offer multiple concrete options for the user to choose from. <!-- item-id: workflow.next-stage-options -->
  - In final next-stage recommendations, put the highest-value option first and mark it as `(最有價值)`. <!-- item-id: workflow.next-stage-highest-value-first -->
  - When suggesting next-stage work or options, use numeric item indexes (`1. 2. 3.`), not dot bullets. <!-- item-id: workflow.next-stage-numbered-options -->
  - When a next-stage option maps to a roadmap milestone, phase, or named track, include that roadmap location so the user can see where it fits in the plan. <!-- item-id: workflow.next-stage-roadmap-location -->
  - If the right choice depends on unknown constraints, ask 1–2 short clarifying questions, but still provide a best-effort set of options based on common assumptions. <!-- item-id: workflow.next-stage-clarifying-questions -->

## Item-ID Checklists
- When an agent reads a Markdown file that carries `item-id` annotations, treat the tracked file as the canonical instruction source; do not use the tracked file itself as the personal work log.
- Before self-tracking progress, the agent should create its own checklist copy under its agent-local checklist directory, preferably with `scripts/item_id_checklist.py`.
- For `AGENTS.md`, the registry and work-cycle tools already generate the standard bootstrap/workcycle checklist copies automatically; use `scripts/item_id_checklist.py` directly when you need another source file or need to regenerate manually.
- In that agent-local copy, every `item-id` line should use checklist-style prefixes such as `- [ ]`, `- [X]`, `- [-]`, and `- [!]` so the agent can mark work in place without changing the tracked source file.
- Use these meanings consistently in the agent-local copy: `- [ ]` means not checked yet, `- [X]` means checked and completed without problems, `- [-]` means not needed for this work cycle, and `- [!]` means checked but problems were found.
- When an item is marked `- [!]`, the agent should add an indented subitem immediately below it explaining the problem.
- Agents may update their own checklist copy with `scripts/item_id_checklist_mark.py`.
- Only update checklist state in the agent's own checklist copy unless the source instructions themselves are being intentionally edited.

## .md
- Read .md from the root folder and its sub-folders, if it exists.

## Scripts
- Do not inline Python code inside `scripts/*.sh`.
- If a script job is better expressed in Python, implement it as a real `scripts/*.py` file and use the `.py` file itself as the entrypoint instead of adding a `.sh` wrapper.

## Feature policy
- For new features, default to no backward compatibility unless the user explicitly requests compatibility support.
