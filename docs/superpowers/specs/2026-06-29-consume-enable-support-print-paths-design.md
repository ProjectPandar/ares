# Consume `enable_support` In Current Support Print Paths

## Goal

Make the already-parsed `enable_support` option control Ares' existing support print-path proxies. When support is disabled, current finalized output must drop support-only proxy paths; when support is enabled, existing support proxy behavior must remain unchanged.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948`: `enable_support` is a `PrintObjectConfig` boolean field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5903-5908`: `enable_support` is a boolean Support option labeled "Enable support" with default `false`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:124-130`: support-enabled objects participate in support layer-height bounds.
- `OrcaSlicer/src/libslic3r/Print.hpp:429-431`: object `has_support()` is true when `enable_support` is true or enforced support layers are present; support material also includes raft support material.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.hpp:28`: support generation checks `enable_support` or enforced support layers.
- `OrcaSlicer/src/libslic3r/Print.cpp:1456-1457`: support-disabled objects skip support-gap validation in a prime-tower path.
- `OrcaSlicer/src/libslic3r/Print.cpp:2291-2300` and `2336-2341`: Orca routes objects through support material generation during the support-material stage.

## Current Ares Boundary

- `crates/ares-core/src/options/support_enable.rs` parses `enable_support` with Orca's default `false` and currently only consumes it as runtime state.
- `crates/ares-core/src/pipeline.rs` validates `enable_support` before model loading.
- `crates/ares-core/src/print_paths/generate.rs` finalizes Ares print paths and already applies support-specific proxy transformations after generic print-path assembly.
- `crates/ares-core/src/print_paths.rs` currently has only two support-specific print roles, `SupportMaterial` and `SupportMaterialInterface`. Support-interface ironing is represented as `PrintPathRole::Ironing` with `extrusion_role() == Some(PrintPathRole::SupportMaterialInterface)`.
- Ares does not yet have real Orca support generation, `PrintObjectConfig` objects, support blockers/enforcers, raft semantics, or per-object support state.

This slice treats the current support roles as the only support material that Ares can safely gate. It does not create new support geometry, infer overhangs, or model Orca's full `has_support()` expression.

## Selected Approach

Filter existing finalized support proxy paths when `enable_support` is false.

Alternatives rejected:

1. Generate real supports from overhang polygons when `enable_support` is true. This is the eventual Orca parity target, but it crosses into support generator ownership and is too broad for the current "consume existing options first" pass.
2. Gate all support-related validation and support option parsing behind `enable_support`. Orca still validates and stores configuration independently, and Ares must continue rejecting malformed support options at API boundaries.
3. Model `enforce_support_layers` and raft coupling in this slice. Upstream includes those in `has_support()` and support-material state, but Ares does not currently have compatible forced-support or raft support-material semantics.

## Included Behavior

1. Continue parsing and validating `enable_support` before model loading through `SliceOptions::support_enable_options()`.
2. Add the support-role gate as the last step inside `finalize_print_paths`, after `apply_support_ironing`, using `options.support_enable_options()?.enabled()` so all existing support-option parsing and validation still runs before disabled support roles are removed.
3. When `enable_support` is true, preserve current support material, support interface, support interface spacing, support ironing, toolpath, extrusion, speed, diagnostics, and G-code behavior.
4. When `enable_support` is false or omitted, remove finalized paths whose role is `PrintPathRole::SupportMaterial` or `PrintPathRole::SupportMaterialInterface`.
5. When `enable_support` is false or omitted, remove support-interface ironing proxy paths represented as `PrintPathRole::Ironing` with `extrusion_role() == Some(PrintPathRole::SupportMaterialInterface)`.
6. Preserve every non-support path, including ordinary ironing paths with no support-interface extrusion role.
7. Keep support-specific option validation active even when support is disabled, including existing `support_top_z_distance`, `support_bottom_z_distance`, `enforce_support_layers`, `support_interface_spacing`, `support_bottom_interface_spacing`, support pattern, support ironing, support placement, support threshold, support type, and support style validation.
8. Keep the implementation inside `ares-core`, platform-neutral, and compatible with WASM.

## Test Impact

This changes the default option behavior for any fixture that manually injects current support proxy roles and omits `enable_support`. Support behavior tests whose purpose is still to exercise support material, support interface, or support ironing must opt in by setting `enable_support: true` in their local option maps or helper defaults. Disabled-support tests must intentionally omit the option or set `enable_support: false` and assert that support roles are absent.

The implementation plan must audit and update known existing positive support-role fixtures in these files, then re-check the final diff with a support-role search so newly noticed fixtures are not missed:

- `crates/ares-core/src/pipeline/tests/filament_ironing_inset.rs`
- `crates/ares-core/src/pipeline/tests/ironing_angle.rs`
- `crates/ares-core/src/pipeline/tests/ironing_inset.rs`
- `crates/ares-core/src/pipeline/tests/ironing_pattern.rs`
- `crates/ares-core/src/pipeline/tests/ironing_spacing.rs`
- `crates/ares-core/src/pipeline/tests/ironing_type_paths.rs`
- `crates/ares-core/src/pipeline/tests/support_angle.rs`
- `crates/ares-core/src/pipeline/tests/support_base_pattern.rs`
- `crates/ares-core/src/pipeline/tests/support_base_pattern_spacing.rs`
- `crates/ares-core/src/pipeline/tests/support_bottom_interface_spacing.rs`
- `crates/ares-core/src/pipeline/tests/support_expansion.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_loop_pattern.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_pattern.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_pattern_gcode.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_spacing.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_speed_flow.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_speed_flow/support_line_width.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_top_layers_runtime.rs`
- `crates/ares-core/src/pipeline/tests/support_ironing_paths.rs`
- `crates/ares-core/src/pipeline/tests/support_ironing_pattern.rs`
- `crates/ares-core/src/pipeline/tests/support_ironing_role_fan_gcode.rs`
- `crates/ares-core/src/pipeline/tests/support_ironing_spacing.rs`
- `crates/ares-core/src/pipeline/tests/support_speed_flow.rs`
- `crates/ares-core/src/pipeline/tests/support_speed_flow/support_line_width.rs`
- `crates/ares-core/src/pipeline/tests/support_z_distance.rs`

## Deferred Behavior

- Real Orca support generation from overhang polygons.
- Orca support blockers/enforcers, manual/automatic support regions, critical regions, small-overhang removal, and support threshold geometry.
- `enforce_support_layers` as part of `has_support()`.
- Raft support material as part of `has_support_material()`.
- Object-level support state, per-object configuration, multi-object propagation, invalidation graph behavior, and `PrintApply` support-used mutation.
- Tree/organic support generation.
- UI, CLI, WASM API changes, new option definitions, and registry changes.
- Orca binary E2E geometry parity for support generation.

## Acceptance Criteria

1. Invalid `enable_support` values still fail before model loading with `SliceError::InvalidInput` mentioning `enable_support`.
2. Existing current support proxy output is preserved when `enable_support` is true.
3. Support material and support material interface proxy paths are absent from finalized print paths, toolpath moves, extrusion moves, speed moves, diagnostics, and emitted G-code when `enable_support` is false or omitted.
4. Support-interface ironing proxy paths are absent when `enable_support` is false or omitted.
5. Ordinary non-support ironing paths are preserved when `enable_support` is false or omitted.
6. Invalid support-specific options, including support z-distance and `enforce_support_layers`, still fail even when `enable_support` is false because the filter runs only after existing support-specific finalization and validation.
7. No Rust source file touched by the implementation exceeds the project 400 LOC split threshold.

## Verification

- Update focused tests in `crates/ares-core/src/pipeline/tests/support_enable.rs`.
- Add focused print-path tests for the support-role filter if direct unit coverage is cleaner than only pipeline coverage.
- `cargo nextest run -p ares-core support_enable`
- `cargo nextest run -p ares-core support_z_distance`
- `cargo nextest run -p ares-core support_ironing_paths`
- `cargo nextest run -p ares-core support_interface_spacing support_base_pattern_spacing`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `enable_support` now removes current Ares support proxy print paths when false or omitted, while real support generation, forced supports, raft support material, per-object state, and support-used propagation remain deferred.
