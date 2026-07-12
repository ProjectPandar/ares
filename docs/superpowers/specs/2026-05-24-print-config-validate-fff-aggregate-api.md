# M207 Spec: PrintConfig validate FFF aggregate API

## Goal
Port OrcaSlicer's source-order aggregation semantics for `Slic3r::validate(const FullPrintConfig&, bool under_cli)` into a single Ares FFF validation API that composes the already source-cited M196-M206 validation slice APIs.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10308`: full FFF `validate(const FullPrintConfig &cfg, bool under_cli)` function, including source-order error insertion and final return.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10207-10235`: `under_cli` only affects the spiral-vase block.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10237-10303`: extrusion-width block runs before generic numeric range block; same-key later range errors are suppressed by source `error_message.find(opt_key) == error_message.end()`.
- M196-M206 milestone docs: already-ported option-definition/function contexts for each validation slice being composed.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8629-8647` `DynamicPrintConfig::validate` printer-technology dispatch and `FullPrintConfig fpc; fpc.apply(*this, true)` materialization.
- SLA / non-FFF empty validation return behavior from `DynamicPrintConfig::validate`.
- Generic range validation for numeric keys/types not already covered by M205-M206.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- Create `crates/ares-core/src/options/validation/aggregate.rs` with `SliceOptions::validate_fff_options(&self, under_cli: bool) -> Result<BTreeMap<String, String>, SliceError>`.
- Modify `crates/ares-core/src/options/validation.rs` to register `mod aggregate;`.
- Add `crates/ares-core/src/options/tests/validation/aggregate.rs` and register it from `crates/ares-core/src/options/tests/validation/mod.rs`.
- `docs/roadmap.md` and `docs/milestones/m207-print-config-validate-fff-aggregate-api.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_fff_options(under_cli: bool) -> Result<BTreeMap<String, String>, SliceError>`.
2. Compose existing validation slice APIs in this exact source order:
   1. `validate_basic_fdm_options()` for `PrintConfig.cpp:10088-10128`.
   2. `validate_firmware_retraction_options()` for `PrintConfig.cpp:10131-10145`.
   3. `validate_gcode_flavor_option()` for `PrintConfig.cpp:10147-10150`.
   4. `validate_infill_pattern_options()` for `PrintConfig.cpp:10152-10170`.
   5. `validate_skirt_and_bridge_flow_options()` for `PrintConfig.cpp:10172-10185`.
   6. `validate_extruder_clearance_options()` for `PrintConfig.cpp:10187-10198`.
   7. `validate_filament_flow_ratio_options()` for `PrintConfig.cpp:10200-10205`.
   8. `validate_spiral_vase_cli_options()` only when aggregate `under_cli` is true, for `PrintConfig.cpp:10207-10235`.
   9. `validate_extrusion_width_options()` for `PrintConfig.cpp:10237-10261`.
   10. `validate_line_width_range_options()` for `PrintConfig.cpp:10263-10303` line-width subset from M205-M206.
3. Merge each slice's returned `BTreeMap` into the aggregate using first-write-wins semantics: if a key already exists, keep the earlier source-order value.
4. Propagate `SliceError::InvalidInput` from the first slice that fails; do not hide malformed input errors behind later checks.
5. Gate the existing M203 `validate_spiral_vase_cli_options()` call at the aggregate level: if `under_cli` is true, merge that slice; if false, skip that slice.
6. Defaults must validate to an empty map.
7. Non-CLI aggregate validation must not report spiral-vase CLI-only errors.
8. If `validate_extrusion_width_options` and `validate_line_width_range_options` both report the same key, the aggregate must keep the extrusion-width message because it appears earlier in Orca source order.
9. Preserve all existing standalone M196-M206 validation APIs unchanged.
10. Do not add `DynamicPrintConfig::validate` printer-technology dispatch, `FullPrintConfig` materialization, SLA behavior, generic validation for unported numeric keys/types, slicing, extrusion, G-code behavior, new crates, or dependencies.
11. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove defaults return an empty aggregate validation map.
- Tests prove one input can accumulate representative errors from basic, firmware retraction, gcode flavor, infill pattern, skirt/bridge flow, extruder clearance, filament flow ratio, spiral-vase CLI, extrusion-width, and line-width range slices.
- Tests prove first-write-wins source ordering for duplicate keys, especially extrusion-width message before later line-width range message for the same key.
- Tests prove `under_cli = false` suppresses spiral-vase CLI-only errors while preserving other errors.
- Tests prove malformed input returns `SliceError::InvalidInput` through the aggregate.
- Tests prove standalone M196-M206 APIs remain callable and unchanged.
- Plan/spec explicitly account for deferred `DynamicPrintConfig::validate`, `FullPrintConfig` materialization, SLA behavior, and unported generic numeric validation.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
