# M202 Spec: PrintConfig validate filament flow ratio

## Goal
Port OrcaSlicer's filament-flow-ratio validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_filament_flow_ratio_options()`, returning validation messages for this source slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10200-10205`: iterates `cfg.filament_flow_ratio.values`, inserts `filament_flow_ratio` error when any value is `<= 0`, then breaks.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227-2237` and `PrintConfig.hpp:1301`: option-definition/default context for `filament_flow_ratio`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10207+` spiral-vase and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs` and/or `crates/ares-core/src/options/validation/`: split the current validation implementation so all modified Rust files remain under 400 LOC while preserving existing public methods.
- Add `SliceOptions::validate_filament_flow_ratio_options(&self) -> Result<BTreeMap<String, String>, SliceError>`.
- `crates/ares-core/src/options/tests/validation/`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m202-print-config-validate-filament-flow-ratio.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_filament_flow_ratio_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing `filament_flow_ratio` uses the source-cited registry default and returns no errors.
3. If any parsed `filament_flow_ratio` value is `<= 0`, report key `filament_flow_ratio` with message `invalid value {serialized_vector}`.
4. Preserve existing Ares/M196 numeric-vector serialization semantics: join parsed `f64` values with comma using Rust `ToString::to_string()`, not six-decimal scalar formatting.
5. Accept all existing numeric-vector boundary forms supported by `parse_numeric_vector`: JSON number, numeric string, JSON array of numbers/strings, comma-separated string, and semicolon-separated string.
6. Malformed vector values return `SliceError::InvalidInput` from the existing numeric-vector parser.
7. Preserve existing M196-M201 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
8. Do not add full validation dispatch, spiral-vase checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
9. Split `options::validation` implementation before adding new logic if needed so modified Rust files remain under 400 LOC.

## Acceptance checks

- Tests prove default/absent `filament_flow_ratio` returns an empty validation map.
- Tests prove zero and negative entries report exact serialized-vector messages under `filament_flow_ratio`.
- Tests prove accepted vector boundary forms use the same predicate and serialization.
- Tests prove malformed boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196/M197/M198/M199/M200/M201 validation APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10207+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
