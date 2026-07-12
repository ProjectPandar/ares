# Consume Wrapping Detection G-code Design

## Goal

Consume the existing OrcaSlicer `enable_wrapping_detection` and `wrapping_detection_gcode` options into concrete Ares G-code output instead of leaving `wrapping_detection_gcode` as metadata only.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1348-1360` declares `enable_wrapping_detection`, `wrapping_detection_layers`, `wrapping_exclude_area`, and `wrapping_detection_gcode` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3987-4003` defines `enable_wrapping_detection` default `false`, `wrapping_detection_layers` default `20`, and `wrapping_exclude_area`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4312-4318` defines `wrapping_detection_gcode` as an advanced multiline string defaulting to `""`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5052-5071` defines `insert_wrapping_detection_gcode`: when `enable_wrapping_detection` is true and `wrapping_detection_gcode` is not empty, it renders the template with `layer_num`, `layer_z`, `max_layer_z`, `most_used_physical_extruder_id`, and `curr_physical_extruder_id`, appends a newline, and updates writer Z if rendered G-code changed Z.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5074`, `:5142-5145`, and `:5163-5166` insert wrapping detection G-code once per layer before the normal layer toolchange/extrusion body.
- `OrcaSlicer/resources/profiles/BBL/machine/Bambu Lab H2D 0.4 nozzle.json:116` and `Bambu Lab P2S 0.4 nozzle.json:250` show real `wrapping_detection_gcode` templates using `layer_num` conditionals and firmware commands such as `G39`.

## Ares Destination Boundary

- Add a focused `crates/ares-core/src/gcode_wrapping_detection.rs` helper that owns option parsing, template rendering, and once-per-layer command formatting for this slice.
- Add a focused `crates/ares-core/src/gcode_layer_markers.rs` helper that groups post-Z layer marker G-code after existing power-loss recovery. It will call the existing `scan_first_layer` helper and the new wrapping detection helper.
- Modify `crates/ares-core/src/gcode.rs` by replacing the direct scan-first-layer post-Z call with the grouped helper call. This keeps `gcode.rs` at or below the 400 LOC guard.
- Modify `crates/ares-core/src/lib.rs` to register the new internal modules.
- Add runtime tests under `crates/ares-core/src/tests/wrapping_detection_gcode.rs` and register them in `crates/ares-core/src/tests/mod.rs`.

## Included Behavior

- Missing `enable_wrapping_detection` defaults to `false`.
- Missing `wrapping_detection_gcode` defaults to `""`.
- `enable_wrapping_detection = true` and non-empty string `wrapping_detection_gcode` emits rendered wrapping detection G-code once for each generated layer.
- Emission occurs after layer Z travel and after existing power-loss recovery G-code, before scan-first-layer G-code, fan commands, and `; segment_count = ...`.
- `enable_wrapping_detection = false` or empty `wrapping_detection_gcode` emits no wrapping detection G-code.
- `enable_wrapping_detection` must be a JSON boolean. Invalid non-boolean values return `SliceError::InvalidInput` mentioning `enable_wrapping_detection`.
- `wrapping_detection_gcode` must be a JSON string. Invalid non-string values return `SliceError::InvalidInput` mentioning `wrapping_detection_gcode`, even when wrapping detection is disabled, matching Ares' boundary-validation style for custom G-code string options.
- The renderer replaces direct placeholder forms:
  - `{layer_num}` and `[layer_num]`
  - `{layer_z}` and `[layer_z]`
  - `{max_layer_z}` and `[max_layer_z]`
  - `{most_used_physical_extruder_id}` and `[most_used_physical_extruder_id]`
  - `{curr_physical_extruder_id}` and `[curr_physical_extruder_id]`
- `layer_num` uses Ares' existing one-based layer number convention for custom layer G-code.
- `layer_z` and `max_layer_z` use the same formatted current layer Z value as the current Ares `time_lapse_gcode` compatibility shell until the upstream max-layer accumulator is ported.
- The physical extruder placeholders render as `0` in this slice because Ares does not yet have Orca's `physical_extruder_map`, current writer filament, or multi-extruder wrapping insertion model in the G-code writer.
- Unknown placeholders, Orca conditionals, and expression placeholders are preserved unchanged.

## Deferred Behavior

- `wrapping_detection_layers`, `wrapping_exclude_area`, clumping exclusion geometry, plate collision validation, and wipe tower depth changes are deferred.
- Orca's `m_enable_wrapping_detection` constructor gate requiring a wrapping exclude polygon and single used filament is deferred because Ares does not yet model that upstream geometry/wipe-tower path. This slice consumes the explicit G-code options only.
- Full Orca placeholder parsing, arithmetic, conditionals, and expression evaluation are deferred. Existing Ares custom G-code behavior only performs direct placeholder replacement.
- Updating the writer's internal Z from rendered wrapping G-code is deferred. Ares currently keeps generated path Z state in the layer loop rather than deriving it from custom G-code strings.
- Toolchange/retract interaction, wipe tower insertion branches, `physical_extruder_map`, current filament writer state, object labels, GUI behavior, printer camera behavior, and firmware response handling remain deferred.
- No new option metadata, public API, CLI flag, WASM API, dependency, filesystem access, UI, OpenGL, terminal behavior, or Ares-owned pipeline concept is added.

## Safety and Platform Constraints

- `ares-core` remains platform-neutral and WASM-compatible.
- Do not edit `crates/ares-core/src/options.rs` because it is at the 400 LOC guard.
- Keep every touched Rust file at or below 400 LOC.
- Use `cargo nextest run`, not `cargo test`.
- No new dependencies.

## Acceptance Criteria

- Focused RED tests fail before implementation using `cargo nextest run -p ares-core wrapping_detection_gcode`.
- After implementation, focused tests prove:
  - enabled wrapping detection emits one rendered block per generated layer;
  - wrapping detection appears after power-loss recovery and before scan-first-layer/fan/segment output;
  - disabled, missing, and empty options are no-ops;
  - brace and bracket direct placeholders render;
  - unknown conditionals and expression placeholders remain unchanged;
  - invalid `enable_wrapping_detection` and invalid `wrapping_detection_gcode` return `SliceError::InvalidInput`.
- Existing `scan_first_layer_gcode` tests still pass after routing through the grouped post-Z helper.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core wrapping_detection_gcode scan_first_layer_gcode`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard
- Independent implementation review returns `VERDICT: APPROVE` before commit and push.
