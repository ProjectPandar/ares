# Consume Support Interface Speed and Flow Design

## Objective

Consume the existing OrcaSlicer support-interface speed and flow options in concrete Ares slicing/G-code behavior. This is a source-cited rewrite slice, not new option metadata: existing `support_interface_speed` and `support_interface_flow_ratio` inputs must affect constructed `PrintPathRole::SupportMaterialInterface` moves.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:968` declares `support_interface_speed` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:981` declares `support_interface_flow_ratio` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1394-1403` defines `support_interface_flow_ratio` as a float, default `1`, range `0..=2`, for support interface material amount.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6124-6131` defines `support_interface_speed` as a float, default `80`, minimum `1`, in `mm/s`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6428-6431` multiplies `erSupportMaterialInterface` flow by `support_interface_flow_ratio` under the existing other-flow-ratio handling.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6475-6479` resolves `erSupportMaterialInterface` speed from `support_interface_speed`.

## Current Ares State

- `crates/ares-core/src/print_paths.rs` already has `PrintPathRole::SupportMaterialInterface`.
- `crates/ares-core/src/extrusion_entity.rs` already maps that print path role to `ExtrusionRole::SupportMaterialInterface`.
- `crates/ares-core/src/speeds/config/accessors.rs` currently maps non-first-layer support-interface print speed to `internal_solid_infill_speed_mm_s`.
- `crates/ares-core/src/extrusions/options.rs` currently maps support-interface flow to `internal_solid_infill_flow_ratio`.
- `docs/roadmap.md` records both support-interface options as registry metadata with runtime behavior deferred.

## Requirements

1. Parse `support_interface_speed` through `ares-core` slice options as a finite numeric JSON number or numeric string, with Orca defaults and bounds: default `80.0`, minimum `1.0`, no percent syntax.
2. Route non-first-layer `PrintPathRole::SupportMaterialInterface` print speed through the parsed `support_interface_speed` value, producing the matching G-code feedrate and speed comments.
3. Preserve Ares's existing first-layer speed policy for `SupportMaterialInterface`: first-layer support-interface print moves continue to use `initial_layer_infill_speed`.
4. Parse `support_interface_flow_ratio` as a finite numeric JSON number or numeric string, default `1.0`, range `0.0..=2.0`.
5. When `set_other_flow_ratios` is enabled, route `PrintPathRole::SupportMaterialInterface` extrusion through `support_interface_flow_ratio` instead of `internal_solid_infill_flow_ratio`.
6. When `set_other_flow_ratios` is omitted or disabled, validate `support_interface_flow_ratio` but leave support-interface extrusion unscaled by it, matching the existing Ares gate used for other flow-ratio options.
7. Keep the implementation platform-neutral and WASM-safe: no file I/O, terminal behavior, UI behavior, native viewer code, OpenGL, or new dependencies in `ares-core`.
8. Keep every touched Rust file at or below 400 LOC.

## Out Of Scope

- Full support generation.
- `support_speed`, `support_flow_ratio`, support body path roles, or support transition roles.
- Support base/interface pattern generation, support spacing, bottom-interface behavior, support extruder/tool selection, or support label generation.
- Changing first-layer support-interface speed or flow policy beyond preserving the existing Ares first-layer gates.
- New option registry metadata or additional milestone text beyond documenting that this deferred runtime behavior is now partially consumed.

## Test Strategy

- Add focused tests using `pipeline::test_support::single_path_pipeline` with `PrintPathRole::SupportMaterialInterface` on layer `1` to avoid first-layer speed policy hiding non-first-layer support-interface speed.
- RED/GREEN command: `cargo nextest run -p ares-core support_interface_speed_flow`.
- Test that an explicit `support_interface_speed` changes the support-interface G-code speed/feedrate while unrelated internal-solid speed differs.
- Test that omitted `support_interface_speed` defaults support-interface speed to Orca's `80 mm/s`.
- Test that invalid `support_interface_speed` values are rejected.
- Test that `support_interface_flow_ratio` changes support-interface G-code extrusion only when `set_other_flow_ratios` is enabled.
- Test that omitted or disabled `set_other_flow_ratios` ignores the support-interface flow scaling but still validates invalid ratio values.

## Verification

Before commit and push, run:

- `cargo fmt --check`
- `cargo nextest run -p ares-core support_interface_speed_flow`
- `cargo nextest run -p ares-core set_other_flow_ratios wall_flow_ratios support_ironing_role_fan_gcode`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- Touched Rust file LOC guard

## Documentation Impact

Update `docs/roadmap.md` where it currently says `support_interface_flow_ratio` and `support_interface_speed` runtime behavior is deferred, preserving the remaining deferred support-generation and support-pattern scope.
