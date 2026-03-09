# Simulator Tests

This directory describes test categories for the simulator harness.

Recommended first categories:

- positive sync
- negative validation
- recovery
- deterministic comparison

Each test case should define:

- topology
- fixture set
- execution mode
- expected pass or fail conditions

## Schema

- `test-case.schema.json` is the formal contract for JSON test definitions in this directory.
- `three-peer-consistency.example.json` is the first example bound to that schema.
- The schema keeps test intent explicit without forcing a specific test runner.

The harness implementation may later represent these as JSON, YAML, TOML, or native test code.
