# Raft support fill rotation precision plan

Source owner: `FillSupportBase::fill_surface`, output rotation at
`FillRectilinear.cpp:3627-3633`. Destination and deferred behavior are
specified in [the spec](../specs/2026-09-29-raft-rotation-precision.md).

1. Reproduce the max-raft G-code mismatch and inspect exact endpoint values.
2. Add an output-geometry regression using independently observed Orca
   coordinates; verify red before changing rotation.
3. Match upstream float trig promotion and remove diagnostic probes.
4. Run raft option E2E comparisons and required Rust checks.
5. Independent read-only review, fix/re-review as needed, then commit and
   push code and documentation together.

## Verification

- Geometry regression red: `/tmp/ares-raft-geometry-red.log`.
- Real-Orca max-raft smoke red: `/tmp/ares-raft-precision-red.log`;
  three independent oracle attempts retain the same E difference.
- Green live sweep: all 12 generated cases across `raft_layers`,
  `raft_contact_distance`, `raft_expansion`, `raft_first_layer_density`,
  and `raft_first_layer_expansion` match complete Orca output. The new
  max-raft smoke also passes. Full outputs and manifests are retained in
  `/tmp/ares-raft-precision-green`, log at
  `/tmp/ares-raft-precision-green.log`.
- The tracked coverage summary updates only the verified raft row:
  611/650 domains complete. Other rows retain their historical evidence;
  this is not a new all-option or all-printer sweep.
- `cargo fmt --check` and `git diff --check` passed.
- `cargo clippy` passed with 227 pre-existing core warnings.
- `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`
  passed. This is compilation evidence, not browser runtime parity.
- Core checks for `x86_64-pc-windows-gnu` and `x86_64-apple-darwin`
  passed. These are compilation checks; live Orca E2E ran on Linux.
- `cargo nextest run --workspace`: 7355 passed, 9 skipped.
  Log: `/tmp/ares-raft-precision-workspace.log`. The original uncommitted
  geometry refactor was present during validation and remains outside this
  commit.
- Independent review found no code repair items; final review checks the
  recorded outputs and test results.
