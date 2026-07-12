# Consume Support Speed and Flow Design

## Objective

Consume the existing OrcaSlicer support-material speed and flow options in concrete Ares slicing/G-code behavior. This is a source-cited rewrite slice, not new option metadata: existing `support_speed` and `support_flow_ratio` inputs must affect constructed support-material print paths.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:974` declares `support_speed` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:980` declares `support_flow_ratio` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1384-1393` defines `support_flow_ratio` as a float, default `1`, range `0..=2`, for support material amount.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6195-6202` defines `support_speed` as a float, default `80`, minimum `1`, in `mm/s`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6428-6436` multiplies `erSupportMaterial` flow by `support_flow_ratio` under `set_other_flow_ratios`, then multiplies non-brim/non-skirt first-layer paths by `first_layer_flow_ratio`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6475-6479` resolves `erSupportMaterial` speed from `support_speed`.

## Current Ares State

- `crates/ares-core/src/extrusion_entity.rs` already has `ExtrusionRole::SupportMaterial`, but no `PrintPathRole::SupportMaterial` can currently construct that role through the existing print-path, move, extrusion, speed, and G-code pipeline.
- `PrintPathRole::SupportMaterialInterface` already consumes `support_interface_speed` and `support_interface_flow_ratio` from a previous slice.
- `crates/ares-core/src/options/flow_ratios.rs` still does not parse or apply `support_flow_ratio`.
- `crates/ares-core/src/options/speed.rs` still does not parse `support_speed`.
- `docs/roadmap.md` still records `support_flow_ratio` and support body speed behavior as deferred.
- `crates/ares-core/src/pipeline/tests.rs` is near the 400 LOC guard, so this slice must avoid growing it past the limit.

## Requirements

1. Add a constructed-path role for ordinary support material: `PrintPathRole::SupportMaterial`, with string name `support_material`, and map it to `ExtrusionRole::SupportMaterial`.
2. Keep support generation out of scope. The new role is only a compatibility shell for manually constructed support-material paths and future source-cited support-generation slices.
3. Parse `support_speed` through `ares-core` slice options as a finite numeric JSON number or numeric string, with Orca defaults and bounds: default `80.0`, minimum `1.0`, no percent syntax.
4. Route non-first-layer `PrintPathRole::SupportMaterial` print speed through the parsed `support_speed` value, producing matching G-code feedrate and speed comments.
5. Preserve Ares's existing first-layer support speed policy by treating `SupportMaterial` like support-interface/infill on first layer: first-layer support-material print moves continue to use `initial_layer_infill_speed`.
6. Parse `support_flow_ratio` as a finite numeric JSON number or numeric string, default `1.0`, range `0.0..=2.0`.
7. When `set_other_flow_ratios` is enabled, route `PrintPathRole::SupportMaterial` extrusion through `support_flow_ratio`.
8. When `set_other_flow_ratios` is omitted or disabled, validate `support_flow_ratio` but leave support-material extrusion unscaled by it, matching the existing Ares gate used for other flow-ratio options.
9. When `set_other_flow_ratios` is enabled on the first layer, compose `SupportMaterial` extrusion with `first_layer_flow_ratio`, matching Orca's non-brim/non-skirt first-layer flow rule and Ares's existing first-layer treatment for `SupportMaterialInterface`.
10. Do not change `SupportMaterialInterface` speed, flow, fan, or width behavior except where exhaustive `match` arms must include the new `SupportMaterial` role.
11. Keep the implementation platform-neutral and WASM-safe: no file I/O, terminal behavior, UI behavior, native viewer code, OpenGL, or new dependencies in `ares-core`.
12. Keep every touched Rust file at or below 400 LOC. If registering focused tests would push `pipeline/tests.rs` over the limit, split the pipeline test module declarations before adding the new test module.

## Out Of Scope

- Full support generation.
- Support transition paths or `erSupportTransition` parity.
- `support_base_pattern`, `support_base_pattern_spacing`, `support_expansion`, `support_style`, support placement, support spacing, support line width, support extruder/tool selection, support labels, and support-pattern generation.
- Changing first-layer support-material speed or flow policy beyond preserving the existing Ares first-layer gates.
- New option registry metadata or additional milestone text beyond documenting that this deferred runtime behavior is now partially consumed.

## Test Strategy

- Add focused tests using `pipeline::test_support::single_path_pipeline` with `PrintPathRole::SupportMaterial` on layer `1` to avoid first-layer speed policy hiding non-first-layer support speed.
- RED/GREEN command: `cargo nextest run -p ares-core support_speed_flow`.
- Test that an explicit `support_speed` changes support-material G-code speed/feedrate while unrelated support-interface speed differs.
- Test that omitted `support_speed` defaults support-material speed to Orca's `80 mm/s`.
- Test that first-layer support-material speed still uses `initial_layer_infill_speed`.
- Test that invalid `support_speed` values are rejected, including percent strings.
- Test that `support_flow_ratio` changes support-material G-code extrusion only when `set_other_flow_ratios` is enabled.
- Test that omitted or disabled `set_other_flow_ratios` ignores the support-material flow scaling but still validates invalid ratio values.
- Test that first-layer support-material extrusion composes `support_flow_ratio` with `first_layer_flow_ratio` when `set_other_flow_ratios` is enabled.
- Test that `SupportMaterial` maps to `ExtrusionRole::SupportMaterial`.

## Verification

Before commit and push, run:

- `cargo fmt --check`
- `cargo nextest run -p ares-core support_speed_flow`
- `cargo nextest run -p ares-core support_interface_speed_flow set_other_flow_ratios`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- Touched Rust file LOC guard

## Documentation Impact

Update `docs/roadmap.md` where it currently says `support_flow_ratio` and support body speed behavior are deferred, preserving the remaining deferred support-generation, support-transition, support-pattern, spacing, style, and geometry scope.
