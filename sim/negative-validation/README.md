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
- `missing-seed-source`
  Artifact: `sim/reports/invalid/missing-seed-source.example.json`
  Expected result: `mycel validate` returns `status: "warning"` because metadata omits `seed_source`
  Strict mode: `mycel validate --strict` returns a non-zero exit code for the same file
- `unknown-topology-reference`
  Artifact: `sim/reports/invalid/unknown-topology-reference.example.json`
  Expected result: `mycel validate` fails because `topology_id` does not match any loaded topology
- `unknown-fixture-reference`
  Artifact: `sim/reports/invalid/unknown-fixture-reference.example.json`
  Expected result: `mycel validate` fails because `fixture_id` does not match any loaded fixture

## Command

```bash
cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/auto-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json --strict
cargo run -p mycel-cli -- validate sim/reports/invalid/unknown-topology-reference.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/unknown-fixture-reference.example.json --json
```

## Smoke Script

Run both the positive and negative validation path in one command:

```bash
./sim/negative-validation/smoke.sh
./sim/negative-validation/smoke.sh --summary-only
```

The script now ends with a short per-case summary so we can confirm the outcome without re-reading every JSON block.
Use `--summary-only` when we want compact CI-oriented output.

The script expects:

- repo-wide `mycel validate --json` returns `status: "ok"`
- the intentional invalid `random` and `auto` reports return `status: "failed"`
- each failure message mentions the matching `seed_source` mismatch
- the `unknown-topology-reference` report returns `status: "failed"` with a topology lookup error
- the `unknown-fixture-reference` report returns `status: "failed"` with a fixture lookup error
- the `missing-seed-source` report returns `status: "warning"` by default
- the same `missing-seed-source` report returns a non-zero exit code under `--strict`

## Why This Directory Exists

- keep negative validation examples discoverable
- avoid mixing invalid artifacts into normal runnable simulator examples
- make it easier to grow a small failure matrix over time
