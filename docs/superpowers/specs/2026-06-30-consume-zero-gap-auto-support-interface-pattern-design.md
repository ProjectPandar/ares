# Consume Zero-Gap Auto Support Interface Pattern

## Goal

Consume `support_top_z_distance = 0` in Ares' current rectangular support-interface compatibility shell by resolving `support_interface_pattern = "auto"` to the existing concentric support-interface proxy when top interface layers are present. This connects the already parsed support Z-distance state to the already implemented support-interface pattern behavior without adding support contact-layer topology.

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:190-192`: declares `SupportMaterialInterfacePattern` values, including `smipAuto` and `smipConcentric`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:956`: declares `support_top_z_distance`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:333-340`: maps `support_interface_pattern` strings, including `"auto"` and `"concentric"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6000`: defines `support_top_z_distance` as a non-negative float with default `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6158-6176`: defines `support_interface_pattern`, defaulting to `smipAuto`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:80-120`: derives `zero_gap_interface_top` when `support_interface_top_layers > 0` and `support_top_z_distance == 0`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:129-138`: resolves `smipAuto` plus `zero_gap_interface_top` to `ipConcentric`.

Ares destination boundary:

- `crates/ares-core/src/print_paths/support_interface_spacing.rs`: use the existing `SupportZDistanceOptions` zero-gap helper when parsing `support_interface_pattern`.
- `crates/ares-core/src/print_paths/generate.rs`: pass the already parsed support Z-distance state into the support-interface spacing pass.
- `crates/ares-core/src/pipeline/tests/support_z_distance.rs`: revise the previous deferred-geometry assertion into concrete zero-gap auto-pattern behavior.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern_gcode.rs`: add G-code evidence for auto plus zero top gap resolving to concentric output.
- `docs/roadmap.md`: record the consumed source-cited behavior and remaining deferrals.

This slice does not add options, option registry entries, dependencies, crates, UI/CLI/WASM bindings, or a new support-generation pipeline.

## Current Ares State

Ares already parses `support_top_z_distance`, `support_bottom_z_distance`, and `enforce_support_layers` into `SupportZDistanceOptions`. It also exposes `zero_gap_interface_top(top_layers)` matching Orca's `Slicing.cpp` predicate, but the helper is currently unused by print-path behavior.

Ares also already parses `support_interface_pattern` in `support_interface_spacing.rs`. The explicit `concentric` pattern emits nested closed rectangular support-interface loops. The current `auto` pattern is always treated as the same single-family rectilinear output as `rectilinear`, regardless of zero top Z gap.

## Included Behavior

1. Keep `support_interface_pattern` parsing and accepted strings unchanged.
2. Keep `support_top_z_distance` validation in the existing typed parser.
3. When `support_interface_pattern` is omitted or explicitly `"auto"`, `support_interface_top_layers > 0`, and `support_top_z_distance == 0.0`, emit the same concentric rectangular support-interface loops as explicit `support_interface_pattern = "concentric"`.
4. When `support_top_z_distance` is omitted or positive, keep omitted and `"auto"` on the current single-family rectilinear output.
5. When `support_interface_top_layers = 0`, keep the existing ordering where interface paths are rewritten to `SupportMaterial` before pattern conversion; zero top gap must not reintroduce interface paths.
6. Keep explicit `rectilinear`, `concentric`, `rectilinear_interlaced`, and `grid` behavior unchanged. Explicit `rectilinear` must stay rectilinear even with zero top Z gap.
7. Preserve `support_interface_loop_pattern`, `support_interface_spacing`, support angle, support ironing, non-target path preservation, metadata preservation, and invalid-pattern validation behavior.
8. Leave `support_bottom_z_distance` zero-gap behavior to a future support contact/bottom-interface slice.

## Deferred Behavior

- Full Orca support contact-layer topology, layer synchronization, and Z placement from `gap_support_object`.
- Exact support-region generation from overhang/contact polygons.
- Soluble interface filament resolution for `smipAuto`.
- Bottom-interface, raft-interface, tree/organic support, support-base-interface, and support-material extruder interactions.
- Exact `FillConcentric` polygon clipping, holes, island ordering, links, and arbitrary polygon behavior beyond Ares' current rectangular proxy.
- UI, CLI, WASM binding changes, new options, new dependencies, and Orca binary E2E support parity.

## Acceptance Criteria

1. `support_interface_pattern = "auto"` plus `support_top_z_distance = 0.0` and positive top interface layers produces the same closed rectangular support-interface loops as explicit `concentric`.
2. Omitted `support_interface_pattern` plus `support_top_z_distance = 0.0` also resolves to concentric.
3. Default or positive `support_top_z_distance` keeps `auto` on the existing single-family line output.
4. Explicit `rectilinear` with `support_top_z_distance = 0.0` keeps single-family rectilinear output.
5. `support_interface_top_layers = 0` with `support_top_z_distance = 0.0` still rewrites interface paths to `SupportMaterial` and bypasses interface pattern conversion.
6. `support_ironing = true` still preserves the solid support-interface rectangle for support ironing while validating invalid pattern values.
7. Existing explicit `concentric`, `grid`, `rectilinear_interlaced`, support-interface spacing, loop-pattern, support-angle, support-expansion, support-base-pattern, and support-Z-distance validation tests remain passing.
8. A focused G-code regression proves `auto` plus zero top gap emits closed-loop support-interface coordinates/count that differ from default auto output.

## Verification Plan

- `cargo nextest run -p ares-core support_z_distance`
- `cargo nextest run -p ares-core support_interface_pattern`
- `cargo nextest run -p ares-core support_interface_pattern_gcode`
- `cargo nextest run -p ares-core support_interface_pattern_concentric support_interface_pattern_interlaced`
- `cargo nextest run -p ares-core support_interface_spacing support_interface_loop_pattern support_ironing_pattern support_ironing_spacing`
- `cargo nextest run -p ares-core support_base_pattern support_base_pattern_spacing support_angle support_expansion`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
