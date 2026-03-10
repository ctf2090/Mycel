# Repo Working Agreements

## Git workflow
- Commit after every code change.
- Push to `origin main` after every code change.
- Multiple chats may work on the same project at the same time; each chat agent should just push its own commits to `origin/main`.
- Commit and push must run serially; only push after the commit has completed successfully, and do not run commit/push in parallel.
- If `git push origin main` is rejected because `origin/main` moved, run `git fetch origin`, `git rebase origin/main`, resolve any conflicts, and retry the push; do not force-push to bypass other chats' commits.
- During rebase conflicts, preserve user changes first, then already-pushed `origin/main` changes from other chats, and then re-apply this chat's work on top; if the conflict cannot be resolved confidently, stop and ask the user instead of guessing.
- After each completed code change and push, proactively check the latest CI workflow status and report any failures, but do not wait for CI to finish unless explicitly asked.
- Before starting any new work, first re-check the latest CI workflow status from the previous push and report any failures.

## Git identity (User vs Agent)
- User commits (VS Code): `ctf2090 <ctf2090@gmail.com>`
- Agent commits: `gpt-5:codex <codex@local>`
- On each new chat, the agent should determine the current `<model_family>:<agent_identity>` string before making commits.
- Preferred setup: keep repo `user.name/user.email` for the user; the agent overrides per commit:
  - `git -c user.name='gpt-5:codex' -c user.email=codex@local commit --no-gpg-sign -m "..."`

## Communication
- The user is not a native English speaker. For every user message (including Chinese), first provide a clear English rephrase from the user's perspective in first person, then provide the final answer in Traditional Chinese. Do not start the rephrase with boilerplate openers such as “I want to know,” “I would like to,” or “I want.”
- Do not start the English rephrase with `I’m asking`. Prefer direct first-person phrasing such as `I prefer...`, `I need...`, `Please...`, or another concise first-person/user-perspective form that matches the request.
- For a single user message, provide the English rephrase once at the start of the turn. Do not repeat the same rephrase in intermediary progress updates; only rephrase again after a new user message arrives.
- When the final answer is written in Chinese, do not include an automatic English translation unless the user explicitly asks for it.
- When replying, assume the agent and user are on the same team; use “we/our” phrasing where appropriate.
- When the user asks for a solution or recommendation, provide multiple viable options by default, not just a single “best” answer.
- For each option, include a one-line tradeoff (cost/time/risk/complexity) so the user can choose.
- As the project grows, proactively suggest mature modules, libraries, or frameworks at the right time when they would clearly reduce maintenance risk or simplify the design. Do not wait for the user to ask if the need is becoming obvious (for example, suggesting `clap` once CLI argument parsing becomes complex).
- When suggesting a mature module, library, or framework, explain why now is the right time, what problem it solves, and the main tradeoff of adopting it.
- After completing a piece of work, end with a short evaluation of valuable next-stage work and let the user choose from multiple concrete options by default.
- In final next-stage recommendations, put the highest-value option first by default.
- If the right choice depends on unknown constraints, ask 1–2 short clarifying questions, but still provide a best-effort set of options based on common assumptions.
- If you use an uncommon English term, include a brief Traditional Chinese translation the first time you use it (for example: “orchestrator (流程編排器)”).
- For timestamps in logs/messages/docs, default to `Asia/Taipei (UTC+8)` unless the user explicitly asks for another timezone.

## New chat bootstrap
- Run `git status -sb` to understand the repo state.
- Scan the repo layout with `ls` and prefer `rg --files` for fast file discovery.
- For a new Codespace project, ensure `gh` and `rg` are installed; install them if missing.
- If a task needs an additional tool or module, the agent should install it directly unless the user explicitly says not to.
- Reply with a short plan and the current repo status before making changes.

## .md
- Read .md from the root folder and its sub-folders, if it exists.

## Feature policy
- For new features, default to no backward compatibility unless the user explicitly requests compatibility support.
