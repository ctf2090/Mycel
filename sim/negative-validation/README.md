# Negative Validation

This directory collects intentionally invalid simulator artifacts and the commands we use to prove that validation fails for the right reason.

It exists to keep negative validator examples organized without polluting normal repo-wide validation.

The actual invalid artifacts may still live elsewhere when that keeps validator behavior simpler.
This directory is the index and matrix for those cases.

## Current Scope

- report metadata self-contradiction
- seed-source and deterministic-seed mismatch

## Current Examples

- `random-seed-prefix-mismatch`
  Artifact: `sim/reports/invalid/random-seed-prefix-mismatch.example.json`
  Expected result: `mycel validate` fails with a `seed_source` / `deterministic_seed` prefix error
- `auto-seed-prefix-mismatch`
  Artifact: `sim/reports/invalid/auto-seed-prefix-mismatch.example.json`
  Expected result: `mycel validate` fails with a `seed_source` / `deterministic_seed` prefix error

## Command

```bash
cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/auto-seed-prefix-mismatch.example.json --json
```

## Smoke Script

Run both the positive and negative validation path in one command:

```bash
./sim/negative-validation/smoke.sh
```

The script expects:

- repo-wide `mycel validate --json` returns `status: "ok"`
- the intentional invalid `random` and `auto` reports return `status: "failed"`
- each failure message mentions the matching `seed_source` mismatch

## Why This Directory Exists

- keep negative validation examples discoverable
- avoid mixing invalid artifacts into normal runnable simulator examples
- make it easier to grow a small failure matrix over time
