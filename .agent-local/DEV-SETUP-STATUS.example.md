# Dev Setup Status

- Status: ready
- Checked at: 2026-03-12 15:00 UTC+8
- Checked by: doc-<n>
- Workspace: /workspaces/Mycel
- Evidence source:
  - `scripts/check-dev-env.sh --json`
  - `scripts/check-dev-env.sh --full --json`
- Notes:
  - Update this file whenever tool availability changes or the workspace is reprovisioned.
  - New chats may skip bootstrap dev-setup checks only when this file says `Status: ready`.

## Tool Checks

| Item | Status | Detail |
|---|---|---|
| `cargo` | passed | `cargo --version` |
| `rustup` | passed | `rustup --version` |
| `rustc` | passed | `rustc --version` |
| `gh` | passed | `gh --version` |
| `rg` | passed | `rg --version` |

## Rust Components

| Item | Status | Detail |
|---|---|---|
| `rustfmt` | passed | `rustup component list --toolchain stable --installed` |
| `clippy` | passed | `rustup component list --toolchain stable --installed` |

## Repo Validation

- Full validation run: yes

| Check | Status | Command |
|---|---|---|
| format | passed | `cargo fmt --all --check` |
| core tests | passed | `cargo test -p mycel-core` |
| CLI tests | passed | `cargo test -p mycel-cli` |
| CLI info | passed | `cargo run -p mycel-cli -- info` |
| fixture validate | passed | `cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json` |
| sim smoke | passed | `./sim/negative-validation/smoke.sh --summary-only` |
