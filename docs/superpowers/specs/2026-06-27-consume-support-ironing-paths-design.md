# Consume Support Ironing Paths Design

## Source Boundary

This slice ports a narrow runtime part of OrcaSlicer support ironing from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997-1000` for `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, and `support_ironing_spacing`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6449` for defaults and meaning: support ironing is disabled by default; when enabled it prints support interface again as ironing with small flow; default support ironing flow is `10%`; default support ironing spacing is `0.1mm`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1212-1215` for the support-material invalidation boundary affected by these options.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:58-61` for moving `object_config.support_ironing`, `support_ironing_flow`, `support_ironing_spacing`, and `support_ironing_pattern` into support generation parameters.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1635` for caching top contact-layer polygons when support ironing is enabled.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1879-1907` for generating support ironing fill paths with the selected pattern, angle, spacing, clipping, and `ExtrusionRole::erIroning`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6115-6140` for the emitted role vocabulary where `erIroning` in support extrusion is labeled `"support ironing"`.

## Rust Destination

The Rust destination is `ares-core` runtime path assembly:

- `crates/ares-core/src/options/support_ironing.rs`: parse `support_ironing` as a boundary boolean and reject non-boolean values.
- `crates/ares-core/src/print_paths/support_ironing.rs`: when support ironing is enabled, duplicate existing `PrintPathRole::SupportMaterialInterface` paths as `PrintPathRole::Ironing` paths on the same layer after the source support-interface path.
- `crates/ares-core/src/print_paths/generate.rs`: expose a final path pass over already assembled `Vec<LayerPrintPaths>`.
- `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs`: use the same final path pass after normal path generation and gap-fill filtering, before toolpath, extrusion, speed, and G-code generation. Tests inject existing support-interface paths through `pipeline::test_support` because normal Ares path generation does not yet create support material.
- `crates/ares-core/src/pipeline/tests/support_ironing_paths.rs`: prove concrete G-code and extrusion behavior through nextest.

## Included Behavior

- `support_ironing` defaults to `false`.
- When `support_ironing` is `true`, every existing `SupportMaterialInterface` print path produces one additional `Ironing` print path with the same points, print Z, layer height override, and open/closed state.
- The extra ironing path reaches concrete G-code as `;EXTRUSION:print:ironing:...` and uses the existing ironing speed, ironing flow, filament ironing speed, filament ironing flow, fan, and hardware behavior already implemented in Ares.
- The original support-interface path is preserved and still emits before the duplicated ironing path.
- Non-boolean `support_ironing` values fail at the slicing/options boundary with an error mentioning `support_ironing`.

## Compatibility Boundary

Orca generates support ironing from cached top contact-layer polygons using `Fill::new_from_type`, support ironing spacing, clipping against upper support islands, and `ExtrusionRole::erIroning`. Ares does not yet own that full support-generation source boundary. This slice is therefore a deliberate compatibility shell around the upstream concept: it consumes `support_ironing` only for `SupportMaterialInterface` paths that already exist in `LayerPrintPaths`, adds a concrete ironing print path, and wires that path through the existing extrusion/speed/G-code runtime. Later support-generation slices must replace this path duplication with the direct `SupportCommon.cpp:1879-1907` fill-generation port.

## Deferred Behavior

- Full support generation remains deferred; this slice only consumes support ironing for support-interface paths that already exist in Ares.
- `support_ironing_pattern` and `support_ironing_spacing` geometric line generation remain deferred until Ares ports the relevant support-interface fill geometry.
- `support_ironing_flow` as a distinct flow ratio remains deferred; the duplicated path uses Ares' existing ironing flow channel for this slice because Ares currently has one `PrintPathRole::Ironing` runtime role.
- Orca's exact `support ironing` label string inside support-specific `GCode::extrude_support` remains deferred because Ares currently emits role names through the unified `PrintPathRole::as_str()` G-code diagnostics.
- Complete support-material invalidation graph parity remains deferred; Ares' in-memory pipeline is recomputed per slice call.

## Acceptance Criteria

- RED: adding focused tests for `support_ironing` causes `cargo nextest run -p ares-core support_ironing_paths` to fail because no extra ironing extrusion is emitted.
- GREEN: after implementation, the same focused command passes.
- Focused tests prove disabled/default behavior preserves one support-interface extrusion and enabled behavior emits support-interface then ironing extrusion.
- Focused tests prove ironing speed/flow affect the duplicated support-ironing path through existing G-code feedrate and E-delta output.
- Focused tests prove the duplicated path preserves the source path points, layer ID, print Z, effective layer height override, unsupported span metadata, seam-gap metadata, and open/closed state while changing only the role to `Ironing`.
- Focused tests prove invalid non-boolean `support_ironing` values return `SliceError::InvalidInput` with the option key.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.

## Docs Impact

- Update `docs/roadmap.md` after implementation to record that `support_ironing` now has concrete runtime behavior for existing support-interface paths and that full `SupportCommon.cpp` support-ironing fill generation, pattern, spacing, and distinct support-ironing flow remain deferred.
- No architecture decision record is required because this slice keeps existing crate boundaries and only adds an `ares-core` path finalization pass.

## Safety And Constraints

- No new dependencies.
- No filesystem, terminal, UI, OpenGL, or native-only behavior in `ares-core`.
- Touched Rust files must stay at or below 400 LOC.
- Existing Ares scaffolding remains a compatibility shell around the upstream `libslic3r` support-ironing concept until full support generation is ported.
