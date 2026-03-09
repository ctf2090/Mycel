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

## Planned Cases

- `missing-seed-source`
  Target type: report
  Expected status: `warning` or `failed` depending on validator strictness
  Planned failure: metadata omits `seed_source`

- `unknown-topology-reference`
  Target type: test-case or report
  Expected status: `failed`
  Planned failure: `topology_id` or `source_topology` cannot be resolved

## Usage

Run one case directly:

```bash
cargo run -p mycel-cli -- validate sim/reports/invalid/random-seed-prefix-mismatch.example.json --json
cargo run -p mycel-cli -- validate sim/reports/invalid/auto-seed-prefix-mismatch.example.json --json
```

Run the whole repo and confirm the invalid artifacts are not part of normal validation:

```bash
cargo run -p mycel-cli -- validate --json
```

Run the smoke check for both paths together:

```bash
./sim/negative-validation/smoke.sh
```
