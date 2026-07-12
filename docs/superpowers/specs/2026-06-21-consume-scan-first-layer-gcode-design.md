# Consume Scan First Layer G-code Design

## Goal

Consume the existing OrcaSlicer `scan_first_layer` option into concrete Ares G-code behavior instead of only preserving its registry metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1346` declares `scan_first_layer` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3624-3629` defines `scan_first_layer` as a boolean option with default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4643-4657` runs second-layer-only printer setup; when `print.is_BBL_printer()` and `scan_first_layer` are true, it retracts, emits `M976 S1 P1 ; scan model before printing 2nd layer`, emits `M400 P100`, then unretracts.
- `OrcaSlicer/src/libslic3r/Print.hpp:1070-1071` exposes `Print::is_BBL_printer()`.
- `OrcaSlicer/src/slic3r/GUI/BackgroundSlicingProcess.cpp:199` and `:683` set `Print::is_BBL_printer()` from the preset bundle's BBL vendor status in GUI slicing.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:2559-2564` also infers BBL printer status from `printer_model` values that start with `Bambu Lab` when consuming G-code/config data.

## Ares Destination Boundary

- Add a focused helper module in `crates/ares-core/src/` that:
  - parses `scan_first_layer` as a boundary boolean via existing `SliceOptions::bool_option`;
  - treats a string `printer_model` starting with `Bambu Lab` as the Ares core-local BBL predicate for this slice;
  - formats the second-layer inspection commands.
- Wire the helper into `crates/ares-core/src/gcode.rs` immediately after the existing second-layer power-loss recovery command point and before layer fan/segment output.
- Add focused runtime tests under `crates/ares-core/src/tests/`.

## Included Behavior

- Missing `scan_first_layer` defaults to `false` and emits no inspection G-code.
- `scan_first_layer = true` emits inspection G-code only on the second generated layer and only when `printer_model` starts with `Bambu Lab`.
- The emitted inspection block is:
  - `M976 S1 P1 ; scan model before printing 2nd layer`
  - `M400 P100`
- The block is emitted after the second layer Z travel and before `; segment_count = ...`, matching the current Ares second-layer setup insertion point used for power-loss recovery.
- Non-Bambu `printer_model`, missing `printer_model`, and `scan_first_layer = false` are no-ops.
- Invalid non-boolean `scan_first_layer` values return `SliceError::InvalidInput` naming `scan_first_layer`.
- Single-layer outputs emit no inspection G-code because there is no second generated layer.

## Deferred Behavior

- Exact Orca retraction/unretraction around the scan block is deferred because Ares does not yet have a runtime retraction/unretraction G-code model in `ares-core`.
- GUI preset-bundle vendor detection is deferred because `ares-core` is platform-neutral and has no GUI preset bundle. This slice uses the source-cited `printer_model` prefix predicate already present in upstream G-code/config consumption.
- Other BBL-only G-code behavior, first-layer inspection result handling, camera runtime behavior, printer networking, firmware response handling, and UI behavior remain deferred.
- No new option metadata, public API, CLI flag, WASM API, dependency, or Ares-owned pipeline concept is added in this slice.

## Safety and Platform Constraints

- `ares-core` remains platform-neutral and WASM-compatible: no filesystem, terminal, OpenGL, UI, process, networking, or native-only behavior.
- Keep `crates/ares-core/src/options.rs` unchanged because it is already at the 400 LOC guard.
- Keep `crates/ares-core/src/gcode.rs` at or below the 400 LOC guard by moving formatting and predicate logic into a helper module.
- Use `cargo nextest run`, not `cargo test`.

## Acceptance Criteria

- Focused tests fail before implementation and pass after implementation using `cargo nextest run -p ares-core scan_first_layer_gcode`.
- Tests cover Bambu second-layer emission, non-Bambu omission, default omission, disabled omission, invalid value rejection, and single-layer omission.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard
- Independent implementation review returns `VERDICT: APPROVE` before commit and push.
