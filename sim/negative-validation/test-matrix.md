# Negative Validation Matrix

## Active Cases

- `random-seed-prefix-mismatch`
  Artifact: `sim/reports/invalid/random-seed-prefix-mismatch.example.json`
  Target type: report
  Expected status: `failed`
  Expected failure: `seed_source = random` with a non-`random:` deterministic seed

- `auto-seed-prefix-mismatch`
  Artifact: `sim/reports/invalid/auto-seed-prefix-mismatch.example.json`
  Target type: report
  Expected status: `failed`
  Expected failure: `seed_source = auto` with a non-`auto:` deterministic seed

- `missing-seed-source`
  Artifact: `sim/reports/invalid/missing-seed-source.example.json`
  Target type: report
  Expected status: `warning`
  Strict mode exit code: non-zero with `--strict`
  Expected failure: metadata omits `seed_source`

- `unknown-topology-reference`
  Artifact: `sim/reports/invalid/unknown-topology-reference.example.json`
  Target type: report
  Expected status: `failed`
  Expected failure: `topology_id` does not match any loaded topology

- `unknown-fixture-reference`
  Artifact: `sim/reports/invalid/unknown-fixture-reference.example.json`
  Target type: report
  Expected status: `failed`
  Expected failure: `fixture_id` does not match any loaded fixture

## Planned Cases

## Usage

Run one case directly:

```bash
cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/auto-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json --strict
cargo run -p mycel-cli -- validate sim/reports/invalid/unknown-topology-reference.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/unknown-fixture-reference.example.json --json
```

Run the whole repo and confirm the invalid artifacts are not part of normal validation:

```bash
cargo run -p mycel-cli -- validate --json
```

Run the smoke check for both paths together:

```bash
./sim/negative-validation/smoke.py
./sim/negative-validation/smoke.py --summary-only
./sim/negative-validation/smoke.py --case unknown-fixture-reference
./sim/negative-validation/smoke.py --case missing-seed-source-strict --summary-only
```

The smoke script currently covers:

- repo-wide success
- `random-seed-prefix-mismatch` hard failure
- `auto-seed-prefix-mismatch` hard failure
- `unknown-topology-reference` hard failure
- `unknown-fixture-reference` hard failure
- `missing-seed-source` warning in normal mode
- `missing-seed-source` non-zero exit under `--strict`
- a final per-case summary for quick scanability
- optional `--summary-only` output for shorter logs
- optional `--case <name>` filtering for single-case debugging
