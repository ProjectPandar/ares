# Adaptive infill field — stepped shells and TPMS-D / TPMS-FK slice

## Upstream source (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)

- `src/libslic3r/Fill/AdaptiveInfillField.hpp`:
  - `AdaptiveInfillShell` (`:127-131`): per-level `{density, expolygons}` pair.
  - `make_adaptive_infill_shells` (`:135-136`): splits an `ExPolygon` in the object frame into density shells from surface inwards at layer midpoint `z`.
  - `fill_adaptive_infill_shells` (`:140-141`): resets `adaptive_infill_mode = Disabled`, shrinks each shell by `-float(scale_(0.5 * spacing))`, and delegates each shrunk shell component to the pattern's single-surface filler.
- `src/libslic3r/Fill/AdaptiveInfillField.cpp`:
  - `ShellRatio = 1.5` (`:47`), `DensityLevels` (`:160-175`).
  - `marchsq::AdaptiveLevelField` (`:711-745`): `gsizef = 0.5f`, `rsizef = 0.05f`, evaluates `float(levels.level(field.depth(Vec3d(x, y, z))))` in the object frame.
  - `make_adaptive_infill_shells` (`:778-809`): computes `DensityLevels(interior_density / surface_density, ShellRatio, gradient)`, returns `{{surface_density, {expolygon}}}` when `levels.count == 0`, extracts marching-squares rings at `float(level + 0.5)`, unions rings with `FillRule::NonZero`, intersects with `remaining`, diffs out the outer shell at `float(surface_density * levels.scale(level))`, and appends any final `remaining` at `interior_density`.
  - `fill_adaptive_infill_shells` (`:811-821`): invokes `make_adaptive_infill_shells`, offsets each shell's expolygons by `-float(scale_(0.5 * spacing))` (`DefaultJoinType = jtMiter`, `DefaultMiterLimit = 3.0`), and calls `fill_shell(shell_params, part)`.
- `src/libslic3r/Fill/FillGyroid.cpp`:
  - `FillGyroid::_fill_surface_single` (`:318-324`): intercepts `AdaptiveInfillMode::SteppedShells` when `this->adaptive_infill_field != nullptr` before rotation and passes `this->z - 0.5 * params.layer_height` to `fill_adaptive_infill_shells`.
- `src/libslic3r/Fill/FillTpmsD.cpp`:
  - `schwarz_d(float, float, float)` (`:28-31`): `sin(x) * sin(y) * sin(z) - cos(x) * cos(y) * cos(z)`.
  - `FillTpmsD::_fill_surface_single` (`:120-126, 133-139`): `SteppedShells` dispatch before rotation; continuous adaptive dispatch using `frequency(d) = d * DensityAdjust / (params.multiline * this->spacing)` (`DensityAdjust = 2.1`) and bounding box padded by `scale_((params.multiline + 1) * this->spacing)`.
- `src/libslic3r/Fill/FillTpmsFK.cpp`:
  - `fischer_koch(float, float, float)` (`:27-30`): `cosf(2 * x) * sinf(y) * cosf(z) + cosf(2 * y) * sinf(z) * cosf(x) + cosf(2 * z) * sinf(x) * cosf(y)`.
  - `FillTpmsFK::_fill_surface_single` (`:135-141, 148-157`): `SteppedShells` dispatch before rotation; continuous adaptive dispatch using double-precision `period(d) = 4.18f * spacing * params.multiline / std::min(0.9f, d)`, frequencies `2. * PI / period(d)`, and bounding box padded by `scale_((params.multiline + 1) * spacing)`.
- `src/libslic3r/Fill/Fill.cpp`:
  - `Layer::make_fills` (`:1387, 1432, 1437-1439`) and `Layer::generate_sparse_infill_polylines_for_anchoring` (`:1620, 1654, 1660-1662`): pass `adaptive_infill_field`, planned `layer->height`, and projected adaptive parameters to all three adaptive TPMS patterns.

## Rust destination

- `crates/ares-core/src/fill/adaptive_infill_field/shells.rs`:
  - `AdaptiveLevelField`, `AdaptiveInfillShell`, `make_adaptive_infill_shells`, and `fill_adaptive_infill_shells`.
- `crates/ares-core/src/fill/adaptive_infill_field/tpms.rs`:
  - Shared `AdaptiveTpmsField`, `make_adaptive_tpms`, `gyroid_equation`, `schwarz_d`, `fischer_koch` (evaluating binary32 trig via `crate::scalar_math::sincosf` for cross-platform and WASM determinism), and component-level continuous adaptive fill helper.
- `crates/ares-core/src/fill/gyroid.rs`:
  - Expose single-component `fill_component` so `SteppedShells` reuses the exact regular parametric gyroid path on each already-contracted shell without re-applying `overlap - 0.5 * spacing`.
- `crates/ares-core/src/fill/tpms_d.rs` and `crates/ares-core/src/fill/tpms_fk.rs`:
  - Keep single-component regular surface fills (`fill_tpmsd_surface`, `fill_tpmsfk_surface`) reusable directly as `fill_adaptive_infill_shells` callbacks, alongside continuous adaptive component dispatch.
- `crates/ares-core/src/project_slice/fill_entities/{generate.rs, gyroid.rs, tpms_d.rs, tpms_fk.rs}`:
  - Thread `adaptive_fields.get(fill.params.adaptive_infill_mode)` and planned `(layer.print_z, layer.height)` into `gyroid`, `tpms_d`, and `tpms_fk` infill emission for both continuous modes and `SteppedShells`.
