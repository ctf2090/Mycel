# Fixtures

This directory holds version-controlled simulator fixtures.

The first goal is to keep fixtures:

- deterministic
- reviewable in git
- independent of any one implementation language
- reusable across simulator phases

## Layout

- `object-sets/`: canonical object collections and failure cases

## Rules

- Keep fixture metadata human-readable.
- Prefer small fixture sets over broad random generation.
- Treat each fixture set as a named scenario with one clear purpose.
- Do not store runtime caches or generated reports here.
