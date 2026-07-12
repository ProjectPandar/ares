# M201 Spec: PrintConfig validate extruder clearance dimensions

## Goal
Port OrcaSlicer's extruder-clearance validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_extruder_clearance_options()`, returning validation messages for this source slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10187-10198`: `extruder_clearance_radius`, `extruder_clearance_height_to_rod`, `extruder_clearance_height_to_lid`, and `nozzle_height` validation/error insertion.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2127-2160` and `PrintConfig.hpp:1513-1516`: option-definition/default context.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10200+` filament-flow, spiral-vase, and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_extruder_clearance_options(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private helper reuse if needed.
- `crates/ares-core/src/options/tests/validation/`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m201-print-config-validate-extruder-clearance.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_extruder_clearance_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing clearance/nozzle keys use source-cited registry defaults and return no errors.
3. If `extruder_clearance_radius <= 0`, report key `extruder_clearance_radius` with message `invalid value {value:.6}`, matching C++ `std::to_string(double)` formatting for this source slice.
4. If `extruder_clearance_height_to_rod <= 0`, report key `extruder_clearance_height_to_rod` with message `invalid value {value:.6}`.
5. If `extruder_clearance_height_to_lid <= 0`, report key `extruder_clearance_height_to_lid` with message `invalid value {value:.6}`.
6. If `nozzle_height <= 0`, report key `nozzle_height` with message `invalid value {value:.6}`.
7. JSON non-number/non-numeric-string values for these float options return `SliceError::InvalidInput`; numeric strings remain accepted to match existing Ares numeric option boundary behavior.
8. Preserve existing M196-M200 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
9. Do not add full validation dispatch, filament-flow checks, spiral-vase checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
10. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove zero and negative clearance/nozzle values report exact six-decimal messages under their own keys.
- Tests prove numeric string clearance/nozzle values use the same predicate and message formatting.
- Tests prove malformed boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196/M197/M198/M199/M200 validation APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10200+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
