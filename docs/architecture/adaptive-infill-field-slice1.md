# Adaptive infill field — continuous gyroid slice

## Upstream source (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)

- `src/libslic3r/PrintConfig.hpp`:
  - `is_adaptive_infill_pattern` (`:177`): admits `ipGyroid`, `ipTpmsD`, `ipTpmsFK`.
  - `AdaptiveInfillMode` (`:291-301`): `Disabled`, `DistanceWarp`, `SmoothBlend`, `SteppedShells`, `Lobes`, `NormalZ`, `NormalY`, `NormalX`, `Count`.
  - `AdaptiveInfillGradient` (`:304-308`): `Linear`, `Quadratic`, `Exponential`.
  - `PrintRegionConfig` fields (`:1528-1530`): `adaptive_infill_mode`, `adaptive_infill_interior_density`, `adaptive_infill_gradient`.
- `src/libslic3r/PrintConfig.cpp`:
  - Static enum maps (`:387-404`) and option definitions (`:3727-3793`).
  - `PrintConfigDef::handle_legacy` (`:9765-9770`): renames `tpms_adaptive` -> `adaptive_infill_mode`, `tpms_interior_density` -> `adaptive_infill_interior_density`, `tpms_adaptive_gradient` -> `adaptive_infill_gradient`.
- `src/libslic3r/Print.hpp`: `PrintObject::adaptive_infill_field` (`:410`), `prepare_adaptive_infill_fields` (`:659`), `m_adaptive_infill_fields` (`:710`).
- `src/libslic3r/PrintObject.cpp`:
  - `PrintObject::prepare_adaptive_infill_fields` (`:1274-1304`): scans printing regions for `0 < sparse_infill_density < 100` and `is_adaptive_infill_pattern`, collects `{layer->bottom_z(), layer->print_z, &layer->lslices}` and bounding box, builds one `AdaptiveInfillField` per active mode, retains non-empty fields.
  - Invoked at `:3365` before `generate_sparse_infill_polylines_for_anchoring` (`:3381`) and `Layer::make_fills` (`Fill.cpp:1387`).
- `src/libslic3r/Fill/AdaptiveInfillField.hpp` (`:24-125`) and `src/libslic3r/Fill/AdaptiveInfillField.cpp`:
  - Separable squared Euclidean distance transform (`distance_transform_line` `:56-84`, `distance_transform_axis` `:86-110`), even-odd scanline `rasterize` (`:113-146`), `target_scale` (`:149-156`), `DensityLevels` (`:160-175`), `RadialScale` (`:180-211`).
  - `AdaptiveInfillField::AdaptiveInfillField` (`:215-489`): grid sizing (`MaxNodes = 1 << 20`, `MinCellSize = 0.5`), connected-component bodies, local maxima and neck-tested lobe clustering (`NeckRatio = 0.8`, `MinLobeRatio = 0.3`), normalized `m_depth`, raymarched `reach` (`Polar = 24`, `Azimuth = 48`, `LobeReach = 2.0`, two-pass 3x3 box filter on `log(reach)`), `DistanceWarp` `mean_depth` (`Samples = 32`), 2D `m_section` lookup, multi-body BFS.
  - Queries: `depth` (`:491-509`), `offset` (`:511-517`), `radial(Lobe, pt)` (`:519-563`), `radial(pt, out)` (`:565-589`), `body_radial` (`:591-629`).
  - `marchsq::AdaptiveTpmsField` (`:636-708`) and `make_adaptive_tpms` (`:751-776`).
- `src/libslic3r/Fill/FillBase.hpp`: `FillParams` fields (`:97-99`), `Fill::adaptive_infill_field` (`:168`).
- `src/libslic3r/Fill/Fill.cpp`:
  - `SurfaceFillParams` fields and ordering/equality (`:317-319, 363-365, 399-401`).
  - Projection (`:1024-1034`): activates only for `is_adaptive_infill_pattern && extrusion_role == erInternalInfill`, clamps `adaptive_infill_interior_density` to at least `1.0`, suppresses `gyroid_optimized` when adaptive mode is active.
  - `Layer::make_fills` (`:1387, 1437-1439`) and `Layer::generate_sparse_infill_polylines_for_anchoring` (`:1620, 1660-1662`).
- `src/libslic3r/Fill/FillGyroid.cpp`: `gyroid(float, float, float)` (`:25-28`) and continuous adaptive branch (`:345-349`, `frequency(d) = d * DensityAdjust / (multiline * spacing)`).
- `src/libslic3r/Layer.cpp`: `Layer::get_sparse_infill_max_void_area` (`:446-447`) is unreachable in `40ecc66f7` because its only caller (`LayerRegion.cpp:699`) sits in the disabled `#else` branch of `#if 1` (`LayerRegion.cpp:176-963`).

## Rust destination

- `crates/ares-core/src/fill/adaptive_infill_field.rs` and submodules:
  - `grid.rs`: grid sizing, `rasterize`, 1D/axis Felzenszwalb-Huttenlocher distance transform.
  - `build.rs`: `AdaptiveInfillField::new` (bodies, peaks, neck test, tied centers, `m_depth`, raymarched `reach`, 3x3 smoothing, `mean_depth`, `m_section`, multi-body BFS).
  - `query.rs`: `empty`, `axis`, `mode`, `depth`, `offset`, `radial`, `body_radial`.
  - `scale.rs`: `target_scale`, `DensityLevels`, `RadialScale`.
  - `tpms.rs`: `AdaptiveTpmsField`, `make_adaptive_tpms`, `gyroid`, `fill_adaptive_gyroid`.
