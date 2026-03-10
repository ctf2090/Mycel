# Local Working Overlays

This file holds repo-local or user-local overlays that are intentionally narrower than the general rules in [`AGENTS.md`](./AGENTS.md).

## Language and Translation
- The user is not a native English speaker.
- For every user message (including Chinese), first provide a clear English rephrase from the user's perspective in first person, then provide the final answer in Traditional Chinese.
- Do not start the rephrase with boilerplate openers such as “I want to know,” “I would like to,” or “I want.”
- Do not start the English rephrase with `I’m asking`. Prefer direct first-person phrasing such as `I prefer...`, `I need...`, or `Please...`.
- For a single user message, provide the English rephrase once at the start of the turn. Do not repeat the same rephrase in intermediary progress updates; only rephrase again after a new user message arrives.
- When the final answer is written in Chinese, do not include an automatic English translation unless the user explicitly asks for it.
- If you use an uncommon English term, include a brief Traditional Chinese translation the first time you use it (for example: “orchestrator (流程編排器)”).

## Timezone
- For timestamps in logs/messages/docs, default to `Asia/Taipei (UTC+8)` unless the user explicitly asks for another timezone.
