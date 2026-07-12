# Consume Support Interface Concentric Pattern Proxy

## Goal

Consume the already parsed `support_interface_pattern = "concentric"` option in Ares' current rectangular support-interface compatibility shell. Closed rectangular `SupportMaterialInterface` proxy paths should become nested closed interface loops, while omitted, `auto`, `rectilinear`, and `grid` preserve their current behavior.

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:190-192`: `SupportMaterialInterfacePattern` declares `smipAuto`, `smipRectilinear`, `smipConcentric`, `smipRectilinearInterlaced`, and `smipGrid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:333-340`: option strings map to `auto`, `rectilinear`, `concentric`, `rectilinear_interlaced`, and `grid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6158-6176`: `support_interface_pattern` defaults to `smipAuto` and documents concentric as the soluble-interface default.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:103-138`: support parameters select concentric or rectilinear/interface-base fill from `smipAuto`, `smipConcentric`, soluble-interface context, zero-gap interface, and top-interface density.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1563-1592,1694-1733`: normal support builds `filler_interface` from the resolved contact fill pattern and emits support-interface contact paths with `ExtrusionRole::erSupportMaterialInterface`.
- `OrcaSlicer/src/libslic3r/Fill/FillConcentric.cpp` and `Fill/FillConcentric.hpp`: upstream concentric fill generator boundary for exact polygon/hole-aware behavior.

Ares destination boundary:

- `crates/ares-core/src/print_paths/support_interface_spacing.rs`: extend the current parsed `SupportInterfacePattern` compatibility shell so `concentric` emits nested closed rectangular interface loops.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern.rs`: replace the current deferred `concentric` assertion with concrete geometry and metadata coverage.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern_gcode.rs`: add focused G-code evidence that concentric changes support-interface path coordinates/count.
- `docs/roadmap.md`: record the source-cited consumed behavior and remaining deferrals.

This is a source-cited rewrite slice over existing Ares support-interface print-path artifacts. It does not add options, option registry entries, dependencies, crates, or a new support pipeline.

## Current Ares State

Ares already validates `support_interface_pattern`, uses `auto`/`rectilinear` as the single interface-angle family, and maps `grid` to interface-angle lines followed by base-angle lines. `concentric` and `rectilinear_interlaced` are currently accepted but intentionally routed to the same single-family output.

The existing rectangular concentric-loop compatibility behavior in ordinary ironing and support ironing provides the local shape for this slice: nested axis-aligned rectangular loops, spaced by the effective path pitch, preserving source print-path metadata. Unlike ironing roles, support-interface loops must also store the starting point again at the end so the current G-code move generator emits the closing edge for `SupportMaterialInterface`.

## Included Behavior

1. Keep `support_interface_pattern` parsing and validation exactly on the current option key and accepted string set.
2. With the option omitted, `auto`, or `rectilinear`, keep current single-family interface lines at `support_angle + 90`.
3. With `grid`, keep current two-family line output and ordering.
4. With `concentric`, convert each eligible closed rectangular `SupportMaterialInterface` path into nested closed rectangular `SupportMaterialInterface` loops whose point lists end where they started.
5. Use the same effective pitch already owned by the support-interface spacing pass: selected interface spacing plus support-interface extrusion width.
6. Preserve source metadata through the existing `rebuild_path` helper: role, extrusion role, effective layer height, effective line width, unsupported span, seam gap, layer id, and print Z.
7. Keep `support_interface_loop_pattern = true` behavior scoped to line-family patterns only. Concentric loops are already closed loops and should not receive an extra duplicate outer loop from the loop-pattern option.
8. Preserve `support_interface_top_layers = 0` ordering: converted `SupportMaterial` paths bypass `support_interface_pattern` and flow to support-base pattern behavior.
9. Preserve `support_ironing = true` behavior: still parse and validate `support_interface_pattern`, but keep the source support-interface rectangle solid for support ironing instead of converting it to concentric loops first.
10. Leave non-interface roles, `SupportMaterial` paths, non-rectangular interface paths, and non-closed interface paths unchanged.
11. Keep `rectilinear_interlaced` accepted and deferred as the current single-family output.

## Deferred Behavior

- Full Orca support-region generation from overhang/contact polygons.
- Exact `FillConcentric` polygon clipping, holes, island ordering, chaining, links, overlap handling, and arbitrary polygon support.
- Exact `smipAuto` soluble-interface resolution to concentric.
- True `smipRectilinearInterlaced` per-interface-id angle alternation.
- Raft contact/base-interface, bottom-interface, tree/organic, support-style-specific, and support-material-generation interactions.
- UI, CLI, WASM binding changes, new options, new dependencies, and Orca binary E2E support parity.

## Acceptance Criteria

1. `support_interface_pattern = "concentric"` changes a closed rectangular support-interface path from open line output to nested closed rectangular interface loops.
2. The concentric loops use the same pitch as `support_interface_spacing + support interface width`; `support_interface_spacing = 0.0` therefore produces denser loops.
3. A narrow rectangle stops before collapsed loops and never emits degenerate loop geometry.
4. Concentric output preserves support-interface role, extrusion role, effective line width, effective layer height, unsupported span, seam gap, layer id, and print Z.
5. `support_interface_loop_pattern = true` does not add an extra duplicate outer loop for `concentric`.
6. `support_ironing = true` keeps the solid interface rectangle and still validates invalid pattern values.
7. `rectilinear_interlaced` remains accepted and single-family for this slice.
8. `grid`, `auto`, `rectilinear`, omitted pattern, top-layer conversion, and non-target preservation tests remain passing.
9. A G-code regression proves `concentric` changes emitted support-interface coordinates/count versus the default rectangular line family.

## Verification Plan

- `cargo nextest run -p ares-core support_interface_pattern`
- `cargo nextest run -p ares-core support_interface_pattern_gcode`
- `cargo nextest run -p ares-core support_interface_spacing support_ironing_pattern support_ironing_spacing`
- `cargo nextest run -p ares-core support_base_pattern support_base_pattern_spacing support_angle support_expansion`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
