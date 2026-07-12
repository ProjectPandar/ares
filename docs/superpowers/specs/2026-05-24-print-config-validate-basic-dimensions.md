# M196 Spec: PrintConfig validate basic dimension and count checks

## Goal
Port the first validation block from OrcaSlicer's `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_basic_fdm_options()`, returning validation messages for the initial FFF scalar/vector checks without adding full validation or slicing behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10128`: creation of `error_message` and checks for `layer_height`, `initial_layer_print_height`, `filament_diameter`, `nozzle_diameter`, `wall_loops`, `top_shell_layers`, and `bottom_shell_layers`.
- `OrcaSlicer/src/libslic3r/libslic3r.h:60`: `SCALING_FACTOR_INTERNAL = 0.000001`, used by the layer-height modulus check through `SCALING_FACTOR`.
- Option-definition default anchors already carried in the Ares registry for these keys.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- `PrintConfig.cpp:10131+` firmware-retraction, enum, bridge-flow, clearance, extrusion multiplier, spiral-vase, extrusion-width, acceleration, skirt/brim/support, wipe-tower, machine-limit, and later validation checks.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_basic_fdm_options(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private parse/default helpers for this validation slice.
- `crates/ares-core/src/options.rs`: register the new module.
- `crates/ares-core/src/options/tests/validation.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m196-print-config-validate-basic-dimensions.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_basic_fdm_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing keys use source-cited registry defaults, matching the upstream `FullPrintConfig` default context.
3. Report `layer_height` when its upstream `get_abs_value("layer_height")` numeric value is `<= 0`.
4. Evaluate the exact upstream predicate `fabs(fmod(layer_height, SCALING_FACTOR)) > 1e-4`; `SCALING_FACTOR` is initialized from `SCALING_FACTOR_INTERNAL = 0.000001` (`libslic3r.h:60`). Because this predicate is not satisfiable by finite JSON `layer_height` values with that scaling factor, preserve the branch without changing threshold/scaling semantics or inventing a reachable invalid case.
5. Report `initial_layer_print_height` when its numeric value is `<= 0`.
6. Report `filament_diameter` when any vector member is `< 1`.
7. Report `nozzle_diameter` when any vector member is `< 0.005`.
8. Report `wall_loops` when it is `< 0`.
9. Report `top_shell_layers` when it is `< 0`.
10. Report `bottom_shell_layers` when it is `< 0`.
11. Messages may use English `invalid value {value}` text but must include the invalid value or serialized invalid vector, preserving key-level error-map behavior.
12. JSON boundary type errors return `SliceError::InvalidInput` rather than being added to the validation map.
13. Preserve existing `set_num_extruders`, `set_num_filaments`, parameter-size API, extruder-variant API, registry APIs, legacy normalization, and FDM normalization behavior.
14. Do not add full validation dispatch, firmware-retraction checks, enum checks, bridge-flow checks, clearance checks, spiral-vase checks, extrusion-width checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
15. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove invalid `layer_height <= 0` is reported.
- Tests prove the exact upstream layer-height modulus predicate `fabs(fmod(layer_height, SCALING_FACTOR)) > 1e-4` is preserved without inventing a reachable invalid case; with `SCALING_FACTOR = 0.000001`, representative finite off-grid JSON values do not report unless the exact upstream predicate evaluates true.
- Tests prove invalid `initial_layer_print_height <= 0` is reported.
- Tests prove invalid `filament_diameter` and `nozzle_diameter` vector members are reported.
- Tests prove negative `wall_loops`, `top_shell_layers`, and `bottom_shell_layers` are reported.
- Tests prove multiple invalid keys are accumulated into one map.
- Tests prove JSON type errors return `SliceError::InvalidInput`.
- Tests prove existing M194/M195 count APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10131+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
