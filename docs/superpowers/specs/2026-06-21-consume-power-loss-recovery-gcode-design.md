# Consume Power Loss Recovery G-code Design

## Goal

Consume the existing OrcaSlicer `enable_power_loss_recovery` option into concrete Ares G-code behavior instead of only preserving its enum metadata or legacy normalization.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:125-129` defines `enum class PowerLossRecoveryMode` with `PrinterConfiguration`, `Enable`, and `Disable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:553` declares static enum maps for `PowerLossRecoveryMode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1347` declares `enable_power_loss_recovery` as `ConfigOptionEnum<PowerLossRecoveryMode>`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:185-190` maps serialized values `printer_configuration`, `enable`, and `disable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3632-3643` defines labels, tooltip, enum values, and the default `PrinterConfiguration`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7971-7976` normalizes legacy string/boolean-like values `"1"`/`"true"` to `enable` and `"0"`/`"false"` to `disable`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:505-524` formats power loss recovery commands: empty for `PrinterConfiguration`, `M1003 S1/0` for BBL printers, and `M413 S1/0` for Marlin firmware.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4643-4646` emits the selected power loss recovery command once when second-layer behavior runs.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3312-3314` and `OrcaSlicer/src/libslic3r/GCode.cpp:3393-3394` disable power loss recovery at finalization only if the mode was `Enable` and the second-layer command had run.

## Ares Destination Boundary

- Add a small runtime parser in `crates/ares-core/src/options/` for the existing `enable_power_loss_recovery` value.
- Add a focused G-code helper module in `crates/ares-core/src/` that formats Orca-compatible Marlin 2 `M413 S1/0` commands.
- Wire the helper into `crates/ares-core/src/gcode.rs` with minimal call sites:
  - after the layer Z/custom G-code point for the second generated layer;
  - before final machine end G-code when Ares previously emitted the second-layer enable command.
- Add focused runtime tests under `crates/ares-core/src/tests/`.

## Included Behavior

- Missing `enable_power_loss_recovery` defaults to `PrinterConfiguration` and emits no power loss recovery G-code.
- Accepted runtime values are the exact Orca serialized strings:
  - `printer_configuration`
  - `enable`
  - `disable`
- Invalid or non-string values return `SliceError::InvalidInput` naming `enable_power_loss_recovery`.
- Existing Ares legacy normalization for `"1"`, `"true"`, `"0"`, and `"false"` remains the ingestion path for old profiles.
- For `gcode_flavor = "marlin2"`:
  - `enable` emits `M413 S1` once on the second layer, then emits `M413 S0` once before final machine end G-code.
  - `disable` emits `M413 S0` once on the second layer and does not emit an additional final disable.
  - `printer_configuration` emits no `M413`.
- If the model has no second generated layer, no power loss recovery command is emitted and no final disable is emitted.
- If `gcode_comments = true`, emitted commands carry Orca's `set Power-loss Recovery` comment.
- Unsupported active Ares flavors emit no power loss recovery command. This includes `marlin`, `klipper`, `reprapfirmware`, and `repetier`.

## Deferred Behavior

- BBL `M1003 S1/0` remains deferred because Ares does not yet have a runtime `m_is_bbl_printers` / `print.is_BBL_printer()` destination boundary.
- Exact Orca interactions with first-layer inspection, wipe tower finalization, object-by-object finalization, and `m_second_layer_things_done` internals remain deferred until those upstream systems have Ares destinations.
- No new option metadata, public API, CLI flag, WASM API, dependency, or Ares-owned pipeline concept is added in this slice.

## Safety and Platform Constraints

- `ares-core` remains platform-neutral and WASM-compatible: no filesystem, terminal, OpenGL, UI, process, or native-only behavior.
- Keep existing module boundaries small. `crates/ares-core/src/options.rs` is already near the 400 LOC guard, so new parser code must live in a module and only use a minimal registration line if needed.
- Keep `crates/ares-core/src/gcode.rs` below the 400 LOC guard by moving formatting/state logic into a helper module.
- Use `cargo nextest run`, not `cargo test`.

## Acceptance Criteria

- Focused tests fail before implementation and pass after implementation using `cargo nextest run -p ares-core power_loss_recovery_gcode`.
- Tests cover Marlin2 enable second-layer `M413 S1`, final `M413 S0`, Marlin2 disable second-layer `M413 S0`, default/printer-configuration omission, unsupported flavor omission, command comments, and invalid value rejection.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard
- Independent implementation review returns `VERDICT: APPROVE` before commit and push.
