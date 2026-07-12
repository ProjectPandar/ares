# M199 Spec: PrintConfig validate infill pattern enum values

## Goal
Port OrcaSlicer's four fill-pattern enum validation blocks from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_infill_pattern_options()`, returning validation messages for this source slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10152-10170`: `sparse_infill_pattern`, `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` enum validation through `print_config_def.get(...)->has_enum_value(...serialize())`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1986-2025`: active option enum values for `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2985`: active option enum values for `sparse_infill_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-255` and `PrintConfig.hpp:87-98`: `InfillPattern` serialization enum context; this is context, not the per-option validation allow-list.
- `PrintConfig.hpp:1090-1092`, `PrintConfig.hpp:1102`, and existing Ares registry definitions provide option-definition/default context.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10131-10145` firmware-retraction compatibility was implemented by M197 and must not be duplicated into this API.
- `PrintConfig.cpp:10147-10150` `gcode_flavor` enum validation was implemented by M198 and must not be duplicated into this API.
- `PrintConfig.cpp:10172+` skirt-height, flow-ratio, and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_infill_pattern_options(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private per-option pattern helpers if needed.
- `crates/ares-core/src/options/tests/validation.rs` and/or `crates/ares-core/src/options/tests/validation/*`: split existing validation tests into submodules before adding M199 tests so each modified Rust file remains under 400 LOC, then add source-behavior tests.
- `docs/roadmap.md` and `docs/milestones/m199-print-config-validate-infill-pattern-enums.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_infill_pattern_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing pattern option keys use source-cited registry defaults and return no errors.
3. Validate `sparse_infill_pattern` against exactly the active enum values from `PrintConfig.cpp:2928-2985`: `rectilinear`, `alignedrectilinear`, `zigzag`, `crosszag`, `lockedzag`, `line`, `grid`, `triangles`, `tri-hexagon`, `cubic`, `adaptivecubic`, `quartercubic`, `supportcubic`, `lightning`, `honeycomb`, `3dhoneycomb`, `lateral-honeycomb`, `lateral-lattice`, `crosshatch`, `tpmsd`, `tpmsfk`, `gyroid`, `concentric`, `hilbertcurve`, `archimedeanchords`, and `octagramspiral`.
4. Validate `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` against exactly the active enum values from `PrintConfig.cpp:1986-2025`: `monotonic`, `monotonicline`, `rectilinear`, `alignedrectilinear`, `concentric`, `hilbertcurve`, `archimedeanchords`, and `octagramspiral`.
5. If any pattern option is any other string, report that option key with message `invalid value {value}`, matching `L("invalid value ") + cfg.<option>.serialize()` for Ares' JSON string boundary.
6. JSON non-string values for these pattern options return `SliceError::InvalidInput`.
7. Preserve existing `validate_basic_fdm_options`, `validate_firmware_retraction_options`, `validate_gcode_flavor_option`, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
8. Do not add full validation dispatch, skirt-height checks, flow checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
9. Keep modified Rust files under 400 LOC; split validation tests into submodules if needed.

## Acceptance checks

- Tests prove default/absent pattern options return an empty validation map.
- Tests prove every active sparse infill pattern value passes.
- Tests prove every active top/bottom/internal solid surface pattern value passes for all three surface-pattern keys.
- Tests prove a value active only for sparse infill, such as `gyroid`, is rejected for `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` with `invalid value gyroid`.
- Tests prove arbitrary unknown strings report exact messages under their own option keys.
- Tests prove non-string JSON boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196/M197/M198 validation APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10172+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
