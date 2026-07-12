# Consume EC Long Retraction Scalar Placeholder Design

## Goal

Port OrcaSlicer's scalar `[long_retraction_when_ec]` machine-start placeholder into Ares G-code rendering so the existing nullable `long_retractions_when_ec` option also drives the first active extruder scalar placeholder.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2827` registers `[long_retraction_when_ec]` for machine-start G-code from `m_config.long_retractions_when_ec.get_at(initial_extruder_id)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2834` registers `[long_retractions_when_ec]` as `new ConfigOptionBoolsNullable(m_config.long_retractions_when_ec)`, already ported in Ares as the vector placeholder.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1058,7650,7941` also update `[long_retraction_when_ec]` for filament-change scopes, but this slice does not implement filament-change placeholder processing.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5096-5100` defines `long_retractions_when_ec` as nullable bools with default `ConfigOptionBoolsNullable {false}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1374` stores the field as `ConfigOptionBoolsNullable long_retractions_when_ec`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1886-1892` defines bool vector `get_at`: out-of-range indexes fall back to the first value, and nullable nil uses the max unsigned-char sentinel; scalar bool conversion treats that sentinel as true.

## Current Ares Context

- `crates/ares-core/src/options/layer_change_retraction.rs` already parses `long_retractions_when_ec` into `Vec<Option<bool>>` with Orca serialized nullable bool tokens.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already renders `[long_retractions_when_ec]` as a nullable vector.
- Existing cut-scalar tests in `crates/ares-core/src/tests/long_retraction_when_cut_placeholder_gcode.rs` define the adjacent scalar bool placeholder pattern.
- Existing EC-vector tests in `crates/ares-core/src/tests/long_retractions_when_ec_vector_placeholder_gcode.rs` define the nullable EC input forms and layer-change non-scope behavior.

## Included Behavior

1. Add `SliceOptions::long_retraction_when_ec()` returning the first value from `long_retractions_when_ec`.
2. Missing `long_retractions_when_ec` uses Orca's default vector `[false]`, rendered as scalar `0`.
3. Supported input comes from the existing `long_retractions_when_ec` parser:
   - scalar bool: `true` -> `1`, `false` -> `0`;
   - scalar null: `null` -> `1`, matching Orca nullable bool `get_at` sentinel truthiness;
   - nonempty nullable bool array: first element controls scalar output;
   - comma-separated string tokens `nil`, `1`, and `0`, with whitespace trimming.
4. Machine-start G-code replaces `[long_retraction_when_ec]` with `1` or `0`.
5. `[long_retraction_when_ec]` remains literal in `layer_change_gcode`; this slice only ports the `GCode.cpp:2827` machine-start scalar placeholder boundary.
6. Invalid `long_retractions_when_ec` input continues to return `SliceError::InvalidInput` mentioning `long_retractions_when_ec`.

## Deferred Behavior

- Do not implement filament-change placeholder updates from `GCode.cpp:1058,7650,7941`.
- Do not change `[long_retractions_when_ec]`, `[retraction_distance_when_ec]`, `[retraction_distances_when_ec]`, cut placeholders, option metadata, or layer-change placeholder scope.
- Do not add crates, dependencies, file I/O, UI, terminal behavior, OpenGL behavior, or independent Ares pipeline concepts.

## Rust Destination Boundary

- Modify `crates/ares-core/src/options/layer_change_retraction.rs` to expose the scalar EC long-retraction accessor.
- Modify `crates/ares-core/src/gcode_machine_start_placeholders.rs` to format and replace `[long_retraction_when_ec]`.
- Add focused G-code tests in `crates/ares-core/src/tests/long_retraction_when_ec_placeholder_gcode.rs` and register the module in `crates/ares-core/src/tests/mod.rs`.

## Acceptance Criteria

- `cargo nextest run -p ares-core long_retraction_when_ec` initially fails before implementation and passes after implementation.
- Tests prove configured first value, default, scalar bool, scalar null sentinel truthiness, serialized string composition, layer-change literal scope, and invalid input behavior.
- Existing adjacent placeholder tests still pass with `cargo nextest run -p ares-core long_retraction_when_cut long_retractions_when_ec long_retraction_when_ec`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC <= 400.

## Safety And Rollback

The change is limited to in-memory option parsing and machine-start string substitution in `ares-core`. It has no external I/O, no dependency changes, and no persistent state. Rollback is reverting the accessor, replacement, tests, spec, and plan files from this slice.

## Spec Self-Review

- Placeholder scan: no unresolved placeholder markers.
- Scope check: one upstream scalar nullable bool machine-start placeholder only.
- Ambiguity check: nullable bool nil scalar behavior is explicitly tied to Orca `get_at` sentinel truthiness.
- Consistency check: the scalar placeholder consumes the same `long_retractions_when_ec` option that already drives the vector placeholder.
