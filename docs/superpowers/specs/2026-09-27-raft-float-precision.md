# Raft float precision and layer-Z stack

## Upstream and Rust boundaries

`SupportMaterial.cpp:617-631` `SupportGridParams` keeps `grid_resolution`,
`extrusion_width`, `support_closing_radius` as **double mm**; `scale_` is
applied (and truncates to `coord_t`) only inside `create`/`create_grid`
at `:670` (`grid_resolution`), `:679-681` (`extrusion_width_scaled`,
`oversampling`, `m_pixel_size`). `BoundingBox::offset(coordf_t)`
(`BoundingBox.cpp:202-208`) constructs `Point(delta, delta)` whose double
constructor **rounds** (`Point.hpp:196`) — the bbox expansion rounds the
double `m_pixel_size` even though pixel arithmetic stays double.

`SupportCommon.cpp:287-288`: `inflate_factor_fine` and
`inflate_factor_1st_layer` are `float(scale_(…))` lattice floats — the
raft expansion path never integerizes the radius.

`SupportCommon.cpp:733-749`: the flange sheath uses `coord_t` scaled
spacing (`flow.scaled_spacing()`, `Flow.hpp:69`) and scaled width
(`scaled_width()`, `Flow.hpp:62`), both `coord_t(scale_(float-field))` —
`Flow` stores `m_width`/`m_spacing` as `float` (`Flow.cpp:129-143`), so
the mm value narrows to f32 before scaling/truncation. The fill region
shrinks by `float(-0.4 * spacing)` (`:749`).

`PrintObjectSlice.cpp:56-69` `new_layers`: object `Layer` ids start at
`slicing_parameters().raft_layers()` (`Slicing.hpp:42`) and
`print_z = hi + object_print_z_min` — `hi` is object-relative, so the
raft lift lands exactly once on `print_z` (consumed by Z-dependent fills
like CrossHatch at `Fill.cpp:1243`).

`ShortestPath.cpp:821-829`: the greedy2 flip-validity epsilon is
`SCALED_EPSILON` = `scale_(1e-4)` lattice units (`libslic3r.h:96`), not
the raw 1e-4 mm literal.

## Rust destinations

- `geometry/support_grid.rs`: `SupportGridParams` keeps `grid_resolution`
  / `extrusion_width` in mm (`f64`) plus `units_per_mm`; scaling happens
  inside `SupportGridPattern::new` with upstream's truncations.
- `project_slice/raft/polygons.rs`: `raft_expansion` /
  `first_layer_expansion` / `fine` become float lattice values (f64
  lattice, cast to f32 only at the Clipper call).
- `project_slice/raft/fills.rs`: sheath erosion and fill shrink use the
  f32-narrowed, trunc-scaled width/spacing; `-0.4·spacing` sign fixed.
- `project_slice/layers.rs`: `planned_layers_with_zaa` assigns upstream
  layer ids (`raft_layers + index`) and `print_z = hi + zmin`.
- `gcode_emit/layers/{schedule,header}.rs`: the object z-shift is no
  longer added a second time on top of the already-shifted `print_z`.
- `gcode_emit/processor/motion/planner/junction.rs`: E-axis junction
  velocities narrow to `float` before the factor multiplications
  (`GCodeProcessor.cpp:4123-4131`).
- `perimeters/classic/shortest_path/polyline_chain/advanced.rs` +
  `chain_polylines` signature: the flip-validity epsilon takes the
  coordinate scale (`1e-4 / scale.factor()`).
- `prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`:
  `AdaptiveCubic`/`SupportCubic` stay accepted (the FillAdaptive port
  provides `adaptive::anchoring_lines`).

## Deferred

- The raft wipe-E 1-ulp deltas and M73 placement drift; the octagram
  open-polyline Clipper6 ordering emulation (`classic_clip.rs`); arachne.
