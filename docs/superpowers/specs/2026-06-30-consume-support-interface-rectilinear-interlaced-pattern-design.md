# Consume Support Interface Rectilinear-Interlaced Pattern

## Goal

Consume the already parsed `support_interface_pattern = "rectilinear_interlaced"` option in Ares' current rectangular support-interface compatibility shell. Closed rectangular `SupportMaterialInterface` proxy paths should use an explicit interlaced rectilinear angle family instead of being treated as the same single-family output as `auto` and `rectilinear`.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:190-192`: declares `SupportMaterialInterfacePattern` with `smipRectilinearInterlaced`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:333-340`: maps `"rectilinear_interlaced"` to `smipRectilinearInterlaced`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6158-6176`: registers `support_interface_pattern`, including the `rectilinear_interlaced` value and UI label.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:103-138`: maps `smipRectilinearInterlaced` to `ipRectilinear` contact fill, separate from grid and concentric.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:140-161,277-278`: initializes no-raft `raft_angle_interface` to `0` and defines `raft_interface_angle(interface_id)` as `raft_angle_interface +/- 45deg`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1589-1592,1713-1718,1748-1754`: chooses `raft_interface_angle(support_layer.interface_id())` for `smipRectilinearInterlaced` interface fills.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1476,1554-1585,2426-2452`: tree support has a separate explicit 0/90 interlaced branch keyed by `area_group.interface_id`; this remains deferred for this rectangular classic-support proxy slice.

## Rust Destination

- `crates/ares-core/src/print_paths/support_interface_spacing.rs`: distinguish `rectilinear_interlaced` from `SingleFamily` and choose an interlaced angle for eligible rectangular support-interface paths.
- `crates/ares-core/src/pipeline/tests.rs`: register a focused interlaced support-interface pattern test module.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern.rs`: remove the stale accepted-as-single-family assertion for `rectilinear_interlaced`.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern_interlaced.rs`: add concrete geometry, ordering, metadata, and interaction coverage without pushing the existing pattern test file over the 400-LOC split threshold.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern_gcode.rs`: add focused G-code evidence that interlaced output changes support-interface coordinates versus the default single family.
- `docs/roadmap.md`: add a dated source-cited roadmap entry for this slice.

## Current State

Ares already validates `support_interface_pattern`, maps `grid` to two rectangular line families, maps `concentric` to nested rectangular loops, and still maps `rectilinear_interlaced` to the same single-family `support_angle + 90` rectangular output as `auto` and `rectilinear`.

The current print-path compatibility shell does not have Orca `SupportLayer::interface_id()`, roof/floor interface group ownership, tree-support `area_group.interface_id`, raft interface-layer state, or polygon fill sorting. It does have stable layer ids and rectangular support-interface proxy paths, which are sufficient for a deterministic no-raft classic-support approximation of the `raft_interface_angle(interface_id)` behavior. In Orca's no-raft classic-support path, `raft_angle_interface` remains `0`, so this slice intentionally does not compose the interlaced angle with `support_angle`.

## Included Behavior

1. Keep `support_interface_pattern` parsing and validation on the current key and accepted string set.
2. Add an internal `RectilinearInterlaced` pattern variant.
3. With `rectilinear_interlaced`, convert each eligible closed rectangular `SupportMaterialInterface` path to open rectilinear interface lines at an interlaced angle selected from the proxy layer id:
   - even `LayerPrintPaths::layer_id()` uses `45deg`;
   - odd `LayerPrintPaths::layer_id()` uses `-45deg`.
4. Use the same pitch already owned by this pass: selected support-interface spacing plus support-interface extrusion width.
5. Preserve source metadata through `rebuild_path`: role, extrusion role, effective layer height, effective line width, unsupported span, seam gap, layer id, and print Z.
6. Keep `support_interface_loop_pattern = true` scoped to line-family patterns, including `rectilinear_interlaced`: prepend the existing closed outer support-interface shell, then append the interlaced open lines.
7. Preserve `support_ironing = true`: still parse and validate `support_interface_pattern`, but keep the source support-interface rectangle solid for support ironing instead of converting it first.
8. Preserve `support_interface_top_layers = 0`: converted `SupportMaterial` paths bypass `support_interface_pattern` and flow to support-base pattern behavior.
9. Leave omitted, `auto`, `rectilinear`, `grid`, `concentric`, non-interface roles, non-closed interface paths, and non-rectangular interface paths within their current semantics.

## Deferred Behavior

- Exact `SupportLayer::interface_id()` sequencing for classic support.
- Exact roof/floor interface separation, top/base interface ownership, and bottom-interface id handling.
- Tree/organic support's explicit 0/90 rectilinear-interlaced branch and `area_group.interface_id` normalization.
- Raft-specific interlaced interface angles, including raft cases where `raft_angle_interface` is derived from `support_angle`, and raft contact/base/interface planning.
- Full Orca `FillRectilinear` path sorting, linking, density behavior, arbitrary polygon clipping, holes, bridge/contact classification, support-layer storage, UI/CLI/WASM binding changes, and Orca binary E2E support parity.
- New options, new crates, or independent Ares pipeline abstractions.

## Acceptance Criteria

1. `support_interface_pattern = "rectilinear_interlaced"` no longer matches omitted/`auto`/`rectilinear` output for eligible rectangular support-interface paths.
2. Even proxy layer ids emit `45deg` rectangular lines.
3. Odd proxy layer ids emit `-45deg` rectangular lines.
4. Configured `support_angle` does not affect the no-raft interlaced proxy angle, while continuing to affect `rectilinear` and `grid` through their existing behavior.
5. `support_interface_spacing = 0.0` changes interlaced line density through the same pitch calculation as other interface patterns.
6. Interlaced lines preserve source print-path metadata and remain open `SupportMaterialInterface` paths.
7. `support_interface_loop_pattern = true` prepends the closed outer shell before interlaced lines.
8. `support_ironing = true`, `support_interface_top_layers = 0`, invalid values, non-target paths, and existing `auto`/`rectilinear`/`grid`/`concentric` behavior remain passing.
9. A focused G-code regression proves interlaced support-interface coordinates differ from default output.

## Verification Plan

- `cargo nextest run -p ares-core support_interface_pattern`
- `cargo nextest run -p ares-core support_interface_pattern_gcode`
- `cargo nextest run -p ares-core support_interface_spacing support_interface_loop_pattern support_ironing_pattern support_ironing_spacing`
- `cargo nextest run -p ares-core support_base_pattern support_base_pattern_spacing support_angle support_expansion`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
