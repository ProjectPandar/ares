# Consume Role Filament Extrusion Design

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1121`, `:1154`, `:1161`: `sparse_infill_filament`, `wall_filament`, and `solid_infill_filament` are `PrintRegionConfig` 1-based extruder selectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4007-4014`, `:4887-4894`, `:5648-5655`: those options default to `1`, have minimum `1`, and describe sparse infill, walls, and solid infill filament assignment.
- `OrcaSlicer/src/libslic3r/PrintRegion.cpp:6-43`: `PrintRegion::extruder(FlowRole)` maps perimeter roles to `wall_filament`, sparse infill to `sparse_infill_filament`, and solid/top solid infill to `solid_infill_filament`; `PrintRegion::flow` then selects `print_config.nozzle_diameter.get_at(extruder(role) - 1)` for the flow geometry.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:869-923`: layer fill generation selects `frTopSolidInfill`, `frSolidInfill`, or `frInfill` before assigning concrete extrusion roles; bottom surfaces become `erBottomSurface` after the extruder has already been selected from `frSolidInfill`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1117-1132`: generated solid-fill extensions explicitly use `layerm.region().extruder(frSolidInfill)` and `layerm.flow(frSolidInfill)`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1568-1637`: ironing parameters store `config.solid_infill_filament`, derive filament-specific overrides from `extruder - 1`, and read nozzle diameter from `nozzle_diameter.get_at(ironing_params.extruder - 1)`.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:83-98`: tool ordering converts those 1-based role filament values to 0-based tool ids, but full tool ordering is not part of this slice.

## Current Ares State

Ares already normalizes legacy `extruder` into `sparse_infill_filament` and `wall_filament`, and falls back `solid_infill_filament` from sparse infill in `crates/ares-core/src/options/fdm_normalization.rs`. The runtime extrusion path still builds `ExtrusionOptions` from `nozzle_diameters()[0]` and `filament_diameters()[0]` only, so role filament choices do not affect generated extrusion amounts. That leaves concrete slicing behavior unconsumed even though the role options are present in normalized option data.

## Goal

Consume the single-region subset of `wall_filament`, `sparse_infill_filament`, and `solid_infill_filament` into actual Ares extrusion calculation by selecting the role's nozzle diameter and filament diameter when computing percentage or automatic line width fallback, thick bridge area, filament cross-section, and G-code E deltas.

## Included Behavior

