# Simulator Scaffold

This directory is the language-neutral scaffold for the Mycel peer simulator.

It exists to separate simulator structure from implementation choice.

## Layout

- `peers/`: peer role definitions and peer-local configuration shape
- `topologies/`: named peer graph and bootstrap examples
- `tests/`: simulator test cases and expected assertions
- `reports/`: machine-readable output shape and report conventions
- `runtime/`: ignored local runtime state for manual experiments

## Build Direction

Recommended sequence:

1. implement one single-process multi-peer harness
2. bind it to fixtures in `../fixtures/`
3. add deterministic report output
4. later reuse the same peer logic in a localhost multi-process harness

This scaffold does not commit us to a language yet.
