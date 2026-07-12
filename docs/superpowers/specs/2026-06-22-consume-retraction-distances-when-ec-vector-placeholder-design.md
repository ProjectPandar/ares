# Consume EC Retraction Distance Vector Placeholder Design

## Goal

Port OrcaSlicer's `retraction_distances_when_ec` nullable float vector placeholder into Ares machine-start G-code rendering so existing option data becomes concrete G-code text instead of remaining inert option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2833` registers `[retraction_distances_when_ec]` with `new ConfigOptionFloatsNullable(m_config.retraction_distances_when_ec)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2826` registers scalar `[retraction_distance_when_ec]` from `m_config.retraction_distances_when_ec.get_at(initial_extruder_id)`, but this slice does not implement that scalar placeholder.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5102-5109` defines `retraction_distances_when_ec` as nullable floats, range `0..=10`, with default `ConfigOptionFloatsNullable {10}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1373` stores the field as `ConfigOptionFloatsNullable retraction_distances_when_ec`.
- `OrcaSlicer/src/libslic3r/Config.hpp:832-893,910-919` defines nullable float behavior: nil is represented as NaN, serialized as `nil`, and comma-separated string deserialization accepts `nil` plus numeric values.

## Current Ares Context

- `crates/ares-core/src/options/layer_change_retraction.rs` already parses non-nullable `retraction_distances_when_cut` and nullable bool `long_retractions_when_ec`.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already renders machine-start placeholders for `[retraction_distances_when_cut]` and `[long_retractions_when_ec]`.
- Existing tests in `crates/ares-core/src/tests/retraction_distances_when_cut_vector_placeholder_gcode.rs` and `crates/ares-core/src/tests/long_retractions_when_ec_vector_placeholder_gcode.rs` define the adjacent vector placeholder and nullable scope patterns.
- Registry metadata for `retraction_distances_when_ec` already exists; this slice must consume that option into runtime G-code behavior rather than adding option metadata.

## Included Behavior

1. Add a `SliceOptions::retraction_distances_when_ec()` runtime accessor that returns a vector preserving nullable finite float values.
2. Missing `retraction_distances_when_ec` uses Orca's default vector `[10]`, rendered as `10`.
3. Supported JSON input forms:
   - finite numeric scalar in `0..=10`: `2.5` -> `2.5`;
   - JSON null scalar: `null` -> `nil`;
   - nonempty array of finite numbers and nulls in range: `[0, null, 10]` -> `0,nil,10`;
   - comma-separated Orca serialized string using `nil` and numeric values, with optional whitespace around tokens.
4. Machine start G-code replaces `[retraction_distances_when_ec]` with comma-separated formatted values, using existing integer-trimming number formatting for finite numbers and `nil` for nulls.
5. `[retraction_distances_when_ec]` remains literal in `layer_change_gcode`; this slice only ports the `GCode.cpp` machine-start vector placeholder boundary.
6. Invalid input returns `SliceError::InvalidInput` mentioning `retraction_distances_when_ec`.

## Deferred Behavior

- Do not implement scalar `[retraction_distance_when_ec]`; Orca's nullable float scalar path can expose NaN/nil formatting and needs a separate parity decision.
- Do not change `[retraction_distances_when_cut]`, `[retraction_distance_when_cut]`, `[long_retractions_when_ec]`, or `[long_retraction_when_ec]`.
- Do not add new option registry metadata, crates, dependencies, file I/O, UI, terminal behavior, OpenGL behavior, or independent Ares pipeline concepts.

## Rust Destination Boundary

- Modify `crates/ares-core/src/options/layer_change_retraction.rs` for nullable float-vector parsing and validation.
- Modify `crates/ares-core/src/gcode_machine_start_placeholders.rs` for nullable float placeholder formatting and replacement.
- Add focused G-code tests in `crates/ares-core/src/tests/retraction_distances_when_ec_vector_placeholder_gcode.rs` and register the module in `crates/ares-core/src/tests/mod.rs`.

## Acceptance Criteria

- `cargo nextest run -p ares-core retraction_distances_when_ec` initially fails before implementation and passes after implementation.
- Tests prove configured nullable vector, default, scalar number, scalar null, serialized string composition, layer-change literal scope, and invalid input behavior.
- Existing adjacent placeholder tests still pass with `cargo nextest run -p ares-core retraction_distances_when_cut retraction_distances_when_ec long_retractions_when_ec`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC <= 400.

## Safety And Rollback

The change is limited to in-memory option parsing and machine-start string substitution in `ares-core`. It has no external I/O, no dependency changes, and no persistent state. Rollback is reverting the accessor, formatter, replacement, tests, spec, and plan files from this slice.

## Spec Self-Review

- Placeholder scan: no unresolved placeholder markers.
- Scope check: one upstream nullable float vector placeholder only.
- Ambiguity check: scalar EC placeholder is explicitly deferred.
- Consistency check: accepted input and rendered output are both expressed as Orca-compatible `nil` plus numeric tokens.
