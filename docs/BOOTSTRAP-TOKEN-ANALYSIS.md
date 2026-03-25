# Bootstrap Token Analysis

Status: internal note for Codex bootstrap token usage in this repo

## Scope

This note captures a token-usage analysis for one `boot coding` chat in the
Mycel repo.

The estimate is based on:

- `scripts/codex_token_usage_summary.py`
- `scripts/agent_work_cycle.py`
- the rollout JSONL for the matching Codex thread

The ranking below uses `input_tokens` growth as the main signal. It is a strong
estimate for "which work consumed the most context budget", but it is not a
precise per-tool billing breakdown.

## Key Reading

- The Codex UI value such as `44K/258K` reflects the thread's current total
  `input_tokens` against the model context window.
- The work-cycle field such as `+7K this cycle est.` reflects the increase
  during that specific work cycle only.
- Therefore, bootstrap batch 1 can legitimately show `usage 40K/258K | +7K this cycle est.`.
  The `40K` is the thread total at closeout time; the `+7K` is the bootstrap
  cycle's estimated contribution.

## Snapshot Used

For the analyzed bootstrap cycle:

- batch-1 start snapshot: `input_tokens = 33,370`
- batch-1 end snapshot: `input_tokens = 40,458`
- estimated bootstrap-cycle spend: `7,088` input tokens, shown in the UI as `+7K`

For the user's follow-up observation:

- the thread later reached about `44K`
- this matches later rollout rows such as `44,160` and `44,947`

## Pre-Boot Split Before Batch 1

The batch-1 start snapshot was `33,370`, so the thread had already accumulated
about `33.4K` input tokens before the bootstrap work cycle formally began.

That number should not be read as "the pre-boot Markdown alone cost 33K". A
better rough split is:

- user-supplied starting context: about `15.8K`
- assistant and tool-driven pre-bootstrap buildup: about `17.6K`

This split comes from the rollout rows observed before the batch-1 start
snapshot:

- `15,809` at the first captured row, which is the best available estimate for
  the initial user-provided context load
- `33,370` at the batch-1 start snapshot
- therefore `33,370 - 15,809 = 17,561` of additional input tokens were added
  before batch 1 began

In practice, that means the pre-bootstrap thread state was roughly:

### A. User-supplied starting context: about `15.8K`

This bucket mostly reflects content already present in the thread before the
bootstrap sequence really got moving, such as:

- the pasted `AGENTS.md instructions`
- environment metadata
- IDE context
- the short `boot coding` request

### B. Assistant and tool-driven pre-bootstrap buildup: about `17.6K`

This bucket reflects the assistant's own startup work before the formal
batch-1 begin snapshot:

- an early kickoff / planning increment of about `1.5K`
- the large bootstrap document-intake increment of about `12.6K`
- another roughly `3.4K` from bootstrap helpers, handoff lookup, and related
  pre-begin coordination

So the most accurate short answer is:

- not "pre-boot Markdown alone = 33K"
- but "the thread had about 33K loaded before batch 1, and about half of that
  came from the user's starting context while the other half came from the
  assistant's pre-bootstrap setup work"

## Ranking To About 44K

The following ranking estimates the largest token consumers from the beginning
of the chat up to the point where the thread was around `44K`.

### 1. Initial chat context load

Estimated cost: about `15.8K`

Main contributors:

- the pasted `AGENTS.md instructions`
- environment and IDE context
- the initial assistant reply and tool-planning frame

### 2. Bootstrap document intake

Estimated cost: about `12.6K`

Main contributors:

- `AGENTS.md`
- `AGENTS-LOCAL.md`
- `.agent-local/dev-setup-status.md`
- `docs/ROLE-CHECKLISTS/README.md`
- `docs/AGENT-REGISTRY.md`
- `.agent-local/agents.json`

This was the largest "work chunk" after the initial chat payload because the
assistant had to absorb the repo bootstrap rules before acting.

### 3. Same-role handoff mailbox review

Estimated cost: about `3.7K`

Main contributors:

- `.agent-local/mailboxes/agt_b2de3eff.md`

That mailbox contained multiple handoff entries, so it added noticeable context
weight even though it was only one file.

### 4. Running bootstrap and absorbing the result

Estimated cost: about `3.4K`

Main contributors:

- `python3 scripts/agent_bootstrap.py coding --model-id gpt-5-codex --scope "boot coding" --concise`
- the claimed role output
- the embedded latest-completed-CI baseline
- the bootstrap next-action summary

### 5. Bootstrap closeout and checklist correction

Estimated cost: about `2.3K`

Main contributors:

- reading the generated work-cycle checklist
- marking the relevant item-id states
- rerunning `scripts/agent_work_cycle.py end`

### 6. Other small bootstrap support steps

Estimated cost: about `7.1K` combined

This bucket includes smaller steps that mattered, but did not dominate on their
own:

- `agent_bootstrap.py --help`
- CI-related repo grep/search
- role-checklist read
- `git status -sb`
- `agent_work_cycle.py end --help`
- locating the latest `coding-10` registry entry
- short progress updates and final bootstrap reply text

## Practical Summary

If this thread is grouped by work type instead of individual steps, the rough
ordering is:

1. reading repo/bootstrap rules and registry state
2. executing bootstrap and absorbing the result
3. closing the cycle cleanly with checklist-driven admin steps

In short, the biggest token driver was not the bootstrap command itself. The
largest cost came from loading and retaining the repo's startup instructions and
coordination state.

## Follow-up Idea

If finer attribution is needed in the future, add a helper that groups rollout
token rows by phase, such as:

- prompt/context load
- docs/rules read
- bootstrap command
- CI lookup
- checklist closeout
- follow-up Q&A
