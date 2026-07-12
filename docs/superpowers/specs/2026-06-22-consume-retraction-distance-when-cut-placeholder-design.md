# Consume Retraction Distance When Cut Placeholder Design

## Scope

Implement the OrcaSlicer `machine_start_gcode` scalar placeholder `[retraction_distance_when_cut]` in `ares-core`.

The upstream boundary is:

- `OrcaSlicer/src/libslic3r/GCode.cpp:2824`, which sets `retraction_distance_when_cut` from `m_config.retraction_distances_when_cut.get_at(initial_extruder_id)` before processing machine start G-code.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1371`, which declares `retraction_distances_when_cut` as `ConfigOptionFloats` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5088-5094`, which defines the option label, millimeter range `10..=18`, and default `18`.

Rust destination:

- `crates/ares-core/src/options/layer_change_retraction.rs` owns the typed accessor for the first `retraction_distances_when_cut` value.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` owns machine-start placeholder rendering so `crates/ares-core/src/gcode_placeholders.rs` stays below the repository 400 LOC split threshold.
- `crates/ares-core/src/tests/retraction_distance_when_cut_placeholder_gcode.rs` owns end-to-end G-code tests.

## Behavior

When `machine_start_gcode` contains `[retraction_distance_when_cut]`, Ares renders the first configured `retraction_distances_when_cut` value before the first layer, matching Orca's use of `initial_extruder_id` for the scalar placeholder in the current single-initial-extruder Ares pipeline.

Accepted input forms follow the existing numeric-vector parsing used by `retraction_length`: JSON number, JSON array of numbers, or comma/semicolon separated numeric string. The first parsed value is used.

Missing `retraction_distances_when_cut` defaults to `18`.

Configured values must be finite and within Orca's `10..=18` millimeter range. Empty vectors, non-numeric entries, NaN-like values, values below `10`, and values above `18` fail slicing with `SliceError::InvalidInput` mentioning `retraction_distances_when_cut`.

The placeholder is only expanded in machine-start G-code. It remains literal in layer-change G-code and other custom G-code scopes unless a later source-cited slice adds those scopes.

## Deferred Behavior

This slice does not implement:

- `[long_retraction_when_cut]`, `[retraction_distance_when_ec]`, or `[long_retraction_when_ec]`.
- Vector placeholders `[retraction_distances_when_cut]`, `[long_retractions_when_cut]`, `[retraction_distances_when_ec]`, or `[long_retractions_when_ec]`.
- Filament-change or tool-change placeholder refresh paths from `OrcaSlicer/src/libslic3r/GCode.cpp:1055-1056`, `7662-7664`, or `7938-7939`.
- Runtime filament cut mechanics around `OrcaSlicer/src/libslic3r/GCode.cpp:2145-2146`.
- Full multi-extruder initial-tool selection beyond Ares' current machine-start placeholder inputs.

## Compatibility And Safety

The implementation remains inside `ares-core` and uses only platform-neutral parsing and string rendering. It adds no file I/O, terminal behavior, UI behavior, OpenGL, networking, or dependencies.

The `gcode_placeholders.rs` split is structural and preserves existing placeholder output. It is included because the file is already at 394 LOC and adding this behavior directly would violate the repository's split guidance.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core retraction_distance_when_cut` fails before implementation because `[retraction_distance_when_cut]` remains literal.
- After implementation, `cargo nextest run -p ares-core retraction_distance_when_cut` passes.
- Adjacent placeholder tests pass with `cargo nextest run -p ares-core retract_length_placeholder retraction_distance_when_cut`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - Rust touched-file LOC guard with each touched Rust file at or below 400 LOC
- Independent implementation review returns `VERDICT: APPROVE`.
