# Consume Retraction Distances When Cut Vector Placeholder Design

## Goal

Consume OrcaSlicer `retraction_distances_when_cut` as concrete `machine_start_gcode` behavior by rendering the vector placeholder `[retraction_distances_when_cut]` to a comma-separated numeric list.

This is a source-cited Rust rewrite slice, not new Ares pipeline design.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2824` sets scalar compatibility placeholder `retraction_distance_when_cut` from `m_config.retraction_distances_when_cut.get_at(initial_extruder_id)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2831` sets vector placeholder `retraction_distances_when_cut` from `new ConfigOptionFloats(m_config.retraction_distances_when_cut)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5088-5094` defines `retraction_distances_when_cut` as `coFloats`, range `10..=18`, default `ConfigOptionFloats {18}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1371` owns the `ConfigOptionFloats retraction_distances_when_cut` member.

## Rust Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs`
  - Add an accessor that returns the full parsed and range-validated `retraction_distances_when_cut` numeric vector.
  - Reuse it for the existing scalar first-extruder `retraction_distance_when_cut` accessor.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs`
  - Replace `[retraction_distances_when_cut]` in `machine_start_gcode` with the full vector formatted using the existing numeric placeholder formatting.
- `crates/ares-core/src/tests/retraction_distances_when_cut_vector_placeholder_gcode.rs`
  - Add behavior tests proving rendered G-code changes, defaults, composition with the scalar placeholder, scope, and invalid input handling.

## Included Behavior

1. Missing `retraction_distances_when_cut` uses Orca default `[18]` and renders `[retraction_distances_when_cut]` as `18`.
2. A JSON number renders as one value.
3. A JSON numeric array renders all values in order as comma-separated numbers.
4. A separated numeric string accepted by Ares' existing numeric vector parser renders all values in order.
5. All parsed values are range-validated against Orca `10..=18`; any out-of-range value rejects the option.
6. Invalid values return `SliceError::InvalidInput` containing `retraction_distances_when_cut`:
   - empty arrays
   - non-numeric values
   - non-finite values
   - values outside `10..=18`
   - empty string tokens
7. Existing scalar `[retraction_distance_when_cut]` behavior remains unchanged and continues to use the first parsed value.
8. Replacement applies only to `machine_start_gcode`; `[retraction_distances_when_cut]` remains literal in `layer_change_gcode`.

## Deferred Behavior

- `retraction_distances_when_ec`, `retraction_distance_when_ec`, `long_retractions_when_ec`, and `long_retraction_when_ec` remain deferred because they require a separate nullable-value parity decision.
- Full Orca placeholder parser expression evaluation remains deferred; this slice only handles literal bracket placeholders currently implemented in Ares.
- No option metadata additions, no new crates, no dependencies, and no filesystem or terminal behavior in `ares-core`.

## Acceptance Criteria

- Focused RED test run fails before implementation with `cargo nextest run -p ares-core retraction_distances_when_cut`.
- Focused GREEN test run passes after implementation with `cargo nextest run -p ares-core retraction_distances_when_cut`.
- Adjacent scalar/vector regression run passes with `cargo nextest run -p ares-core retraction_distance_when_cut retraction_distances_when_cut`.
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
- Ambiguity check: nullable EC placeholders are explicitly deferred; this slice only covers non-nullable `ConfigOptionFloats`.
- Consistency check: acceptance criteria use `cargo nextest run`, not `cargo test`.
