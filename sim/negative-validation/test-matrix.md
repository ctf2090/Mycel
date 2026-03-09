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

## Planned Cases

- `unknown-topology-reference`
  Target type: test-case or report
  Expected status: `failed`
  Planned failure: `topology_id` or `source_topology` cannot be resolved

## Usage

Run one case directly:

```bash
cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/auto-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/missing-seed-source.example.json --json --strict
```

Run the whole repo and confirm the invalid artifacts are not part of normal validation:

```bash
cargo run -p mycel-cli -- validate --json
```

Run the smoke check for both paths together:

```bash
./sim/negative-validation/smoke.sh
```

The smoke script currently covers:

- repo-wide success
- `random-seed-prefix-mismatch` hard failure
- `auto-seed-prefix-mismatch` hard failure
- `missing-seed-source` warning in normal mode
- `missing-seed-source` non-zero exit under `--strict`
