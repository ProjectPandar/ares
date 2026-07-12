# Consume `support_expansion` in Support Print Paths

## Goal

Consume the existing `support_expansion` option in concrete slicing output before adding more options. The narrow runtime slice applies Orca's normal-support XY expansion concept to Ares support print-path artifacts that the current Rust pipeline can represent: closed rectangular `SupportMaterial` and `SupportMaterialInterface` paths during print-path finalization.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:972-973`: `support_base_pattern_spacing` and `support_expansion` fields in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6187-6193`: `support_expansion` is a `coFloat`, default `0`, labeled "Normal Support expansion", measured in millimeters.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1396`: `object_config.support_expansion.value` is converted to scaled XY expansion while generating support material.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1517`: non-zero `xy_expansion` is applied to support polygons with `expand(diff_polygons, xy_expansion, SUPPORT_SURFACES_OFFSET_PARAMETERS)`.

## Current Ares Boundary

- `crates/ares-core/src/print_paths/generate.rs`: finalizes `LayerPrintPaths` after path generation.
- `crates/ares-core/src/print_paths/support_interface.rs`: already consumes `support_interface_top_layers` by rewriting support interface roles.
- `crates/ares-core/src/print_paths/support_ironing.rs`: already duplicates support interface paths for support ironing and contains rectangle-bound helpers for path geometry.
- Ares currently has support-material/interface path roles, speed/flow/fan behavior, and support ironing, but does not yet have Orca's full support-area generator. `support_expansion` is applied at print-path finalization because Ares lacks the upstream `SupportMaterial.cpp` overhang-polygon generator; this is a temporary compatibility shell that will migrate to the overhang-detection/support-generation stage when that stage is ported. Rectangular closed-path offsetting represents Orca's polygon `expand(...)` behavior only within the current artifact boundary.

## Included Behavior

1. Parse `support_expansion` from existing `SliceOptions` values in print-path finalization.
2. Accept finite numeric JSON values and numeric strings in millimeters.
3. Keep omitted `support_expansion` at Orca's default `0`.
4. Apply positive values by expanding closed rectangular support-material and support-interface paths outward in XY.
5. Apply negative values by shrinking closed rectangular support-material and support-interface paths inward in XY.
6. Drop a support rectangle if shrinking collapses either axis to zero or below.
7. Preserve role, extrusion role, effective layer height, unsupported span, seam gap, closed state, layer id, and print Z for retained paths.
8. Leave non-support roles and non-rectangular support paths unchanged.
9. Run expansion after `support_interface_top_layers` role rewriting and before `support_ironing`, so support ironing duplicates inherit expanded support-interface geometry.

## Deferred Behavior

- Full Orca support material generation from overhang polygons in `SupportMaterial.cpp`.
- Tree support behavior in `TreeSupport.cpp`.
- `support_base_pattern_spacing`, `support_interface_spacing`, `support_interface_pattern`, `support_interface_loop_pattern`, support pattern generation, and support line spacing.
- Arbitrary polygon offsetting for non-rectangular support paths.
- UI metadata beyond the existing option registry.

## Acceptance Criteria

1. `support_expansion: 0.5` expands a closed rectangular `SupportMaterial` path by 0.5 mm on all sides, preserves metadata, and the G-code extrusion span reflects the larger rectangle.
2. `support_expansion: -0.25` shrinks a closed rectangular `SupportMaterialInterface` path by 0.25 mm on all sides while preserving interface role behavior and metadata.
3. A shrink that collapses a rectangle removes that support path without panicking.
4. Non-rectangular support paths and non-support paths are unchanged.
5. Invalid values such as non-finite strings, arrays, booleans, nulls, and objects produce `SliceError::InvalidInput` mentioning `support_expansion`.
6. With `support_ironing: true`, generated support ironing duplicates follow the expanded support-interface rectangle rather than the pre-expansion geometry.

## Verification

- Targeted tests for `support_expansion` in `ares-core`.
- `cargo nextest run -p ares-core support_expansion`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `support_expansion` is now consumed for current rectangular support print-path artifacts and that full Orca support generation remains deferred.
