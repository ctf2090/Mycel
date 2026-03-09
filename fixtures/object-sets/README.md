# Object Sets

This directory contains named fixture sets for simulator and verification tests.

Each subdirectory should represent one scenario.

Recommended contents for each scenario:

- `README.md`: scenario purpose and expected outcomes
- `fixture.json`: language-neutral metadata about peers, documents, and expected results
- optional object files if we later decide to store canonical examples separately

## Current Scenarios

- `minimal-valid/`
- `hash-mismatch/`
- `signature-mismatch/`
- `partial-want-recovery/`