- Parse explicit role filament ids from `SliceOptions` as Orca-style 1-based non-zero ids for runtime extrusion. JSON numbers and numeric strings such as `"2"` are accepted when they represent an integer id greater than zero. Missing role selectors default to `1`.
- Keep legacy `extruder = 0` valid only as the existing FDM normalization no-op; explicit `wall_filament`, `sparse_infill_filament`, or `solid_infill_filament` values of `0`, negative numbers, non-integers, or non-numeric strings fail in `SliceOptions::extrusion_options`.
- Resolve a valid role selector `N` to hardware index `N - 1` independently for each hardware vector. If `N - 1` is outside `nozzle_diameter`, use `nozzle_diameter[0]`; if `N - 1` is outside `filament_diameter`, use `filament_diameter[0]`. This matches Orca-style `get_at` fallback without adding paired-vector validation.
- Use `wall_filament` for `ExternalPerimeter`, `OverhangPerimeter`, and `InternalPerimeter`.
- Use `sparse_infill_filament` for `SparseInfill`.
- Use `solid_infill_filament` for `SolidInfill`, `TopSolidInfill`, `BottomSurface`, and `Ironing`. `BottomSurface` follows solid infill because Orca chooses `frSolidInfill` before assigning `erBottomSurface`; `Ironing` follows solid infill because Orca stores `config.solid_infill_filament` in `IroningParams`.
- Keep `Skirt`, `Brim`, `GapFill`, `Bridge`, `InternalBridge`, `SupportMaterial`, and `SupportMaterialInterface` on the first hardware entry because this slice has no source-cited Ares support-region/tool-ordering boundary for those roles.
- Preserve existing numeric explicit line width behavior.
- Resolve percentage line widths for `line_width`, `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, and `top_surface_line_width` against the nozzle diameter selected for the queried extrusion role, matching Orca's `ConfigOptionFloatOrPercent` plus `PrintRegion::flow` ordering.
- Continue resolving `support_line_width` and `initial_layer_line_width` against the first nozzle entry in this slice because support-region ownership and first-layer cross-role extrusion semantics are outside the cited role filament boundary.
- Role nozzle diameter affects automatic width fallback and thick bridge area only where the role already uses automatic width/nozzle geometry.
- Preserve existing scalar `filament_flow_ratio`, role flow ratios, print flow ratio, first-layer flow ratio, and small-area flow multiplication.
- Add runtime tests proving role filament selection changes G-code E deltas and effective line width metadata for wall, sparse infill, and solid/top/bottom surface roles.

## Deferred Behavior

- Full Orca multi-region `PrintRegion` ownership and painted region filament assignment.
- Tool-change G-code ordering, wipe tower behavior, purge volumes, and `LayerTools` scheduling beyond selecting extrusion geometry.
- Support-interface/support-material filament selectors, because current Ares support paths are synthetic role paths rather than source-cited support generation output.
- Per-role `filament_flow_ratio` vectors. Ares currently consumes the first filament flow ratio as a scalar, and this slice does not change that contract.

## Rust Destination

- `crates/ares-core/src/options/flow_ratios.rs`: pass hardware vectors and role filament ids into `ExtrusionOptions`.
- `crates/ares-core/src/extrusions/options.rs`: store per-role hardware selectors and unresolved numeric-or-percent width specs with simple builder methods or constructor arguments, following the existing immutable builder style.
- `crates/ares-core/src/extrusions/options/accessors.rs`: route nozzle diameter and filament diameter lookup through the role selector before percentage width resolution, automatic width fallback, thick bridge area, filament area, and E delta calculation.
- Tests remain in existing `crates/ares-core/src/options/tests/...`, `crates/ares-core/src/extrusions/tests...`, and `crates/ares-core/src/pipeline/tests...` modules, adding new focused files only if needed to keep Rust files under 400 LOC.

## Docs Impact

No architecture or roadmap update is required for this slice. It consumes already-staged `PrintRegionConfig` role filament options inside the existing `ares-core` extrusion boundary and does not change crate boundaries, public CLI/WASM API shape, or milestone priority. The SDD spec and implementation plan are the required docs artifacts for this source-cited slice.

## Acceptance Criteria

- `cargo nextest run -p ares-core role_filament_extrusion` fails before implementation and passes after implementation.
- `cargo nextest run -p ares-core fdm_normalization filament_flow_ratio wall_flow_ratios sparse_infill_flow_ratio top_bottom_solid_surface` passes after implementation.
- A runtime pipeline/G-code test proves changing `wall_filament` from `1` to `2` with different nozzle/filament diameters changes perimeter E output.
- The wall runtime test also verifies automatic wall line-width metadata uses the selected role nozzle when line widths are zero.
- A runtime pipeline/G-code test proves changing `sparse_infill_filament` from `1` to `2` changes sparse infill E output while perimeter remains on the wall filament selector.
- The sparse infill runtime test also verifies automatic sparse line-width metadata uses the selected role nozzle.
- Runtime tests prove `solid_infill_filament` changes solid, top, and bottom surface extrusion behavior and that their effective line-width metadata uses the selected role nozzle according to the existing Ares width rules.
- Focused width tests prove percentage `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, `top_surface_line_width`, and fallback `line_width` resolve against the selected role nozzle instead of always `nozzle_diameter[0]`.
- A focused extrusion-options test proves `Ironing` uses the solid selector for automatic top-surface width fallback and E calculation.
- Focused runtime-boundary tests prove missing `wall_filament`, `sparse_infill_filament`, and `solid_infill_filament` default to selector `1`.
- Focused runtime-boundary tests prove numeric-string role selectors such as `"2"` are accepted, while zero, negative, non-integer, and non-numeric string selectors are rejected.
- Focused runtime-boundary tests prove selector `N` values beyond one or both hardware vector lengths fall back independently: missing nozzle entries use `nozzle_diameter[0]`, and missing filament entries use `filament_diameter[0]`.
- Invalid non-integer, negative, or zero explicit role filament ids return `SliceError::InvalidInput` at the runtime extrusion-options boundary.
- Full verification passes before commit: `cargo fmt --check`, focused nextest commands, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC checks.
