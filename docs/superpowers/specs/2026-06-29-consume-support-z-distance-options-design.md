# Consume Support Z Distance Options Design

## Goal

Consume the registered Orca support Z-distance option state before adding more options:

- `support_top_z_distance`
- `support_bottom_z_distance`
- `enforce_support_layers`

This slice adds typed runtime parsing, validation, and derived zero-gap support-interface state for the current Ares support finalization shell. It does not add support contact-layer topology.

## Upstream Boundary

Source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:956-958`: `support_top_z_distance`, `support_bottom_z_distance`, and `enforce_support_layers` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6000`: `support_top_z_distance` definition, float minimum `0`, default `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6002-6011`: `support_bottom_z_distance` definition, float minimum `0`, default `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6013-6025`: `enforce_support_layers` definition, integer range `0..=5000`, default `0`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:81-120`: `SlicingParameters` derives `zero_topZ_contact`, `zero_gap_interface_top`, and `zero_gap_interface_bottom` from top/bottom Z gaps plus resolved top/bottom interface layer counts.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1160-1195`: changing these options invalidates support material.

Rust destination boundary:

- Add `crates/ares-core/src/options/support_z_distance.rs` as the typed runtime parser and state object.
- Add `SliceOptions::support_z_distance_options()` in `crates/ares-core/src/options.rs`.
- Call the parser from `crates/ares-core/src/print_paths/generate.rs::finalize_print_paths()` before existing support-interface finalization.
- Keep derived state crate-internal until a source-cited support generator owns contact-layer topology.

## Existing Ares Context

Ares already registers these options and validates registry lookup metadata in `crates/ares-core/src/options/tests/registry_lookup_support_z_distance_enforce.rs`.

Current print-path support behavior is a generic shell:

- `support_interface_top_layers` and `support_interface_bottom_layers` can reclassify existing generic `SupportMaterialInterface` paths.
- support spacing, support pattern, support expansion, support angle, and support ironing operate only on existing rectangular support path shells.
- Ares does not yet generate Orca top-contact layers, bottom-contact layers, raft contact layers, object/support Z-contact relationships, or support invalidation graphs.

## Required Behavior

Parsing:

- `support_top_z_distance` accepts JSON numbers and numeric strings.
- `support_bottom_z_distance` accepts JSON numbers and numeric strings.
- Both Z distances default to `0.2` mm.
- Both Z distances must be finite and `>= 0.0`.
- `enforce_support_layers` accepts JSON integer numbers and integer strings.
- `enforce_support_layers` defaults to `0`.
- `enforce_support_layers` must be in `0..=5000`.
- JSON float-form numbers such as `5.0` are not JSON integer numbers for this parser and must be rejected for `enforce_support_layers`.
- Reject non-numeric, non-finite, negative Z-distance, fractional integer, signed-negative integer, boolean, null, array, object, and out-of-range values with `SliceError::InvalidInput` mentioning the offending key.
- The finite/range checks are Ares input-boundary validation of Orca option-definition limits, not a claim that upstream performs identical parse-time validation.

State:

- Expose crate-internal getters:
  - `top_z_distance_mm()`
  - `bottom_z_distance_mm()`
  - `enforce_support_layers()`
- Expose crate-internal zero-gap helpers matching the cited Orca `SlicingParameters` expressions:
  - `zero_top_contact()` is true when top Z distance is `0.0`.
  - `zero_gap_interface_top(top_layers)` is true when top interface layers are positive and top Z distance is `0.0`.
  - `zero_gap_interface_bottom(top_layers, bottom_layers)` resolves `bottom_layers < 0` to `top_layers`, then returns true when the resolved bottom count is positive and either bottom Z distance or top Z distance is `0.0`.

Pipeline integration:

- `finalize_print_paths()` must parse `support_z_distance_options()` before support-interface rewriting so invalid values fail through the active slicing path.
- Existing generic support-interface path geometry and role behavior must remain unchanged.

## Deferred Behavior

- No new option registry keys.
- No new public CLI, WASM, or core API surface.
- No support material invalidation implementation.
- No support contact-layer topology, raft contact behavior, object/support Z-contact analysis, bridge/contact flow changes, or organic/classic tree support geometry.
- No geometry changes based on `enforce_support_layers`; parsed state only until the Orca support generator slice exists.
- No new dependencies.

## Tests

Add focused option tests:

- Defaults match Orca values.
- Numeric JSON and numeric string values parse for top/bottom Z distances.
- `enforce_support_layers` parses integer JSON numbers and integer strings.
- Boundary values are accepted: Z distances at `0.0`, enforce layers at `0` and `5000`.
- Derived zero-gap helpers match the Orca expressions for top layers, explicit bottom layers, and `bottom = -1`.
- Invalid values reject with errors mentioning the relevant key.

Add focused print-path finalization tests:

- Invalid `support_top_z_distance` fails through `finalize_print_paths()`.
- Invalid `support_bottom_z_distance` fails through `finalize_print_paths()`.
- Invalid `enforce_support_layers` fails through `finalize_print_paths()`.
- Valid zero-gap settings do not change existing generic support-interface role behavior in this slice.

Run the existing registry lookup test for these options to confirm definition metadata still matches the consumed runtime keys.

## Documentation

Update `docs/roadmap.md` with a concise source-cited runtime status entry for the three consumed support Z-distance/enforce options. The entry must map Ares `zero_top_contact()` to upstream `zero_topZ_contact`, state that finite/range checks are Ares input-boundary validation of Orca option-definition limits, and say full support contact-layer topology, support invalidation, and enforced-support region generation remain deferred.

## Acceptance Criteria

- `support_top_z_distance`, `support_bottom_z_distance`, and `enforce_support_layers` are parsed into a typed crate-internal runtime state.
- Invalid values fail through the current slicing finalization path.
- Existing support path output remains unchanged for valid values.
- Existing registry lookup tests for the three keys still pass.
- `cargo fmt --check`, `git diff --check`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted nextest, and `cargo nextest run --workspace` pass before commit.
- All changed Rust source files remain at or below 400 LOC or are split.