- `crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/{arms.rs, arms_tail.rs, gyroid.rs, tpms.rs}`:
  - Thread `adaptive_fields` and planned `plan.layers[layer_index].height` into `gyroid`, `tpms_d`, and `tpms_fk` bridge-over-infill sparse anchoring for both continuous modes and `SteppedShells`.
- `crates/ares-core/src/project_slice/capabilities.rs`:
  - Remove the temporary Slice 1 `validate_adaptive_infill` rejection guard and its two call sites (`validate` and `validate_settings`) while leaving unrelated guards (`brim_type`, `print_order`, `spiralinset`, `layer_config_ranges`, `filament_map_mode`) untouched.

## Included and deferred behaviour

Included:
- `SteppedShells` mode (`AdaptiveLevelField`, `make_adaptive_infill_shells`, `fill_adaptive_infill_shells`) across all three adaptive TPMS patterns (`gyroid`, `tpmsd`, `tpmsfk`) and all three gradients (`linear`, `quadratic`, `exponential`), including equal-density (`levels.count == 0`) half-spacing shell contraction and inverted (`surface < interior`) density ratios.
- Continuous adaptive dispatch (`lobes`, `distance_warp`, `smooth_blend`, `normal_z`, `normal_y`, `normal_x`) for `tpmsd` and `tpmsfk`, using deterministic `crate::scalar_math::sincosf` binary32 trig.
- Bridge-over-infill sparse anchoring (`generate_sparse_infill_polylines_for_anchoring`) for `gyroid + stepped_shells`, `tpmsd` (continuous and `stepped_shells`), and `tpmsfk` (continuous and `stepped_shells`).
- Empty-field fallback (`AdaptiveInfillField::empty()`) to regular infill for thin parts across all three patterns and modes.
- Lifting the deferred `validate_adaptive_infill` admission rejections in `capabilities.rs`.

Deferred:
- None within `AdaptiveInfillField` (`Fill/AdaptiveInfillField.{hpp,cpp}` and its `FillGyroid` / `FillTpmsD` / `FillTpmsFK` integration are complete at `40ecc66f7`).

## Failure modes

- Computing `interior_density / surface_density` in `f64` instead of `f32` widened to `f64` in `make_adaptive_infill_shells`, shifting `DensityLevels::count` or `levels.scale(level)` near threshold ratios.
- Short-circuiting `fill_adaptive_infill_shells` when `levels.count == 0` instead of still shrinking `shell.expolygons` by `-float(scale_(0.5 * spacing))` before calling `fill_shell`.
- Re-applying `Fill::fill_surface`'s `overlap - 0.5 * spacing` contraction inside each `SteppedShells` callback instead of calling the single-component filler directly on each shrunk shell part.
- Running `join_close_ends(..., 2 * spacing, ...)` per shell instead of once per outer `surface_fill` expolygon after all shells append to `polylines_out`.
- Closing `AdaptiveLevelField` marching-squares rings with a duplicated end vertex or unioning them with `EvenOdd` instead of `NonZero` before intersecting with `remaining`.
- Using `flow.height` (combined infill height) instead of planned `layer.height` when computing the layer midpoint `z - 0.5 * layer_height` in `fill_entities` or `sparse_anchoring`.
- Narrowing `FillTpmsFK`'s `period(density)` lambda result to `f32` before dividing `2.0 * PI / period` in continuous adaptive mode (upstream deduces `double` from `4.18f * spacing * params.multiline / std::min(0.9f, density)`).
- Padding `FillTpmsD` or `FillTpmsFK` continuous adaptive bounding boxes by `multiline * spacing` instead of `(multiline + 1) * spacing`.
- Evaluating `fischer_koch` or `schwarz_d` via platform `f32::sin`/`f32::cos` instead of `crate::scalar_math::sincosf`, causing 1-ULP divergence on WASM versus Linux glibc.
- Failing to suppress `gyroid_optimized` when `adaptive_infill_mode == SteppedShells`, causing recursive `fill_shell` calls with `adaptive_infill_mode = Disabled` to enter the optimized marching-squares gyroid branch instead of parametric waves.

## Verification

Verified end-to-end through the public slicing API (`ares_core::slice_project`) and CLI (`ares`) against pinned OrcaSlicer 2.5.0 (both Official and Seeded lanes) using strict complete-stream byte comparison (`golden::compare_ordered_bytes_generator_only`) and Native double-replay determinism checks on synthetic 3MF fixtures covering:
1. `gyroid + stepped_shells` across `linear`, `quadratic`, and `exponential` gradients, equal density (`30%`/`30%`), inverted density (`5%`/`30%`), `gyroid_optimized` precedence, and stepped-block internal bridge-over-infill anchoring.
2. `tpmsd` continuous modes (`lobes` across `linear`/`quadratic`/`exponential`, `distance_warp`, `smooth_blend`, `normal_z`, `normal_y`, `normal_x`), `tpmsd + stepped_shells`, stepped-block bridge-over-infill anchoring (both `lobes` and `stepped_shells`), and thin-part empty-field fallback.
3. `tpmsfk` continuous modes (`lobes` across `linear`/`quadratic`/`exponential`, `distance_warp`, `smooth_blend`, `normal_z`, `normal_y`, `normal_x`), `tpmsfk + stepped_shells`, stepped-block bridge-over-infill anchoring (both `lobes` and `stepped_shells`), and thin-part empty-field fallback.
