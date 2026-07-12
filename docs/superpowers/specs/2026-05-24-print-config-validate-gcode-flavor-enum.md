# M198 Spec: PrintConfig validate gcode flavor enum value

## Goal
Port OrcaSlicer's `gcode_flavor` enum validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_gcode_flavor_option()`, returning validation messages for this enum-value slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10147-10150`: `gcode_flavor` enum validation through `print_config_def.get("gcode_flavor")->has_enum_value(cfg.gcode_flavor.serialize())`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:161-176`: string mapping for `GCodeFlavor` values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:33-46`: `GCodeFlavor` enum declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785-3817` and `PrintConfig.hpp:1355`: option-definition context; active `def->enum_values` here are the validation allow-list used by `has_enum_value`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10131-10145` firmware-retraction compatibility was implemented by M197 and must not be duplicated into this API.
- `PrintConfig.cpp:10152+` fill-pattern and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_gcode_flavor_option(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private flavor helper if needed.
- `crates/ares-core/src/options/tests/validation.rs`: add source-behavior tests.
- `docs/roadmap.md` and `docs/milestones/m198-print-config-validate-gcode-flavor-enum.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_gcode_flavor_option()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing `gcode_flavor` uses the source-cited registry default and returns no errors.
3. Accept exactly the active `gcode_flavor` option enum values from `PrintConfig.cpp:3785-3817`: `marlin`, `klipper`, `reprapfirmware`, `repetier`, and `marlin2`.
4. If `gcode_flavor` is any other string, including mapped-but-commented-out strings from `PrintConfig.cpp:3785-3817`, report key `gcode_flavor` with message `invalid value {value}`, matching `L("invalid value ") + cfg.gcode_flavor.serialize()` for Ares' JSON string boundary.
5. JSON non-string `gcode_flavor` values return `SliceError::InvalidInput`.
6. Preserve existing `validate_basic_fdm_options`, `validate_firmware_retraction_options`, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
7. Do not add full validation dispatch, firmware-retraction behavior, fill-pattern checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent `gcode_flavor` returns an empty validation map.
- Tests prove every active `gcode_flavor` option enum value passes.
- Tests prove unknown strings report exact message `invalid value unknown-firmware` for key `gcode_flavor`.
- Tests prove mapped-but-commented-out Orca strings such as `reprap`, `teacup`, `makerware`, `sailfish`, `smoothie`, `mach3`, `machinekit`, and `no-extrusion` are rejected with `invalid value {value}`.
- Tests prove non-string JSON boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196 `validate_basic_fdm_options` behavior remains intact.
- Tests prove existing M197 `validate_firmware_retraction_options` behavior remains intact, including unknown `gcode_flavor` deferral in that API.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10152+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
