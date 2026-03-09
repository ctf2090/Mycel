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

The harness implementation may later represent these as JSON, YAML, TOML, or native test code.
