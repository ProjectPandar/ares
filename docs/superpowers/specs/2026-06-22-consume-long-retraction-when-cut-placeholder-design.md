# Consume `long_retraction_when_cut` Placeholder Design

## Goal

Consume OrcaSlicer's scalar `long_retraction_when_cut` machine-start placeholder in Ares G-code output so an already-known Orca option affects concrete runtime behavior instead of remaining option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2825` sets `long_retraction_when_cut` from `m_config.long_retractions_when_cut.get_at(initial_extruder_id)` before processing `machine_start_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5081-5086` defines `long_retractions_when_cut` as `coBools` with default `{false}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1372` owns the `ConfigOptionBools long_retractions_when_cut` field on `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1891-1892` defines `ConfigOptionBools::get_at(i)` as indexed lookup with fallback to the first value.
- `OrcaSlicer/src/libslic3r/Config.hpp:1894-1903` serializes bool vectors as comma-separated `1` / `0` values.
- `OrcaSlicer/src/libslic3r/Config.hpp:1916-1948` deserializes non-nullable bool vectors from comma-separated `1` / `0` values and rejects `nil`.

## Rust Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs` adds a narrow `SliceOptions::long_retraction_when_cut()` accessor for the first/current bool value.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders `[long_retraction_when_cut]` as `1` or `0` in the existing machine-start placeholder pass.
- `crates/ares-core/src/tests/long_retraction_when_cut_placeholder_gcode.rs` covers runtime behavior.
- `crates/ares-core/src/tests/mod.rs` registers the new test module.

## Included Behavior

- Missing `long_retractions_when_cut` defaults to `false`, matching Orca's default `{false}`.
- JSON boolean values render as `1` for `true` and `0` for `false`.
- JSON boolean arrays render the first value, matching Ares' current initial-extruder scope and Orca's `get_at(initial_extruder_id)` fallback behavior for the initial tool.
- Comma-separated string values accept only `1` and `0` tokens, matching `ConfigOptionBools` serialization/deserialization.
- Empty arrays, empty string tokens, non-bool JSON array entries, numeric JSON values, `nil`, `true`, and `false` string tokens are rejected with `SliceError::InvalidInput` containing `long_retractions_when_cut`.
- `[long_retraction_when_cut]` expands only in `machine_start_gcode`; the same token in `layer_change_gcode` remains literal.
- Rendering composes with existing machine-start placeholders such as `[retraction_distance_when_cut]` and `[num_extruders]`.

## Deferred Behavior

- `long_retraction_when_cut` refresh during tool-change and filament-change G-code paths from `GCode.cpp:1056`, `GCode.cpp:7664`, `GCode.cpp:7939`.
- Vector placeholder `[long_retractions_when_cut]` from `GCode.cpp:2832`.
- `long_retraction_when_ec`, `retraction_distance_when_ec`, and their vector placeholders.
- Full multi-extruder initial tool selection beyond Ares' current first/current value convention.
- Generated filament-cut movement, purge reduction policy, and `enable_long_retraction_when_cut` gating semantics outside this placeholder rendering slice.
- Semicolon-separated bool strings, because the cited upstream `ConfigOptionBools` path splits on commas.

## Docs Impact

No roadmap or architecture update is required for this slice. The existing roadmap already records the placeholder rewrite direction, and this spec plus the implementation plan are the source-cited delivery artifacts for the concrete behavior change.

## Acceptance Criteria

- Focused RED run: `cargo nextest run -p ares-core long_retraction_when_cut` fails before implementation because the placeholder remains literal and invalid `long_retractions_when_cut` values are not rejected through the new behavior.
- Focused GREEN run: `cargo nextest run -p ares-core long_retraction_when_cut` passes after implementation.
- Adjacent regression run: `cargo nextest run -p ares-core retraction_distance_when_cut long_retraction_when_cut` passes.
- Full verification passes before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, with every touched Rust file at or below 400 LOC
- Independent spec, plan, and implementation reviewers return `VERDICT: APPROVE`.
