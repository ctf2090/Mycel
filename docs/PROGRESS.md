# Mycel Progress View

Status: draft

This page turns [`ROADMAP.md`](../ROADMAP.md) and [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) into one quick progress view.

## Current Lane

The current build lane is:

1. close `M1` parsing and canonicalization debt
2. finish `M2` replay, rebuild, and narrow write-path closure
3. delay `M3+` reader, wire, and app-layer expansion until the shared core is stable

## Milestone Timeline

```mermaid
flowchart LR
  subgraph Minimal
    M1["M1<br/>Core Object and Validation Base<br/>Mostly complete"]
    M2["M2<br/>Replay, Storage, and Rebuild<br/>Substantially underway"]
  end

  subgraph ReaderPlusGovernance["Reader-plus-governance"]
    M3["M3<br/>Reader and Governance Surface<br/>Planned next"]
  end

  subgraph FullStack["Full-stack"]
    M4["M4<br/>Wire Sync and Peer Interop<br/>Later"]
    M5["M5<br/>Selective App-Layer Expansion<br/>Later"]
  end

  M1 --> M2 --> M3 --> M4 --> M5
```

## Milestone Snapshot

| Milestone | Status | Main focus now | Main gaps |
|---|---|---|---|
| `M1` | Mostly complete | shared parsing, canonical helpers, verification coverage | full object-family coverage, shared canonical utility, stronger `mycel-core` tests |
| `M2` | Substantially underway | replay, `state_hash`, store rebuild, persisted indexes | narrow authoring/write path, broader reader reuse, stronger replay/store fixtures |
| `M3` | Planned next | accepted-head reader workflows, profile-locked reading, text inspection | depends on stable `M1/M2` shared core |
| `M4` | Later | wire envelope, sync workflow, peer interop | depends on stable reader and store model |
| `M5` | Later | selective app-layer growth | depends on stable protocol core and sync |

## Implementation Matrix

Legend:

- `Done`: current checklist section is substantially closed for the minimal client
- `Mostly done`: only closure or follow-up work remains
- `Partial`: meaningful implementation exists, but the section is not closeable yet
- `Not started`: still mostly future work

| Area | Status | Primary milestone | Current read |
|---|---|---|---|
| 1. Repo and Build Setup | Mostly done | `M1` | only the shared canonical JSON utility remains open |
| 2. Object Types and IDs | Partial | `M1` | typed parsing exists for several families, but `document`, `block`, `snapshot`, and stricter field handling remain |
| 3. Canonical Serialization and Hashing | Partial | `M1` | core rules exist, but duplicate-key rejection and shared reuse are still open |
| 4. Signature Verification | Partial | `M1` | object signature rules are mostly present; wire-envelope checks are not |
| 5. Patch and Revision Engine | Mostly done | `M2` | replay and `state_hash` are in place; patch-op base is strong |
| 6. Local State and Storage | Mostly done | `M2` | store ingest, rebuild, and indexes exist; local transport/safety separation remains |
| 7. Wire Protocol | Not started | `M4` | canonical wire envelope and message validation are still future work |
| 8. Sync Workflow | Not started | `M4` | first-time and incremental sync remain future work |
| 9. Views and Head Selection | Mostly done | `M3` | deterministic selector core exists; multi-profile and dual-role closure remain |
| 10. Merge Generation | Partial | `M2` | verification is replay-based, but local merge authoring is not built |
| 11. CLI or API Surface | Partial | `M2` / `M3` | verification and inspection exist; authoring, sync, and workflow split remain |
| 12. Interop Test Minimum | Partial | `M1` / `M2` | fixtures and smoke coverage exist, but several normative and round-trip checks remain |
| 13. Ready-to-Build Gate | Partial | whole plan | replay, head selection, and rebuild are green; parse, wire sync, and merge generation are not |

## Suggested Reading Path

1. Read [`ROADMAP.md`](../ROADMAP.md) for build order and milestone intent.
2. Read [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) for section-by-section closure items.
3. Use [`progress.html`](./progress.html) for the public visual summary.
