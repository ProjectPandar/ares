# Consume Raft Layers Support Material Activation Design

## Upstream Boundary

This slice ports the support-material state predicate around OrcaSlicer's
existing `raft_layers` option. It does not generate real raft layers.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:943` declares `raft_layers` in
  `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5028-5037` defines
  `raft_layers` as an integer support option with default
  `INITIAL_RAFT_LAYERS`, minimum `0`, and maximum `100`.
- `OrcaSlicer/src/libslic3r/Print.hpp:429-431` defines support material state
  as `has_support() || has_raft()`, with `has_raft()` true when
  `raft_layers > 0`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:116-124` copies
  `object_config.raft_layers` into slicing parameters and includes positive
  base raft layers in the support layer-height bound gate.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:194-218` performs real raft layer
  planning; that adjacent behavior remains deferred.

## Current Ares State

`ares-core` already registers `raft_layers` in
`options/registry/definitions/table/tail_raft.rs` with Orca source citations
and default `0`. `options/brim.rs` also reads `raft_layers` to suppress the
current EFC outline brim gate when a raft is present, but it only requires a
non-negative integer.

The current final support proxy filter in `print_paths/generate.rs` preserves
support proxy artifacts when `enable_support` is true or
`enforce_support_layers > 0`. Orca's broader support-material state also
includes `raft_layers > 0`, so Ares still drops current support proxy paths for
raft-only support-material state.

## Design

Add a focused `raft_layers` runtime parser for the existing option. The parser
must apply Orca's integer range, defaulting to `0` and accepting numeric strings
for consistency with nearby option parsers:

- omitted value: `0`
- integer number or integer string in `0..=100`: accepted
- negative, fractional, non-integer, non-numeric, and values above `100`:
  `SliceError::InvalidInput`

Reuse this parser in both existing Ares consumers:

- `brim` EFC outline gating must continue to suppress the EFC outline when
  `raft_layers > 0`, now using the Orca `0..=100` range.
- `finalize_print_paths` must preserve current support proxy artifacts when
  `raft_layers > 0`, even when `enable_support` is absent/false and
  `enforce_support_layers` is absent/zero.

Keep the final filter location in `finalize_print_paths`: after support option
validation, support interface transforms, ordinary ironing, and support ironing.

The filtered support proxy role set remains:

- `PrintPathRole::SupportMaterial`
- `PrintPathRole::SupportMaterialInterface`
- `PrintPathRole::Ironing` whose `extrusion_role()` is
  `Some(PrintPathRole::SupportMaterialInterface)`

All downstream moves, extrusions, speeds, diagnostics, and G-code continue to
follow from the final path list.

## Included Behavior

- Positive `raft_layers` preserves current Ares `SupportMaterial` and
  `SupportMaterialInterface` proxy paths when `enable_support` is absent or
  false and `enforce_support_layers` is absent or zero.
- Positive `raft_layers` preserves support-interface ironing proxy paths when
  `support_ironing` is true.
- Omitted `raft_layers`, explicit `0`, and explicit `"0"` keep the disabled
  support filtering behavior when no other support-material state is active.
- Invalid `raft_layers` values fail before support proxy filtering.
- The existing brim EFC outline raft gate keeps its behavior for valid values
  and rejects out-of-range raft layer counts through the shared parser.
- Existing `enable_support = true` and `enforce_support_layers > 0` behavior
  remains unchanged.
- Ordinary non-support paths and ordinary ironing remain unaffected.

## Deferred Behavior

- Generating real raft layers or raft extrusion paths.
- Raft contact/base/interface layer planning from `Slicing.cpp:194-218`.
- Consuming other raft options such as `raft_contact_distance`,
  `raft_expansion`, `raft_first_layer_density`, or
  `raft_first_layer_expansion`.
- Per-object `has_raft()` / `has_support_material()` data modeling.
- Support layer synchronization, support blockers/enforcers, support material
  generation, tree/organic support, and raft/support interactions beyond the
  current proxy artifacts.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. A disabled or omitted `enable_support` plus positive `raft_layers` preserves
   `SupportMaterial` proxy print paths and their downstream toolpath,
   extrusion, speed, diagnostics, and G-code artifacts.
2. A disabled or omitted `enable_support` plus positive `raft_layers` preserves
   `SupportMaterialInterface` proxy print paths and their downstream artifacts.
3. A disabled `enable_support` plus positive `raft_layers`,
   `support_ironing = true`, and a closed support-interface rectangle preserves
   both the interface path and the support-interface ironing duplicate.
4. A disabled or omitted `enable_support` plus omitted, numeric zero, or string
   zero `raft_layers` still removes support proxy artifacts when
   `enforce_support_layers` is also zero or omitted.
5. Invalid `raft_layers` values return `SliceError::InvalidInput` before
   filtering and include `raft_layers` in the error text.
6. `brim_options()` rejects out-of-range `raft_layers` through the shared parser
   and keeps the existing "no EFC outline while raft is present" behavior for
   valid positive values.
7. Existing `enable_support = true` and `enforce_support_layers > 0` proxy
   behavior remains unchanged.
8. `docs/roadmap.md` records that `raft_layers > 0` now participates in Ares'
   current support proxy activation, while real raft generation remains
   deferred.

## Verification Plan

- Add RED tests under `crates/ares-core/src/pipeline/tests/support_enable.rs`
  for positive `raft_layers` preserving support material, support interface,
  and support-interface ironing proxy paths without `enable_support`.
- Add tests for omitted, numeric zero, and string zero `raft_layers` preserving
  the disabled-support filter when no other support-material state is active.
- Add invalid `raft_layers` coverage for the final support proxy path.
- Extend `crates/ares-core/src/options/tests/brim_runtime.rs` so
  `brim_options()` rejects `raft_layers` above Orca's `100` maximum while
  preserving the existing valid positive raft gate.
- Run targeted tests:
  - `cargo nextest run -p ares-core support_enable`
  - `cargo nextest run -p ares-core brim_runtime`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines.
