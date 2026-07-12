# Consume G-code Flavor Preamble Design

## Goal

Consume OrcaSlicer `gcode_flavor` as concrete Ares G-code writer behavior for the preamble and E reset gates. The slice must turn the already registered and validated option into emitted G-code differences instead of adding more option metadata.

## Upstream Boundary

Line citations are pinned to the checked-out `OrcaSlicer` revision `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:33-46` declares `GCodeFlavor` variants.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1355` declares `ConfigOptionEnum<GCodeFlavor> gcode_flavor`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:161-176` maps serialized flavor keys such as `marlin`, `klipper`, `reprapfirmware`, `repetier`, and `marlin2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785-3817` registers the visible `gcode_flavor` option and defaults it to `gcfMarlinLegacy` / `marlin`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:81-106` emits `G90`/`G21` unless flavor is MakerWare, then emits extrusion-axis mode commands only for RepRapSprinter, RepRapFirmware, MarlinLegacy, MarlinFirmware, Teacup, Repetier, Smoothie, and Klipper.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:480-502` suppresses `G92 E0` for Mach3, MakerWare, and Sailfish and otherwise emits reset only in absolute E mode.

## Current Ares State

- Ares registry already exposes `gcode_flavor` with upstream metadata and default `marlin`.
- Ares validation already accepts active UI values `marlin`, `klipper`, `reprapfirmware`, `repetier`, and `marlin2`, and rejects inactive or unsupported values.
- `GCodeWriter::preamble()` currently always emits Marlin-like `G90`, `G21`, and relative/absolute E commands, ignoring `gcode_flavor`.

## Ares Destination Boundary

- Add `crates/ares-core/src/options/gcode_flavor.rs` with a compact `GCodeFlavor` enum and `SliceOptions::gcode_flavor()`.
- Keep `crates/ares-core/src/options.rs` to a module declaration only for this slice.
- Extend `crates/ares-core/src/gcode_writer.rs` so `GCodeWriter` stores flavor and applies the Orca preamble gates for active Ares flavors.
- Update `crates/ares-core/src/gcode.rs` to read `options.gcode_flavor()?` once and set the writer flavor before `preamble()`.
- Add focused tests in new files where practical to avoid pushing near-limit files over 400 LOC.

## Included Behavior

1. Missing `gcode_flavor` defaults to `marlin`.
2. `marlin`, `marlin2`, `klipper`, `reprapfirmware`, and `repetier` all keep the existing default preamble shape for active Ares flavors: `G90`, `G21`, and either `M83` or `M82` plus `G92 E0` depending on `use_relative_e_distances`.
3. A non-string `gcode_flavor` is rejected with `SliceError::InvalidInput`.
4. An unsupported string such as `makerware` is rejected with `SliceError::InvalidInput`.
5. The writer has explicit unit coverage for the currently unreachable upstream gates: MakerWare preamble omits `G90`/`G21` and extrusion mode commands; Sailfish suppresses absolute-E `G92 E0`; active Marlin-like flavors keep mode commands.

## Deferred Behavior

- Ares does not activate hidden Orca flavor strings (`reprap`, `teacup`, `makerware`, `sailfish`, `mach3`, `machinekit`, `smoothie`, `no-extrusion`) through public `SliceOptions` in this slice; this preserves the existing validation boundary.
- Temperature, fan, retract/unretract, acceleration command syntax, time-estimator, and firmware-specific G-code beyond `GCodeWriter::preamble()` and `reset_e()` are deferred to later source-cited flavor slices.
- `gcode_flavor` UI label/enum metadata generation is already represented by registry/staged-source modules and is not changed here.

## Docs Impact

This spec and its implementation plan document the slice. No roadmap update is required because this continues the current option-consumption milestone and does not change milestone ordering.

## Acceptance Criteria

- Option tests prove default `gcode_flavor` maps to `GCodeFlavor::MarlinLegacy`, active strings map to the expected enum values, non-string values fail, and inactive strings fail.
- Writer tests prove active Marlin-like flavors emit `G90`, `G21`, and extrusion mode commands.
- Writer tests prove MakerWare omits `G90`, `G21`, `M82`, `M83`, and `G92 E0`.
- Writer tests prove Sailfish suppresses absolute-E reset.
- Integration tests prove slicing with `gcode_flavor: "klipper"` still emits `G90`, `G21`, and `M83`, and slicing with `gcode_flavor: "makerware"` is rejected.
- Existing relative-E behavior remains intact.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass.

## Safety

The runtime surface remains limited to the active, already validated Ares flavor strings. Hidden Orca flavors are exercised only through writer unit tests so the writer boundary is ready for future slices without weakening public option validation.
