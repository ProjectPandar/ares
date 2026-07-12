# Consume Long EC Retraction Vector Placeholder Design

## Goal

Port OrcaSlicer's `long_retractions_when_ec` nullable bool vector placeholder into Ares machine-start G-code rendering so existing option data produces concrete G-code text instead of remaining inert metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2834` registers `[long_retractions_when_ec]` with `new ConfigOptionBoolsNullable(m_config.long_retractions_when_ec)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2827` registers scalar `[long_retraction_when_ec]` from `get_at(initial_extruder_id)`, but this slice does not implement that scalar placeholder.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5096-5100` defines `long_retractions_when_ec` as nullable bools with default `ConfigOptionBoolsNullable {false}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1374` stores the field as `ConfigOptionBoolsNullable long_retractions_when_ec`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1876,1894-1903,1916-1948,1951-1958` defines nullable bool serialization/deserialization: `nil` is the nullable sentinel, serialized vector values are comma-separated `nil`, `1`, and `0`.

## Current Ares Context

- `crates/ares-core/src/options/layer_change_retraction.rs` already parses `long_retractions_when_cut` for non-nullable vector placeholders.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already renders machine-start placeholders including `[long_retractions_when_cut]`.
- Existing tests in `crates/ares-core/src/tests/long_retractions_when_cut_vector_placeholder_gcode.rs` define the adjacent non-nullable vector behavior and scope boundaries.
- Registry metadata for `long_retractions_when_ec` already exists; this slice must consume that option into runtime G-code behavior rather than adding option metadata.

## Included Behavior

1. Add a `SliceOptions::long_retractions_when_ec()` runtime accessor that returns a vector preserving nullable bool values.
2. Missing `long_retractions_when_ec` uses Orca's default vector `[false]`, rendered as `0`.
3. Supported JSON input forms:
   - boolean scalar: `true` -> `1`, `false` -> `0`;
   - JSON null scalar: `null` -> `nil`;
   - array of booleans and nulls: `[true, null, false]` -> `1,nil,0`;
   - comma-separated Orca serialized string using only `nil`, `1`, and `0`, with optional whitespace around tokens.
4. Machine start G-code replaces `[long_retractions_when_ec]` with comma-separated `1`, `0`, and `nil`.
5. `[long_retractions_when_ec]` remains literal in `layer_change_gcode`; this slice only ports the `GCode.cpp` machine-start placeholder boundary already handled by `gcode_machine_start_placeholders`.
6. Invalid input returns `SliceError::InvalidInput` mentioning `long_retractions_when_ec`.

## Deferred Behavior

- Do not implement scalar `[long_retraction_when_ec]`; Orca's nullable bool `get_at` behavior converts the nil sentinel through a bool return path and needs a separate parity decision.
- Do not implement `[retraction_distances_when_ec]` or `[retraction_distance_when_ec]`; nullable float formatting is a separate slice.
- Do not add new option registry metadata, crates, dependencies, file I/O, UI, terminal behavior, OpenGL behavior, or independent Ares pipeline concepts.

## Rust Destination Boundary

- Modify `crates/ares-core/src/options/layer_change_retraction.rs` for nullable bool-vector parsing.
- Modify `crates/ares-core/src/gcode_machine_start_placeholders.rs` for placeholder formatting and replacement.
- Add focused G-code tests in `crates/ares-core/src/tests/long_retractions_when_ec_vector_placeholder_gcode.rs` and register the module in `crates/ares-core/src/tests/mod.rs`.

## Acceptance Criteria

- `cargo nextest run -p ares-core long_retractions_when_ec` initially fails before implementation and passes after implementation.
- Tests prove configured vector, default, scalar bool, null/nil handling, serialized string composition, layer-change literal scope, and invalid input behavior.
- Existing adjacent cut placeholder tests still pass with `cargo nextest run -p ares-core long_retractions_when_cut long_retractions_when_ec`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC <= 400.

## Safety And Rollback

The change is limited to in-memory option parsing and machine-start string substitution in `ares-core`. It has no external I/O, no dependency changes, and no persistent state. Rollback is reverting the accessor, formatter, replacement, tests, spec, and plan files from this slice.

## Spec Self-Review

- Placeholder scan: no unresolved placeholder markers.
- Scope check: one upstream placeholder vector only.
- Ambiguity check: scalar EC placeholder and nullable float EC placeholders are explicitly deferred.
- Consistency check: accepted input and rendered output are both expressed as Orca-compatible `nil`, `1`, and `0` tokens.
