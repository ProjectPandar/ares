# `FillSpiralInset` (`spiralinset`) Complete Parity Coverage

## Upstream Source Boundary

Pinned upstream commit: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Upstream file & symbol | Responsibility |
| --- | --- |
| `src/libslic3r/Fill/FillSpiralInset.hpp`, `FillSpiralInset.cpp` | `FillSpiralInset`: `is_self_crossing() == false`, `no_sort() == true`, classic `Polylines` overload (`:310-345`), solid Arachne `ThickPolylines` overload (`:347-493`), shared `generate_spiral_insets` chaining (`:29-308`). |
| `src/libslic3r/Fill/FillBase.cpp:36,69,131-228` | `Fill::new_from_type(ipSpiralInset)`, common `overlap - 0.5 * spacing` surface offset (`fill_surface` / `fill_surface_arachne`), `fill_surface_extrusion` (`variable_width`, `no_sort`, `keep_fill_order` reversal gate, `_create_gap_fill`). |
| `src/libslic3r/Fill/Fill.cpp:970-1015,1430-1525` | `group_fills` pattern/density/order projection (`top_surface_fill_order`, `bottom_surface_fill_order`, `internal_solid_infill_pattern`, `sparse_infill_pattern`, `elefant_foot_layers_density`) and `Layer::make_fills` (`params.use_arachne` at `:1430`, per-expolygon `f->spacing` reset at `:1502`). |
| `src/libslic3r/Fill/Fill.cpp:1561-1685` | `Layer::generate_sparse_infill_polylines_for_anchoring`: `case ipSpiralInset:` (`:1593`), zero-overlap classic `fill_surface` dispatch (`params.use_arachne = false`, `params.dont_adjust = false`). |
| `src/libslic3r/Fill/Fill.cpp:1707-1932` | `Layer::make_ironing`: `Fill::new_from_type(ironing_params.pattern)` (`:1831`) with `overlap = 0`, `density = 1.0`, `dont_adjust = true` (`FillBase.hpp:78`), `fill_order = Default`, `no_sort = true` (`:1921`). |
| `src/libslic3r/Support/SupportCommon.cpp:610-622,2037-2067` | Support contact ironing: `Fill::new_from_type(support_params.ironing_pattern)` (`:2038`) with `overlap = 0`, `density = 1.0`, `dont_adjust = true`, `fill_order = Default`, emitting `erIroning` paths. |
| `src/libslic3r/Support/SupportParameters.hpp:155-156` | Maps `smipSpiralInset` (`support_interface_pattern = spiralinset`) to `ipSpiralInset`. |
| `src/libslic3r/Layer.cpp:435-475` | `Layer::get_sparse_infill_max_void_area()` classifies `ipSpiralInset` with `ipConcentric` (`spacing * spacing`, `:453`). Its sole caller (`LayerRegion.cpp:699`) lives inside the inactive `#else` branch (`LayerRegion.cpp:654-734`), so active `LayerRegion::process_external_surfaces` (`:516-653`) does not call it. |
| `src/libslic3r/LayerRegion.cpp:1085-1126` | `LayerRegion::simplify_path`: checks `path->role() == erInternalInfill` on every fill `ExtrusionPath` (including `variable_width` outputs) for `ArcFittingMode::SparseInfill` admission and `SCALED_SPARSE_INFILL_RESOLUTION` (`0.04 mm`). |
| `src/libslic3r/GCode.cpp:9482-9483` | Multiplies `mm3_per_mm` by `bottom_solid_infill_flow_ratio` for `erBottomSurface` extrusions. |
| `src/libslic3r/PrintConfig.cpp:336,474,2559,2632,2686,7578,12499-12516` | Registers `"spiralinset"` in `InfillPattern` and `SupportMaterialInterfacePattern` key maps, adds `"spiralinset"` to `enum_values` of `top_surface_pattern`, `bottom_surface_pattern`, `internal_solid_infill_pattern`, and `support_interface_pattern`. `Slic3r::validate` (`:12499-12516`) checks `has_enum_value` on project-level `sparse_infill_pattern`, `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern`, rejecting project-level `sparse_infill_pattern = spiralinset` while accepting object/modifier-region `sparse_infill_pattern = spiralinset`, project/region `internal_solid_infill_pattern = spiralinset`, `ironing_pattern = spiralinset`, and `support_ironing_pattern = spiralinset`. |
| `src/libslic3r/ClipperUtils.cpp:1217-1287` | `traverse_pt_outside_in` re-attaches misplaced Clipper2 loops so `FillSpiralInset` receives loops in strict containment order. |

## Existing Foundation and Gaps Closed by This Slice

A prior slice (`docs/architecture/spiral-inset-fill.md`) ported the core
`FillSpiralInset` algorithm in `crates/ares-core/src/fill/spiral_inset.rs`,
`chain.rs`, and `walls.rs`, together with `outside_in.rs`, top/bottom surface
dispatch in `project_slice/fill_entities/spiral_inset.rs`, and support
interface dispatch. Four upstream `spiralinset` entry points remained guarded
or incomplete; this slice closes every remaining project-pipeline gap:

