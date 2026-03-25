# Intentional Compaction Eval Checklist

Use this checklist when we intentionally drive the current Codex chat into
compaction to verify the repo-local guard and closeout behavior.

## Goal

Confirm that, after compaction:

1. The old pre-compaction agent no longer resumes normal work.
2. We distinguish old-agent recovery behavior from the new-agent guard block
   behavior inside the compacted thread.
3. The actual `compact_context_detected` / `blocked-closeout` path is verified
   on the newly claimed agent in that same compacted thread.
4. Optional: `agent_safe_commit.py` and `agent_push.py` are both refused for
   the actually blocked agent.

## Current Agent

- Display ID: `coding-10`
- Agent UID: `agt_74b1d9fb`

## Before Compaction

1. Do not start new feature work, create new commits, or make unrelated edits.
2. Keep the test target narrow: only validate the blocker and closeout flow.

## Trigger Begin After Compaction For The Old Agent

Run this in the same compacted chat:

```bash
python3 scripts/agent_work_cycle.py begin agt_74b1d9fb --scope "intentional compaction eval"
```

Expected result:

- command fails with `no active display_id; recover it before touch`
- no normal `Before work` line
- this does not by itself prove a compaction guard block
- this old agent is no longer the target for `blocked-closeout` validation

## Verify Guard State For The Old Agent

```bash
python3 scripts/agent_guard.py check agt_74b1d9fb --json
```

Expected result:

- `"blocked": false`
- no `compact_context_detected` block is recorded for `agt_74b1d9fb`

## Verify Old-Agent Closeout Behavior

```bash
python3 scripts/agent_work_cycle.py end agt_74b1d9fb
```

Expected result:

- command succeeds
- output includes `blocked_closeout: false`

Then try:

```bash
python3 scripts/agent_work_cycle.py end agt_74b1d9fb --blocked-closeout
```

Expected result:

- command is rejected
- output indicates the agent is not guard-blocked, so `--blocked-closeout`
  is not allowed

## Verify The Actual Compaction Block On The New Agent

In the same compacted thread, claim a fresh agent and use that new agent as the
real validation target for compaction guard behavior.

Expected result:

- the newly claimed agent, not `agt_74b1d9fb`, is the one that records
  `compact_context_detected`
- `agent_guard.py check <new-agent> --json` returns `"blocked": true`
- normal `python3 scripts/agent_work_cycle.py end <new-agent>` is rejected
- `python3 scripts/agent_work_cycle.py end <new-agent> --blocked-closeout`
  succeeds with `blocked_closeout: true`

## Optional Commit / Push Guard Checks

```bash
python3 scripts/agent_safe_commit.py --name 'gpt-5.4:agt_74b1d9fb' --email 'ctf2090+mycel@gmail.com' --agent-id 'agt_74b1d9fb' -m 'test: blocked guard' -- AGENTS.md
python3 scripts/agent_push.py HEAD
```

Expected result:

- both commands are refused with blocked-agent messaging when run against the
  actually blocked new agent in the compacted thread

## What To Bring Back

If we want to review the result in a fresh chat, bring back:

1. the failed post-compaction `begin` output for old agent `agt_74b1d9fb`
2. the JSON output of `agent_guard.py check agt_74b1d9fb --json`
3. the normal `end` success output for `agt_74b1d9fb`
4. the `end --blocked-closeout` rejection output for `agt_74b1d9fb`
5. the claim / guard / closeout outputs for the newly claimed blocked agent in
   the same compacted thread