- `crates/ares-core/src/options/`:
  - `region_options.rs`, `region_options/{overrides.rs, metadata.rs, merge.rs}`: region-level `adaptive_infill_mode`, `adaptive_infill_interior_density`, `adaptive_infill_gradient`.
  - `legacy/token.rs` and `typed_legacy/actions/rules.rs`: legacy key renames (`tpms_adaptive`, `tpms_interior_density`, `tpms_adaptive_gradient`).
- `crates/ares-core/src/project_slice/`:
  - `capabilities.rs`: explicit `UnsupportedProjectFeature` rejection for deferred active modes/patterns.
  - `group_fills/{types.rs, coalesce.rs, params/projection.rs}`: `SurfaceFillParams` fields, comparison order, `gyroid_optimized` suppression.
  - `prepare_infill/adaptive_fields.rs`: `prepare_adaptive_infill_fields` built once per object on `PostCompensationPrintObject::lslices` and `PlannedLayer::{print_z, height}`.
  - `prepare_infill/bridge_over_infill/{types.rs, transaction.rs, transaction/candidate_expansion.rs, transaction/candidate_expansion/layers.rs, sparse_anchoring/mod.rs, sparse_anchoring/arms.rs}`: cache and thread fields into bridge anchoring.
  - `fill_entities.rs`, `fill_entities/{generate.rs, gyroid.rs}`: thread fields into gyroid infill emission.

## Included and deferred behaviour

Included:
- Region-level overrides (object, modifier, part, layer range) and legacy key renames for `adaptive_infill_mode`, `adaptive_infill_interior_density`, and `adaptive_infill_gradient`.
- `SurfaceFillParams` projection, coalescing, and `gyroid_optimized` precedence when adaptive infill is active.
- Full `AdaptiveInfillField` construction and `radial`/`depth` evaluation for all six continuous modes (`distance_warp`, `smooth_blend`, `lobes`, `normal_z`, `normal_y`, `normal_x`) and all three gradients (`linear`, `quadratic`, `exponential`).
- Continuous adaptive `gyroid` execution through both sparse infill emission and bridge-over-infill anchoring, including empty-field fallback for thin parts and no-op handling for non-adaptive patterns or `0%`/`100%` density.
- Explicit `SliceError::UnsupportedProjectFeature` admission errors for deferred active paths so unsupported settings never silently emit regular infill.

Deferred:
- `SteppedShells` mode (`AdaptiveLevelField`, `make_adaptive_infill_shells`, `fill_adaptive_infill_shells`).
- Continuous and stepped-shell adaptive dispatch for `tpmsd` (`FillTpmsD.cpp:28-31, 120-139`) and `tpmsfk` (`FillTpmsFK.cpp:27-30, 135-157`).

## Failure modes

- Precision mismatch between `f32::sqrt` in peak joining (`AdaptiveInfillField.cpp:318`) and `f64::sqrt` in body/lobe depth and normalized `m_depth` (`:290, 339, 348`).
- Wrong rounding mode in `node_of` / `radial`: C++ `std::lround` rounds half away from zero (`f64::round`), not ties-to-even.
- Wrong grid alignment for negative coordinates: `align_to_grid` must floor via Euclidean division (`div_euclid * cell`).
- Using combined `flow.height` instead of planned `layer.height` for layer midpoint `z - 0.5 * layer_height` in `make_adaptive_tpms`.
- Applying `overlap - 0.5 * spacing` contraction inconsistently between `fill_entities` (`overlap`) and `sparse_anchoring` (`overlap = 0.0`).
- Failing to suppress `gyroid_optimized` when `adaptive_infill_mode != Disabled`, or failing to fall back to regular gyroid when `AdaptiveInfillField::empty()` is true.
- Silently accepting deferred active modes (`stepped_shells` on `gyroid`, or any active mode on `tpmsd`/`tpmsfk` with `0 < sparse_infill_density < 100`) instead of returning `UnsupportedProjectFeature`, or erroneously rejecting non-adaptive patterns / `0%` / `100%` density.
- Any core-count-dependent ordering in lobe clustering or connected-component indexing.

## Verification

Verified end-to-end through the public slicing API (`ares_core::slice_project`) and CLI (`ares`) against pinned OrcaSlicer 2.5.0 using strict complete-stream byte comparison (`golden::compare_ordered_bytes_generator_only`) and Native double-replay determinism checks on synthetic 3MF fixtures covering:
1. Continuous `gyroid` `lobes` across `linear`, `quadratic`, and `exponential` gradients, confirming sparse moves differ from the `disabled` baseline.
2. Continuous `gyroid` `distance_warp` and `smooth_blend`.
3. Continuous `gyroid` 2D section modes `normal_z`, `normal_y`, and `normal_x`.
4. Multi-lobe necked geometry (`NeckRatio`, `LobeReach`, `LobeMorph`) and multi-body geometry (nearest-body BFS).
5. Per-object metadata overrides, legacy `tpms_*` key renames, precedence over `gyroid_optimized`, thin-object empty-field fallback, and non-adaptive pattern (`grid`) passthrough.
6. Stepped-block internal bridge-over-infill anchoring on adaptive `gyroid`.
7. Public API rejection (`SliceError::UnsupportedProjectFeature`) for deferred active paths (`gyroid + stepped_shells`, `tpmsd + lobes`, `tpmsfk + lobes`).
