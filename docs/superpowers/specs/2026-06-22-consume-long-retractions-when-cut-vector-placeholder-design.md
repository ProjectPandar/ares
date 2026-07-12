# Consume Long Retractions When Cut Vector Placeholder Design

## Goal

Consume OrcaSlicer `long_retractions_when_cut` as concrete `machine_start_gcode` behavior by rendering the vector placeholder `[long_retractions_when_cut]` to a comma-separated `0`/`1` list.

This is a source-cited Rust rewrite slice, not new Ares pipeline design.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2825` sets scalar compatibility placeholder `long_retraction_when_cut` from `m_config.long_retractions_when_cut.get_at(initial_extruder_id)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2832` sets vector placeholder `long_retractions_when_cut` from `new ConfigOptionBools(m_config.long_retractions_when_cut)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5081-5086` defines `long_retractions_when_cut` as `coBools` with default `ConfigOptionBools {false}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1372` owns the `ConfigOptionBools long_retractions_when_cut` member.

## Rust Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs`
  - Add an accessor that returns the full parsed `long_retractions_when_cut` boolean vector.
  - Reuse it for the existing scalar first-extruder accessor where possible.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs`
  - Replace `[long_retractions_when_cut]` in `machine_start_gcode` with the full vector formatted as Orca-style `1,0,...`.
- `crates/ares-core/src/tests/long_retractions_when_cut_vector_placeholder_gcode.rs`
  - Add behavior tests proving rendered G-code changes, defaults, composition, scope, and invalid input handling.

## Included Behavior

1. Missing `long_retractions_when_cut` uses Orca default `[false]` and renders `[long_retractions_when_cut]` as `0`.
2. A JSON boolean renders as one value: `true` -> `1`, `false` -> `0`.
3. A JSON boolean array renders all values in order as comma-separated `1`/`0`.
4. A comma-separated Orca-style string containing only `1` and `0` renders all values in order.
5. Invalid values return `SliceError::InvalidInput` containing `long_retractions_when_cut`:
   - empty arrays
   - arrays containing non-booleans
   - JSON null
   - JSON numbers
   - strings other than comma-separated `1`/`0`
   - empty string tokens
6. Existing scalar `[long_retraction_when_cut]` behavior remains unchanged and continues to use the first parsed value.
7. Replacement applies only to `machine_start_gcode`; `[long_retractions_when_cut]` remains literal in `layer_change_gcode`.

## Deferred Behavior

- `retraction_distances_when_cut` vector placeholder remains deferred.
- `retraction_distance_when_ec`, `long_retraction_when_ec`, and their nullable vector placeholders remain deferred because they require a separate nullable-value parity decision.
- Full Orca placeholder parser expression evaluation remains deferred; this slice only handles literal bracket placeholders currently implemented in Ares.
- No option metadata additions, no new crates, no dependencies, and no filesystem or terminal behavior in `ares-core`.

## Acceptance Criteria

- Focused RED test run fails before implementation with `cargo nextest run -p ares-core long_retractions_when_cut`.
- Focused GREEN test run passes after implementation with `cargo nextest run -p ares-core long_retractions_when_cut`.
- Adjacent scalar/vector regression run passes with `cargo nextest run -p ares-core long_retraction_when_cut long_retractions_when_cut`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, with each touched Rust file at or below 400 lines.

## Safety And Rollback

The change is confined to start G-code placeholder rendering and option parsing in `ares-core`. Rollback is a single commit revert. No persisted data, external services, filesystem I/O, UI, OpenGL, or platform-specific APIs are introduced.

## Self Review

- Placeholder scan: no TBD/TODO/open placeholders remain.
- Scope check: one upstream placeholder family, one Rust destination boundary, one test file.
- Ambiguity check: nullable EC placeholders are explicitly deferred; this slice only covers non-nullable `ConfigOptionBools`.
- Consistency check: acceptance criteria use `cargo nextest run`, not `cargo test`.
