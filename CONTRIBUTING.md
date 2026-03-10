# Contributing to Mycel

Thanks for contributing to Mycel.

This repository is still in an early spec-first stage. Most changes are expected to help with one of these areas:

- protocol/spec clarification
- design-note refinement
- fixture and simulator validation coverage
- Rust workspace implementation toward the first client

## Before You Contribute

- Read [README.md](./README.md) for the current project scope.
- Read [AGENTS.md](./AGENTS.md) for the active repository working rules.
- If you are contributing through an AI coding bot or automation workflow, read [BOT-CONTRIBUTING.md](./BOT-CONTRIBUTING.md) too.
- If you are setting up bot-facing GitHub workflow, sync the tracked labels first with [`scripts/sync-labels.sh`](./scripts/sync-labels.sh).
- Use [`scripts/check-labels.sh`](./scripts/check-labels.sh) to verify the tracked labels still match GitHub after host-side changes.
- Prefer narrow, explicit changes over broad cleanup.
- Keep protocol-core changes conservative unless the change is clearly justified.

## Change Expectations

- If you change behavior, update the relevant tests or validation coverage.
- If you change a protocol, profile, or design-note concept, update the relevant Markdown docs in both English and Traditional Chinese when both versions exist.
- Keep implementation aligned with current documented scope; do not assume backward compatibility unless it is explicitly required.
- Prefer small, reviewable commits over large mixed changes.

## Pull Request Guidance

- Explain what changed.
- Explain why the change is needed now.
- Call out any protocol, schema, fixture, or CLI contract impact.
- Note any follow-up work that should happen next.

## License Expectations

Unless explicitly stated otherwise in a future policy, contributions submitted to this repository are expected to be provided under the same [MIT License](./LICENSE) terms as the rest of the repository.

## Questions and Ambiguity

If a change touches protocol boundaries, governance rules, custody logic, or signer/security assumptions, prefer opening the design question explicitly instead of silently choosing one interpretation in code.
