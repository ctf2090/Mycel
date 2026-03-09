# Head Inspect Fixtures

This directory holds local input bundles for the minimal `mycel head inspect` CLI surface.

Recommended contents:

- `head-inspect.schema.json`: formal schema for one local input bundle
- `*.example.json`: version-controlled example bundles for smoke tests and manual inspection

Current example bundles:

- `minimal-head-selection.example.json`: selects one accepted head from two eligible revisions using three signed View objects
