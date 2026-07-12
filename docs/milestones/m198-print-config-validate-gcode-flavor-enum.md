# M198: PrintConfig validate gcode flavor enum value

## Goal
Port OrcaSlicer's `gcode_flavor` enum validation slice into Ares as an explicit `SliceOptions::validate_gcode_flavor_option()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10147-10150`, with active `gcode_flavor` option enum-value context from `PrintConfig.cpp:3785-3817` and serialization mapping context from `PrintConfig.cpp:161-176` / `PrintConfig.hpp:33-46`. It covers only `print_config_def.get("gcode_flavor")->has_enum_value(cfg.gcode_flavor.serialize())` validation and the resulting `gcode_flavor` error. No firmware-retraction compatibility behavior from `PrintConfig.cpp:10131-10145`, fill-pattern checks from `PrintConfig.cpp:10152+`, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_gcode_flavor_option()` returns a key-to-message map like Orca validation.
- Missing `gcode_flavor` uses the source-cited registry default and passes.
- The active Orca `gcode_flavor` option enum values pass: `marlin`, `klipper`, `reprapfirmware`, `repetier`, and `marlin2`.
- Unknown strings such as `unknown-firmware`, and mapped-but-commented-out Orca strings such as `reprap`, `teacup`, `makerware`, `sailfish`, `smoothie`, `mach3`, `machinekit`, and `no-extrusion`, report key `gcode_flavor` with exact message `invalid value {value}`.
- JSON boundary type errors for non-string `gcode_flavor` return `SliceError::InvalidInput`.
- Existing M196 basic validation behavior and M197 firmware-retraction compatibility behavior remain intact.
- `PrintConfig.cpp:10152+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
