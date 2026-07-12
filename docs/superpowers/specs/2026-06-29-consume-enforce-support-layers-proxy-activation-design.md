# Consume Enforce Support Layers Proxy Activation Design

## Upstream Boundary

This slice ports the support-activation predicate around OrcaSlicer's existing
`enforce_support_layers` option, not real enforced support generation.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948-958` declares
  `enable_support` and `enforce_support_layers` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6013-6025` defines
  `enforce_support_layers` as an integer support option with default `0`,
  minimum `0`, and maximum `5000`.
- `OrcaSlicer/src/libslic3r/Print.hpp:429-431` defines object support state as
  `enable_support || enforce_support_layers > 0`, with support material also
  enabled by raft.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:124-132` treats
  `enable_support`, raft layers, or positive `enforce_support_layers` as
  support-bearing state for support layer-height bounds.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.hpp:28` checks
  `enable_support.value || enforce_support_layers` before support material
  generation details.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10228-10233` rejects both
  `enable_support` and positive `enforce_support_layers` in spiral-vase CLI
  validation, confirming that positive enforced layers are support state.

## Current Ares State

`ares-core` already parses `enforce_support_layers` through
`options/support_z_distance.rs` with Orca's `0..=5000` integer range and
default `0`. `finalize_print_paths` already validates those support z-distance
options before support proxy transforms.

The previous `enable_support` slice added a final support proxy filter in
`print_paths/generate.rs`. That filter currently keeps current Ares support
proxy paths only when `enable_support` is true. This is narrower than Orca's
support predicate because positive `enforce_support_layers` counts as
support-enabled state.

## Design

Keep the final filter location in `finalize_print_paths`: it must remain after
support validation, support interface transforms, ordinary ironing, and support
ironing. Change only the boolean passed into that filter.

The filter must preserve support proxy paths when:

- `enable_support` is true, or
- parsed `enforce_support_layers` is greater than zero.

It must remove support proxy paths when both are false or zero/omitted.

The filtered path set remains exactly the current support proxy role set:

- `PrintPathRole::SupportMaterial`
- `PrintPathRole::SupportMaterialInterface`
- `PrintPathRole::Ironing` whose `extrusion_role()` is
  `Some(PrintPathRole::SupportMaterialInterface)`

All downstream moves, extrusions, speeds, diagnostics, and G-code continue to
follow from the final path list.

## Included Behavior

- Positive `enforce_support_layers` preserves current Ares `SupportMaterial`
  and `SupportMaterialInterface` proxy paths even when `enable_support` is
  absent or false.
- Positive `enforce_support_layers` preserves support-interface ironing proxy
  paths when `support_ironing` is true.
- Omitted `enforce_support_layers`, explicit `0`, and explicit `"0"` keep the
  disabled-support filtering behavior when `enable_support` is false or
  omitted.
- Invalid `enforce_support_layers` values still fail before support proxy
  filtering.
- `enable_support = true` behavior remains unchanged.
- Ordinary non-support paths and ordinary ironing remain unaffected.

## Deferred Behavior

- Generating real enforced support layers for the first N layers.
- Changing support threshold, support blockers/enforcers, or overhang
  detection.
- Raft-driven support material activation through `raft_layers`.
- Per-object support state and `PrintObject::has_support()` data modeling.
- Tree/organic support generation and branch behavior.
- Orca support material generation, support layer synchronization, and exact
  support layer-height bounds.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. A disabled or omitted `enable_support` plus positive
   `enforce_support_layers` preserves `SupportMaterial` proxy print paths and
   their downstream toolpath, extrusion, speed, diagnostics, and G-code
   artifacts.
2. A disabled or omitted `enable_support` plus positive
   `enforce_support_layers` preserves `SupportMaterialInterface` proxy print
   paths and their downstream artifacts.
3. A disabled `enable_support` plus positive `enforce_support_layers`,
   `support_ironing = true`, and a closed support-interface rectangle preserves
   both the interface path and the support-interface ironing duplicate.
4. A disabled or omitted `enable_support` plus omitted, numeric zero, or string
   zero `enforce_support_layers` still removes support proxy artifacts.
5. Invalid `enforce_support_layers` values still return
   `SliceError::InvalidInput` before filtering.
6. Existing valid `enable_support = true` proxy behavior remains unchanged.
7. `docs/roadmap.md` records that `enforce_support_layers > 0` now participates
   in Ares' current support proxy activation, while real enforced support
   generation remains deferred.

## Verification Plan

- Add RED tests under `crates/ares-core/src/pipeline/tests/support_enable.rs`
  for positive `enforce_support_layers` preserving support material, support
  interface, and support-interface ironing proxy paths without
  `enable_support`.
- Add tests for omitted, numeric zero, and string zero
  `enforce_support_layers` preserving the disabled-support filter.
- Run targeted tests:
  - `cargo nextest run -p ares-core support_enable`
  - `cargo nextest run -p ares-core support_z_distance`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines.
