# M197 Spec: PrintConfig validate firmware retraction compatibility

## Goal
Port OrcaSlicer's firmware-retraction validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_firmware_retraction_options()`, returning validation messages for this compatibility slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10131-10145`: `use_firmware_retraction` firmware flavor support check and `wipe` incompatibility check.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:161-176`: string mapping for `GCodeFlavor` values used to represent the supported/unsupported firmware flavors at Ares' JSON boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:33-46`: `GCodeFlavor` enum declaration context.
- Option-definition default anchors for `use_firmware_retraction`, `gcode_flavor`, and `wipe` are already carried in the Ares registry.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10147-10150` `gcode_flavor` enum value validation.
- `PrintConfig.cpp:10152+` fill-pattern and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_firmware_retraction_options(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private bool/string helpers for this validation slice.
- `crates/ares-core/src/options/tests/validation.rs`: add source-behavior tests.
- `docs/roadmap.md` and `docs/milestones/m197-print-config-validate-firmware-retraction.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_firmware_retraction_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing keys use source-cited registry defaults, matching the upstream `FullPrintConfig` default context.
3. If `use_firmware_retraction` is `false`, return no firmware-retraction errors from this slice regardless of `gcode_flavor` or `wipe` values.
4. If `use_firmware_retraction` is `true`, accept these Orca-supported `gcode_flavor` strings: `klipper`, `smoothie`, `reprap`, `reprapfirmware`, `marlin`, `marlin2`, `machinekit`, and `repetier`.
5. If `use_firmware_retraction` is `true`, report key `use_firmware_retraction` with the exact upstream support message `--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware` when `gcode_flavor` is a known unsupported Orca flavor from `PrintConfig.cpp:161-176`, such as `teacup`, `makerware`, `sailfish`, `mach3`, or `no-extrusion`.
6. Do not implement the deferred `PrintConfig.cpp:10147-10150` `gcode_flavor` enum validation block: arbitrary unknown `gcode_flavor` strings such as `"unknown-firmware"` must not be rejected by this M197 API. The only unsupported flavor strings reported by this slice are the known mapped Orca enum strings from `PrintConfig.cpp:161-176`: `teacup`, `makerware`, `sailfish`, `mach3`, and `no-extrusion`.
7. If `use_firmware_retraction` is `true`, report key `use_firmware_retraction` with the exact upstream wipe incompatibility message `--use-firmware-retraction is not compatible with --wipe` when any `wipe` vector entry is `true`.
8. If both unsupported flavor and enabled wipe apply, preserve Orca `std::map::emplace` first-insert-wins behavior: the earlier unsupported-flavor message (`--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware`) remains for the shared `use_firmware_retraction` key.
9. Parse `wipe` as a bool vector from a JSON bool or JSON bool array; reject non-bool members with `SliceError::InvalidInput`.
10. Parse `use_firmware_retraction` as a JSON bool and `gcode_flavor` as a JSON string; malformed JSON boundary types return `SliceError::InvalidInput`.
11. Preserve existing `validate_basic_fdm_options`, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
12. Do not add full validation dispatch, `gcode_flavor` enum validation, fill-pattern checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
13. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove `use_firmware_retraction = false` returns no errors for unsupported `gcode_flavor` and enabled `wipe`.
- Tests prove all supported firmware flavor strings pass when `use_firmware_retraction = true` and `wipe` is false.
- Tests prove known unsupported firmware flavor strings report the exact upstream support message `--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware`.
- Tests prove enabled `wipe` reports the exact upstream wipe incompatibility message `--use-firmware-retraction is not compatible with --wipe`.
- Tests prove unsupported-flavor message (`--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware`) remains when both unsupported flavor and enabled wipe apply, matching `std::map::emplace` first-insert-wins behavior for the shared `use_firmware_retraction` key.
- Tests prove arbitrary unknown `gcode_flavor` strings are deferred to `PrintConfig.cpp:10147-10150` enum validation: they do not return `SliceError::InvalidInput` and do not produce a M197 firmware-retraction compatibility error.
- Tests prove malformed bool/string boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196 `validate_basic_fdm_options` behavior remains intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10147+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
