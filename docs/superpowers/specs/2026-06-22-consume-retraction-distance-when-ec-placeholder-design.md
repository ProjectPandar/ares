# Consume EC Retraction Distance Scalar Placeholder Design

## Goal

Port OrcaSlicer's scalar `[retraction_distance_when_ec]` machine-start placeholder into Ares G-code rendering so the existing nullable `retraction_distances_when_ec` option also drives the first active extruder scalar placeholder.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2826` registers `[retraction_distance_when_ec]` for machine-start G-code from `m_config.retraction_distances_when_ec.get_at(initial_extruder_id)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2833` registers `[retraction_distances_when_ec]` as `new ConfigOptionFloatsNullable(m_config.retraction_distances_when_ec)`, already ported in Ares as the vector placeholder.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1057,7649,7940` also update `[retraction_distance_when_ec]` for filament-change scopes, but this slice does not implement filament-change placeholder processing.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5102-5109` defines `retraction_distances_when_ec` as nullable floats with min `0`, max `10`, sidetext `mm`, and default `ConfigOptionFloatsNullable {10}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1373` stores the field as `ConfigOptionFloatsNullable retraction_distances_when_ec`.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-952` defines nullable float vectors where `nil` is represented as NaN and serializes to `nil` only through the nullable vector serializer.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.hpp:50` wraps scalar double placeholders with `ConfigOptionFloat`; `PlaceholderParser.cpp` expression parsing rejects NaN tokens, so this slice treats scalar nil as invalid rather than inventing a numeric fallback.

## Current Ares Context

- `crates/ares-core/src/options/layer_change_retraction.rs` already parses `retraction_distances_when_ec` into `Vec<Option<f64>>`, defaulting to `[Some(10.0)]` and validating finite `0..=10` numeric values while preserving `None` for vector `nil`.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already renders `[retraction_distances_when_ec]` as a nullable vector.
- Existing cut-scalar tests in `crates/ares-core/src/tests/retraction_distance_when_cut_placeholder_gcode.rs` define the adjacent scalar numeric placeholder pattern.
- Existing EC-vector tests in `crates/ares-core/src/tests/retraction_distances_when_ec_vector_placeholder_gcode.rs` define the nullable EC input forms and layer-change non-scope behavior.

## Included Behavior

1. Add `SliceOptions::retraction_distance_when_ec()` returning the first numeric value from `retraction_distances_when_ec`.
2. Missing `retraction_distances_when_ec` uses Orca's default vector `[10]`, rendered as scalar `10`.
3. Supported input comes from the existing `retraction_distances_when_ec` parser:
   - scalar number and first array element render with existing numeric placeholder formatting;
   - comma-separated string tokens are accepted through the existing nullable-number parser;
   - first value `nil` or JSON `null` returns `SliceError::InvalidInput` mentioning `retraction_distances_when_ec` only when `[retraction_distance_when_ec]` is present in `machine_start_gcode`.
4. Machine-start G-code replaces `[retraction_distance_when_ec]` with the scalar numeric value.
5. `[retraction_distance_when_ec]` remains literal in `layer_change_gcode`; this slice only ports the `GCode.cpp:2826` machine-start scalar placeholder boundary.
6. Existing `[retraction_distances_when_ec]` vector-only templates continue to render vector `nil` as `nil` without scalar rejection.
7. Invalid `retraction_distances_when_ec` input continues to return `SliceError::InvalidInput` mentioning `retraction_distances_when_ec`.

## Deferred Behavior

- Do not implement filament-change placeholder updates from `GCode.cpp:1057,7649,7940`.
- Do not change `[retraction_distances_when_ec]`, `[long_retraction_when_ec]`, cut placeholders, option metadata, or layer-change placeholder scope.
- Do not add crates, dependencies, file I/O, UI, terminal behavior, OpenGL behavior, or independent Ares pipeline concepts.
- Do not attempt full Orca placeholder parser NaN rendering or expression parity in this slice.

## Rust Destination Boundary

- Modify `crates/ares-core/src/options/layer_change_retraction.rs` to expose the scalar EC retraction-distance accessor.
- Modify `crates/ares-core/src/gcode_machine_start_placeholders.rs` to format and replace `[retraction_distance_when_ec]`.
- Add focused G-code tests in `crates/ares-core/src/tests/retraction_distance_when_ec_placeholder_gcode.rs` and register the module in `crates/ares-core/src/tests/mod.rs`.

## Acceptance Criteria

- `cargo nextest run -p ares-core retraction_distance_when_ec` initially fails before implementation and passes after implementation.
- Tests prove configured first value, default, scalar number, serialized string composition, scalar nil rejection only when scalar placeholder is present, vector-only nil preservation, layer-change literal scope, and invalid input behavior.
- Existing adjacent placeholder tests still pass with `cargo nextest run -p ares-core retraction_distance_when_cut retraction_distances_when_ec retraction_distance_when_ec`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC <= 400.

## Safety And Rollback

The change is limited to in-memory option parsing and machine-start string substitution in `ares-core`. It has no external I/O, no dependency changes, and no persistent state. Rollback is reverting the accessor, replacement, tests, spec, and plan files from this slice.

## Spec Self-Review

- Placeholder scan: no unresolved placeholder markers.
- Scope check: one upstream scalar nullable float machine-start placeholder only.
- Ambiguity check: scalar nil behavior is explicit and tied to the upstream `ConfigOptionFloat`/NaN boundary.
- Consistency check: the scalar placeholder consumes the same `retraction_distances_when_ec` option that already drives the vector placeholder.
