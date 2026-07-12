# Consume Support Interface Top Layers Runtime Design

## Objective

Consume `support_interface_top_layers` in concrete Ares runtime behavior for support-interface paths that already exist in `LayerPrintPaths`. This is a source-cited `libslic3r` rewrite slice: when top interface layers are disabled, Ares must stop treating existing support-interface paths as support-interface extrusion and route them through base support material behavior instead.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:964` declares `support_interface_top_layers` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6072-6088` defines `support_interface_top_layers` as an integer option, default `3`, minimum `0`, with GUI values `0..3`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:28-32` normalizes `num_top_interface_layers = max(0, support_interface_top_layers)` and sets `has_top_contacts` from whether that value is positive.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:70-73` changes support-interface flow to base support flow when `support_interface_top_layers == 0`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:116-120` changes top-interface spacing/density to base support spacing/density when `support_interface_top_layers == 0`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:145-167` only collects top interface contact projection when `num_top_interface_layers > 0`.

## Current Ares State

- `crates/ares-core/src/print_paths.rs` already has `PrintPathRole::SupportMaterial` and `PrintPathRole::SupportMaterialInterface`.
- `crates/ares-core/src/print_paths/generate.rs` already has `finalize_print_paths`, which is the shared final pass before toolpath, extrusion, speed, and G-code generation.
- `crates/ares-core/src/pipeline/test_support.rs` can inject support-interface paths through `single_path_pipeline`, and existing tests prove that support-interface speed, flow, filament, fan, and support ironing affect concrete G-code.
- `crates/ares-core/src/options/tests/registry_lookup_support_interface_layers_spacing.rs` already exposes metadata/defaults for `support_interface_top_layers`, but no runtime behavior currently consumes it.
- `crates/ares-core/src/options.rs` is already 400 LOC, so this slice must not add methods there.

## Requirements

1. Parse `support_interface_top_layers` from `SliceOptions::values()` in a new small `print_paths` support-interface finalization module. Accept omitted values as Orca default `3`.
2. Accept JSON integer numbers, float-encoded integers, and numeric strings that represent finite non-negative integers. Reject negative, fractional, non-finite, percent, boolean, null, array, and object values with `SliceError::InvalidInput` mentioning `support_interface_top_layers`.
3. In `finalize_print_paths`, before support ironing is applied, rewrite every `PrintPathRole::SupportMaterialInterface` path to `PrintPathRole::SupportMaterial` when parsed `support_interface_top_layers == 0`.
4. Preserve path geometry and metadata during the rewrite: points, effective layer height, unsupported span, seam gap, and closed/open state must remain unchanged.
5. When `support_interface_top_layers > 0` or omitted, preserve existing support-interface behavior exactly, including support-interface speed/flow/fan behavior and support ironing duplication.
6. When `support_interface_top_layers == 0`, concrete G-code for an injected support-interface path must use `support_material` role behavior: support speed/flow and the base layer fan baseline apply, support-interface fan/speed/flow role overrides no longer apply, and support ironing must not duplicate the rewritten path.
7. Keep implementation platform-neutral and WASM-safe: no file I/O, terminal behavior, UI behavior, native viewer code, OpenGL, or new dependencies in `ares-core`.
8. Keep every touched Rust file at or below 400 LOC.

## Out Of Scope

- Full support generation and contact projection remain deferred.
- `support_interface_bottom_layers` runtime behavior remains deferred.
- Support interface spacing, pattern, loop pattern, bottom-interface spacing, support material contact geometry, base-interface layers, soluble interface behavior, and support extruder/tool ordering remain deferred.
- Option registry metadata is not expanded in this slice.
- `options.rs` is not extended; parsing stays in the `print_paths` runtime boundary.

## Acceptance Criteria

- RED: focused tests fail before implementation because `support_interface_top_layers = 0` still emits `support_material_interface` G-code and support ironing still duplicates it.
- GREEN: `cargo nextest run -p ares-core support_interface_top_layers_runtime` passes after implementation.
- Tests prove omitted/default top interface layers preserve existing `SupportMaterialInterface` G-code.
- Tests prove `support_interface_top_layers = 0` rewrites support-interface paths to support material in concrete G-code, uses support speed/flow instead of support-interface speed/flow, and uses the base layer fan baseline instead of the support-interface fan override.
- Tests prove `support_interface_top_layers = 0` prevents support ironing duplication because no support-interface path remains for the ironing pass.
- Tests prove invalid top-layer values return `SliceError::InvalidInput` with the option key.
- Tests prove the path rewrite preserves points and path metadata.

## Verification

Before commit and push, run:

- `cargo fmt --check`
- `cargo nextest run -p ares-core support_interface_top_layers_runtime support_ironing_paths support_interface_speed_flow support_speed_flow`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- touched Rust file LOC guard

## Docs Impact

Update `docs/roadmap.md` after implementation to record that `support_interface_top_layers = 0` is now consumed for existing support-interface paths by routing them to base support material behavior, while full `SupportCommon.cpp` support contact generation and adjacent support-interface options remain deferred.
