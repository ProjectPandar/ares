# M197: PrintConfig validate firmware retraction compatibility

## Goal
Port OrcaSlicer's firmware-retraction validation slice into Ares as an explicit `SliceOptions::validate_firmware_retraction_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10131-10145`, with enum string mapping context from `PrintConfig.cpp:161-176` and `PrintConfig.hpp:33-46`. It covers only `use_firmware_retraction` compatibility with supported `gcode_flavor` values and incompatibility with enabled `wipe` entries. No `gcode_flavor` enum validation from `PrintConfig.cpp:10147-10150`, later validation checks, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_firmware_retraction_options()` returns a key-to-message map like Orca validation.
- Default/absent values pass using source-cited registry defaults.
- `use_firmware_retraction = false` produces no firmware-retraction errors regardless of `gcode_flavor` or `wipe`.
- `use_firmware_retraction = true` accepts Orca-supported firmware flavors: `klipper`, `smoothie`, `reprap`, `reprapfirmware`, `marlin`, `marlin2`, `machinekit`, and `repetier`.
- `use_firmware_retraction = true` rejects known unsupported firmware flavors such as `teacup`, `makerware`, `sailfish`, `mach3`, and `no-extrusion` with the exact upstream support message: `--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware`.
- `use_firmware_retraction = true` rejects any enabled `wipe` array entry with the exact upstream wipe incompatibility message: `--use-firmware-retraction is not compatible with --wipe`.
- When unsupported flavor and enabled wipe both apply, the unsupported-flavor message (`--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware`) remains for `use_firmware_retraction`, matching `std::map::emplace` first-insert-wins behavior.
- JSON boundary type errors return `SliceError::InvalidInput`.
- Existing M196 basic validation behavior remains intact.
- `PrintConfig.cpp:10147+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
