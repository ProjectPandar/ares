# Consume `ironing_type` Into Ordinary Ironing Paths

## Context

This slice ports the first executable part of OrcaSlicer ordinary ironing selection into Ares. It is a source-cited Rust rewrite slice, not a new Ares-owned ironing feature.

Upstream boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:100-105` defines `IroningType::{NoIroning, TopSurfaces, TopmostOnly, AllSolid, Count}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1151` places `ironing_type` with ordinary ironing pattern, flow, spacing, inset, speed, angle, and filament override options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:257-263` maps config strings to `IroningType`: `"no ironing"`, `"top"`, `"topmost"`, and `"solid"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4161-4176` registers `ironing_type`, labels the modes, and defaults to `NoIroning`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1499-1720` implements `Layer::make_ironing`, including type gating, top/topmost/all-solid selection, and final `erIroning` extrusion entities.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:699-715` runs ironing over every object layer.

Current Ares state:

- `crates/ares-core` already has `PrintPathRole::Ironing` and G-code output for Ironing-role paths.
- `ironing_speed`, `filament_ironing_speed`, `ironing_flow`, `filament_ironing_flow`, and `ironing_fan_speed` already affect existing Ironing paths.
- `support_ironing` already duplicates support-interface paths into Ironing paths.
- Ordinary `ironing_type` is still only metadata/config surface; it does not yet generate ordinary Ironing paths from top or solid infill.

## Design

Add a narrow `ironing_type` runtime selection pass after normal print paths are generated and filtered, before support ironing is added.

The pass parses `ironing_type` from `SliceOptions` with Orca's exact string values:

- omitted or `"no ironing"`: no ordinary Ironing paths
- `"top"`: duplicate current Ares `TopSolidInfill` paths as `Ironing`
- `"topmost"`: duplicate current Ares `TopSolidInfill` paths only on the final print layer as `Ironing`
- `"solid"`: duplicate current Ares ordinary solid-area paths as `Ironing`, limited to `TopSolidInfill`, `SolidInfill`, and `BottomSurface`

Duplicated ordinary Ironing paths are appended after the source paths within each layer so ironing is emitted after the original solid/top infill for that layer. The duplicate keeps the source geometry, effective layer height, unsupported-span metadata, seam-gap metadata, and closed/open shape state, but has role `PrintPathRole::Ironing` and no support-specific `extrusion_role` override. Existing Ironing speed, flow, fan, and G-code channels then consume the generated path.

Invalid `ironing_type` values return `SliceError::InvalidInput` naming `ironing_type`. No legacy aliases are accepted.

## Ares Destination

- Add `crates/ares-core/src/options/ironing_type.rs` for parsing the ordinary ironing enum.
- Add `crates/ares-core/src/print_paths/ironing.rs` for ordinary path duplication.
- Wire the pass from `crates/ares-core/src/print_paths/generate.rs::finalize_print_paths`.
- Add focused pipeline tests under `crates/ares-core/src/pipeline/tests/ironing_type_paths.rs` and register them in `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md` after implementation review to record the consumed behavior and deferred Orca parity.

## Deferred Behavior

This slice does not implement full `Fill::make_ironing` geometry generation. The following upstream behavior remains deferred:

- `ironing_pattern`, `ironing_spacing`, `ironing_inset`, `ironing_direction`, `ironing_angle`, `ironing_angle_fixed`, `filament_ironing_spacing`, and `filament_ironing_inset`
- polygon union/offset/intersection against layer slices
- new ironing line filling from `Fill::fill_surface`
- extruder-grouped region merging and multi-extruder ownership beyond Ares' current single active path ownership
- Orca's whole-face versus just-infill distinction
- complete sparse-vs-solid surface classification beyond Ares' current `PrintPathRole` output
- support ironing fill generation, support ironing pattern/spacing, and support-specific G-code label parity
- Orca binary E2E geometry parity

## Acceptance Criteria

- Focused RED/GREEN nextest coverage proves that `ironing_type` changes actual generated paths and G-code, not only option parsing.
- Omitting `ironing_type` or setting `"no ironing"` emits no ordinary Ironing paths for a normal rectangular top surface.
- `"top"` emits ordinary Ironing G-code for generated `TopSolidInfill` and emits it after the source top-solid infill in the same layer.
- `"topmost"` emits ordinary Ironing only on the last layer when multiple top shell layers exist.
- `"solid"` emits ordinary Ironing for Ares' current solid-area roles: bottom surface, internal solid infill, and top solid infill.
- Ordinary `ironing_type` does not duplicate support-interface paths; `support_ironing` remains the support-interface gate.
- Invalid values such as `true`, `null`, arrays, objects, `"TopSurfaces"`, and `"top surfaces"` return `SliceError::InvalidInput` mentioning `ironing_type`.
- Verification uses `cargo nextest run`, never `cargo test`.
- Full verification before commit includes `cargo fmt --check`, focused nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.
- Every touched Rust file remains at or below 400 LOC.
