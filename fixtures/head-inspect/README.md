# Head Inspect Fixtures

This directory holds local input bundles for the minimal `mycel head inspect` CLI surface.

Recommended contents:

- `head-inspect.schema.json`: formal schema for one local input bundle
- `<fixture-name>/bundle.json`: repo-native fixture directory layout for smoke tests and manual inspection
- optional flat `*.json` / `*.example.json` bundles for one-off inputs

Current example bundles:

- `minimal-head-selection/bundle.json`: selects one accepted head from two eligible revisions using three signed View objects
- `viewer-score-channels/bundle.json`: exercises bounded viewer bonus/penalty, anti-Sybil gating, expiry, and challenge review/freeze pressure without turning selector choice into raw popularity counting

Optional bundle fields:

- `objects`: additional verified replay objects, such as `patch` objects, that `mycel head render` can use when rendering without `--store-root`
- `profiles`: explicit fixed reader profiles keyed by profile id; when a bundle declares more than one, `mycel head inspect` and `mycel head render` require `--profile-id`
- `viewer_signals`: local viewer-side score and escalation inputs for bounded viewer-in-selector fixture coverage
