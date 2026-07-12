# M203 Spec: PrintConfig validate spiral vase CLI constraints

## Goal
Port OrcaSlicer's CLI-only spiral-vase validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_spiral_vase_cli_options()`, returning validation messages for this source slice without adding full validation dispatch, UI correction behavior, or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10207-10235`: the `cfg.spiral_mode && under_cli` validation block for `wall_loops`, `sparse_infill_density`, `top_shell_layers`, `enable_support`, and `enforce_support_layers`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2881-2889`, `4918-4924`, `5678-5684`, `5903-5908`, `6013-6025`, and `6564-6573`: option-definition/default context for the checked keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948`, `958`, `1101`, `1158`, `1167`, and `1560`: `FullPrintConfig` field type context.

Related upstream behavior explicitly deferred:

- Non-CLI spiral-mode popup correction behavior mentioned by the source comment.
- `PrintConfig.cpp:10237+` extrusion-width and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation/spiral_vase.rs`: add `SliceOptions::validate_spiral_vase_cli_options(&self) -> Result<BTreeMap<String, String>, SliceError>`.
- `crates/ares-core/src/options/validation.rs`: register the new validation submodule.
- `crates/ares-core/src/options/tests/validation/spiral_vase.rs`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m203-print-config-validate-spiral-vase-cli.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_spiral_vase_cli_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing keys use source-cited registry defaults; default `spiral_mode == false` returns no errors, while `spiral_mode == true` with missing constrained keys validates the source-cited defaults.
3. If `spiral_mode == false`, return no errors from this source slice even when the constrained keys have conflicting values.
4. If `spiral_mode == true`, validate only this source slice:
   - `wall_loops != 1` inserts `wall_loops` with `Invalid value when spiral vase mode is enabled: {value}`.
   - `sparse_infill_density > 0` inserts `sparse_infill_density` with `Invalid value when spiral vase mode is enabled: {value:.6}` to match `std::to_string(double)` formatting.
   - `top_shell_layers > 0` inserts `top_shell_layers` with `Invalid value when spiral vase mode is enabled: {value}`.
   - `enable_support == true` inserts `enable_support` with `Invalid value when spiral vase mode is enabled: 1`, matching C++ bool-to-integer formatting through `std::to_string` overload resolution.
   - `enforce_support_layers > 0` inserts `enforce_support_layers` with `Invalid value when spiral vase mode is enabled: {value}`.
5. JSON malformed bool/int/float values return `SliceError::InvalidInput`; numeric strings remain accepted for numeric keys to match existing Ares option-boundary behavior.
6. Preserve existing M196-M202 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
7. Do not add full validation dispatch, non-CLI UI correction behavior, extrusion-width checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove `spiral_mode == false` suppresses this source slice.
- Tests prove all five spiral-mode conflicts report exact keys and messages.
- Tests prove `spiral_mode == true` with missing constrained keys uses registry defaults and reports the default `wall_loops`, `sparse_infill_density`, and `top_shell_layers` conflicts.
- Tests prove numeric string values use the same predicates and message formatting for numeric options.
- Tests prove malformed boundary values return `SliceError::InvalidInput`.
- Tests prove existing M196/M197/M198/M199/M200/M201/M202 validation APIs remain intact.
- Plan/spec explicitly account for deferred non-CLI popup correction behavior, deferred `PrintConfig.cpp:10237+` validation behavior, and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