1. **Capability guards (`crates/ares-core/src/project_slice/capabilities.rs`)**:
   - Remove the temporary `spiralinset` rejections for
     `internal_solid_infill_pattern`, `sparse_infill_pattern`,
     `ironing_pattern`, and `support_ironing_pattern` in both `validate` and
     `validate_settings`.
   - Leave project-level `sparse_infill_pattern = spiralinset` validation to
     `options/validation/infill_pattern.rs` (mirroring `PrintConfig.cpp:12499`),
     which already rejects `"spiralinset"` on project-level
     `sparse_infill_pattern` while allowing object/region overrides in
     `Metadata/model_settings.config`, matching upstream OrcaSlicer.
   - Do not alter any unrelated guards (`brim_type`, `print_order`,
     `adaptive_infill`, `filament_map_mode`, `layer_config_ranges`).

2. **Project surface & sparse fill materialization (`project_slice/fill_entities/spiral_inset.rs`, `perimeters/classic/materialize/types.rs`, `extrusion_islands/tool_regions.rs`, `gcode_emit/motion/materialized.rs`, `contour_z/entities.rs`, `gcode_emit/layers/farthest/target.rs`, `fill_entities/simplify.rs`, `path_simplification.rs`)**:
   - Add `InternalInfill` to `materialize::ExtrusionRole` so `FillSpiralInset`
     paths materialized through `variable_width::convert_with_role`
     (`FillBase.cpp:207-210`) retain `ExtrusionRole::InternalInfill` (`;TYPE:Sparse infill`),
     sparse tool-region accounting, and `LayerRegion.cpp:1105-1113`
     sparse-infill simplification and arc-fitting (`0.04 mm` tolerance and
     `ArcFittingMode::SparseInfill` admission).
   - Reset `spacing = params.spacing` per expolygon inside
     `fill_entities::spiral_inset::append` (`Fill.cpp:1502`).
   - Apply `params.flow_ratio` to variable-width entities emitted by
     `fill_entities::spiral_inset::append` so `bottom_solid_infill_flow_ratio`
     (`GCode.cpp:9483`) is preserved after `variable_width::convert_with_role`
     reconstructs per-segment `Flow::with_width`.

3. **Object ironing & support contact ironing (`project_slice/fill_entities/surface.rs`)**:
   - Replace the `ProcessInfillPattern::SpiralInset` unsupported error in
     `surface_polylines` with classic `FillSpiralInset::_fill_surface_single`
     dispatch (`FillBase.cpp:69,131-150`, applying `overlap - 0.5 * spacing`
     and propagating `request.call.dont_adjust` and `params.fill_order`).
   - Both `fill_entities/ironing/surface.rs` (`Fill.cpp:1831,1913`) and
     `support_material/ironing.rs` (`SupportCommon.cpp:2038,2058`) flow
     through `surface_polylines`, enabling both `ironing_pattern = spiralinset`
     and `support_ironing_pattern = spiralinset` without duplicating logic.

4. **Bridge-over-infill sparse anchoring (`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms_tail.rs`)**:
   - Add the `SurfaceFillPattern::Configured(ProcessInfillPattern::SpiralInset)`
     arm matching `Fill.cpp:1593,1646-1679`: contract each expolygon by
     `-0.5 * spacing` (zero overlap), reset spacing per expolygon, and invoke
     `fill::spiral_inset::classic` with `dont_adjust = false` and
     `ProcessSurfaceFillOrder::Default`.

## Deferred / Excluded Boundaries

- Raw STL/option-only entry (`options/infill/patterns.rs` and legacy `infills`
  module) does not use the project Arachne pipeline and remains outside this
  slice.
- `Layer::get_sparse_infill_max_void_area` (`Layer.cpp:453`) is dead upstream
  under `LayerRegion.cpp:654`'s `#else` branch and is not ported into active
  runtime code.
- Unrelated capability rejections (`brim_type: leading_edge_only`,
  `print_order`, `adaptive_infill`, `filament_map_mode`, `layer_config_ranges`)
  remain unchanged.

## Verification Strategy

Live OrcaSlicer v2.5.0 determinism checks confirm that official and seeded
reference builds produce identical G-code across all modes below:

- `internal_solid_infill_pattern = spiralinset` (default and with
  `detect_narrow_internal_solid_infill = 0` on hole geometry, plus
  `elefant_foot_layers_density < 100%` exercising the classic density fallback)
- Object-level `sparse_infill_pattern = spiralinset` in
  `Metadata/model_settings.config` (both box and hole geometry with internal
  bridges exercising `generate_sparse_infill_polylines_for_anchoring`)
- Project-level `sparse_infill_pattern = spiralinset` rejected by validated
  project slicing matching `PrintConfig.cpp:12499-12501`
- `top_surface_pattern = spiralinset` and `bottom_surface_pattern = spiralinset`
  on hole geometry across `default`, `inward`, and `outward` fill orders, plus
  `bottom_solid_infill_flow_ratio != 1.0`
- `ironing_pattern = spiralinset` (top ironing on hole geometry and
  single-nozzle multi-layer geometry)
- `support_ironing_pattern = spiralinset` (organic tree support contact ironing)

Whole-G-code parity tests compare native OrcaSlicer output against both the
validated Ares public API and `ares` CLI using
`golden::compare_ordered_bytes_generator_only`, plus existing `concentric`,
`ironing`, and `spiral_inset` regression suites.
