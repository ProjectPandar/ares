# Floating-point arithmetic call sites

Companion to the [determinism audit](transcendental-determinism-audit.md) and
[transcendental call-site inventory](transcendental-determinism-sites.md).
Line numbers refer to the same Ares revision.

Explicit `sqrt` and `mul_add` are IEEE-rounded for the Rust form; match
upstream widths, fusion and operation order before relying on this fact.
Fixed squares and `powi` are indexed separately because a general integer
power's multiplication order is not an IEEE-defined primitive. Of particular
interest are decimal-format powers of ten and positive/negative powers of
two in the adaptive pressure advance number parser. The scale and format
semantics must be preserved. Basic arithmetic, `abs` and integral
rounding are pervasive and do not need a target libm replacement.

| Rust source | `sqrt` lines | `mul_add` lines | `powi` lines |
|---|---|---|---|
| [`arachne/extrusion_line.rs`](../../crates/ares-core/src/arachne/extrusion_line.rs#L258) | 273, 291 | — | 258 |
| [`arachne/skeletal/graph.rs`](../../crates/ares-core/src/arachne/skeletal/graph.rs#L298) | 298 | — | — |
| [`arachne/trapezoidation/discretize.rs`](../../crates/ares-core/src/arachne/trapezoidation/discretize.rs#L107) | 107, 176 | — | — |
| [`arachne/trapezoidation/filtering.rs`](../../crates/ares-core/src/arachne/trapezoidation/filtering.rs#L243) | 243 | — | — |
| [`arachne/trapezoidation/transitions.rs`](../../crates/ares-core/src/arachne/trapezoidation/transitions.rs#L94) | 94 | — | — |
| [`arachne/trapezoidation/voronoi.rs`](../../crates/ares-core/src/arachne/trapezoidation/voronoi.rs#L248) | 248 | — | — |
| [`arachne/wall_toolpaths/outline.rs`](../../crates/ares-core/src/arachne/wall_toolpaths/outline.rs#L129) | 129, 131, 254 | — | 129, 131, 203 |
| [`arachne/wall_toolpaths/outline/degenerate.rs`](../../crates/ares-core/src/arachne/wall_toolpaths/outline/degenerate.rs#L44) | 44, 46 | — | — |
| [`arachne/wall_toolpaths/outline/self_intersections.rs`](../../crates/ares-core/src/arachne/wall_toolpaths/outline/self_intersections.rs#L50) | 50 | — | — |
| [`arachne/wall_toolpaths/postprocess.rs`](../../crates/ares-core/src/arachne/wall_toolpaths/postprocess.rs#L96) | 96 | — | — |
| [`arachne/wall_toolpaths/stitch.rs`](../../crates/ares-core/src/arachne/wall_toolpaths/stitch.rs#L270) | 270 | — | — |
| [`arrangement/arranger.rs`](../../crates/ares-core/src/arrangement/arranger.rs#L59) | 59, 272 | — | — |
| [`arrangement/nest/edges.rs`](../../crates/ares-core/src/arrangement/nest/edges.rs#L27) | 27 | — | — |
| [`arrangement/nest/geometry.rs`](../../crates/ares-core/src/arrangement/nest/geometry.rs#L143) | 143 | — | — |
| [`arrangement/rotcalipers.rs`](../../crates/ares-core/src/arrangement/rotcalipers.rs#L109) | 109 | — | — |
| [`brims/ears/paths.rs`](../../crates/ares-core/src/brims/ears/paths.rs#L123) | — | 123 | — |
| [`fill/adaptive/filler.rs`](../../crates/ares-core/src/fill/adaptive/filler.rs#L101) | 101 | — | — |
| [`fill/adaptive/hooks.rs`](../../crates/ares-core/src/fill/adaptive/hooks.rs#L81) | 81 | — | — |
| [`fill/adaptive/hooks/geometry.rs`](../../crates/ares-core/src/fill/adaptive/hooks/geometry.rs#L9) | 9, 74, 111 | — | — |
| [`fill/adaptive/hooks/touching.rs`](../../crates/ares-core/src/fill/adaptive/hooks/touching.rs#L83) | 83, 84 | — | — |
| [`fill/adaptive/hooks/trim.rs`](../../crates/ares-core/src/fill/adaptive/hooks/trim.rs#L36) | 36 | — | — |
| [`fill/adaptive/multiline/math.rs`](../../crates/ares-core/src/fill/adaptive/multiline/math.rs#L16) | 16, 46 | — | — |
| [`fill/adaptive/octree.rs`](../../crates/ares-core/src/fill/adaptive/octree.rs#L93) | 93, 94, 95, 96, 108, 109, 110, 111 | — | — |
| [`fill/adaptive/octree/frame.rs`](../../crates/ares-core/src/fill/adaptive/octree/frame.rs#L71) | 71 | — | — |
| [`fill/base_support/arches.rs`](../../crates/ares-core/src/fill/base_support/arches.rs#L110) | 110, 114, 117, 120, 175, 183 | — | — |
| [`fill/base_support/caps.rs`](../../crates/ares-core/src/fill/base_support/caps.rs#L157) | 157, 164 | — | — |
| [`fill/base_support/mod.rs`](../../crates/ares-core/src/fill/base_support/mod.rs#L283) | 283 | — | — |
| [`fill/connect/collision.rs`](../../crates/ares-core/src/fill/connect/collision.rs#L40) | 40, 199, 237, 280 | — | — |
| [`fill/connect/contour.rs`](../../crates/ares-core/src/fill/connect/contour.rs#L389) | 389 | — | — |
| [`fill/connect/hooks.rs`](../../crates/ares-core/src/fill/connect/hooks.rs#L79) | 91, 105 | — | 79 |
| [`fill/corner_smoothing/shapes.rs`](../../crates/ares-core/src/fill/corner_smoothing/shapes.rs#L62) | 62, 64, 74 | — | — |
| [`fill/corner_smoothing/vector.rs`](../../crates/ares-core/src/fill/corner_smoothing/vector.rs#L17) | 17 | — | — |
| [`fill/cubic_trapezoidal.rs`](../../crates/ares-core/src/fill/cubic_trapezoidal.rs#L25) | 25, 52 | — | — |
| [`fill/cubic_trapezoidal/envelope.rs`](../../crates/ares-core/src/fill/cubic_trapezoidal/envelope.rs#L3) | 3 | — | — |
| [`fill/cubic_trapezoidal/rows.rs`](../../crates/ares-core/src/fill/cubic_trapezoidal/rows.rs#L18) | 18 | — | — |
| [`fill/gyroid_field.rs`](../../crates/ares-core/src/fill/gyroid_field.rs#L21) | 21, 22 | — | — |
| [`fill/gyroid.rs`](../../crates/ares-core/src/fill/gyroid.rs#L206) | 206, 218 | — | — |
| [`fill/honeycomb.rs`](../../crates/ares-core/src/fill/honeycomb.rs#L143) | 143, 147, 149 | — | — |
| [`fill/lightning/distance_field.rs`](../../crates/ares-core/src/fill/lightning/distance_field.rs#L183) | 183, 259, 260 | — | — |
| [`fill/lightning/layer_geom/geometry.rs`](../../crates/ares-core/src/fill/lightning/layer_geom/geometry.rs#L86) | 91 | — | 86, 91 |
| [`fill/lightning/tree_geom.rs`](../../crates/ares-core/src/fill/lightning/tree_geom.rs#L122) | 171 | — | 122, 171 |
| [`fill/lightning/tree_node.rs`](../../crates/ares-core/src/fill/lightning/tree_node.rs#L65) | 67, 228 | — | 65, 66 |
| [`fill/lightning/tree_ops.rs`](../../crates/ares-core/src/fill/lightning/tree_ops.rs#L75) | 75, 201, 234 | — | — |
| [`fill/plane_path/generate.rs`](../../crates/ares-core/src/fill/plane_path/generate.rs#L138) | 138, 139, 160, 161, 163, 168 | — | — |
| [`fill/rectilinear/costs.rs`](../../crates/ares-core/src/fill/rectilinear/costs.rs#L139) | 139 | — | — |
| [`fill/rectilinear/path_matrix.rs`](../../crates/ares-core/src/fill/rectilinear/path_matrix.rs#L73) | 73 | — | — |
| [`fill/rectilinear/perimeter.rs`](../../crates/ares-core/src/fill/rectilinear/perimeter.rs#L173) | 173 | — | — |
| [`fill/rectilinear/segments.rs`](../../crates/ares-core/src/fill/rectilinear/segments.rs#L180) | — | — | 180 |
| [`fill/rectilinear/segments/cleanup.rs`](../../crates/ares-core/src/fill/rectilinear/segments/cleanup.rs#L71) | — | — | 71, 72 |
| [`fill/spiral_inset/chain.rs`](../../crates/ares-core/src/fill/spiral_inset/chain.rs#L60) | 60, 155, 185, 207 | — | — |
| [`fill/spiral_inset/walls.rs`](../../crates/ares-core/src/fill/spiral_inset/walls.rs#L29) | 29 | — | — |
| [`fill/stars_trapezoidal.rs`](../../crates/ares-core/src/fill/stars_trapezoidal.rs#L24) | 24 | — | — |
| [`fill/stars_trapezoidal/rows.rs`](../../crates/ares-core/src/fill/stars_trapezoidal/rows.rs#L5) | 5, 8, 9 | — | — |
| [`fill/three_d_honeycomb.rs`](../../crates/ares-core/src/fill/three_d_honeycomb.rs#L112) | 112, 203 | — | — |
| [`fill/three_d_honeycomb/grid.rs`](../../crates/ares-core/src/fill/three_d_honeycomb/grid.rs#L109) | 109, 159, 172 | — | — |
| [`fill/three_d_honeycomb/tops.rs`](../../crates/ares-core/src/fill/three_d_honeycomb/tops.rs#L59) | 59, 73 | — | — |
| [`fill/tpms_d.rs`](../../crates/ares-core/src/fill/tpms_d.rs#L246) | 246 | — | — |
| [`fill/tpms_fk.rs`](../../crates/ares-core/src/fill/tpms_fk.rs#L198) | 198 | — | — |
| [`fill/triangles_trapezoidal.rs`](../../crates/ares-core/src/fill/triangles_trapezoidal.rs#L23) | 23 | — | — |
| [`fill/triangles_trapezoidal/rows.rs`](../../crates/ares-core/src/fill/triangles_trapezoidal/rows.rs#L5) | 5, 6 | — | — |
| [`gcode_processor/statistics.rs`](../../crates/ares-core/src/gcode_processor/statistics.rs#L134) | — | — | 134 |
| [`gcode_spiral_vase/geometry.rs`](../../crates/ares-core/src/gcode_spiral_vase/geometry.rs#L6) | 6, 10 | — | 6, 10 |
| [`gcode_spiral_vase/project.rs`](../../crates/ares-core/src/gcode_spiral_vase/project.rs#L247) | 247 | — | — |
| [`geometry/bridge_direction.rs`](../../crates/ares-core/src/geometry/bridge_direction.rs#L103) | 103 | — | — |
| [`geometry/bridge_direction/principal_components.rs`](../../crates/ares-core/src/geometry/bridge_direction/principal_components.rs#L19) | 19, 121 | — | — |
| [`geometry/chain_points/exact.rs`](../../crates/ares-core/src/geometry/chain_points/exact.rs#L85) | 85 | — | — |
| [`geometry/clipper/variable_offset.rs`](../../crates/ares-core/src/geometry/clipper/variable_offset.rs#L216) | 216 | — | — |
| [`geometry/clipper/z_clip.rs`](../../crates/ares-core/src/geometry/clipper/z_clip.rs#L45) | 45, 81, 82 | — | — |
| [`geometry/clipper2/core/ellipse_point64.rs`](../../crates/ares-core/src/geometry/clipper2/core/ellipse_point64.rs#L16) | 16 | — | — |
| [`geometry/clipper2/offset/floating_point_tolerance.rs`](../../crates/ares-core/src/geometry/clipper2/offset/floating_point_tolerance.rs#L91) | 91 | — | — |
| [`geometry/edge_grid/query.rs`](../../crates/ares-core/src/geometry/edge_grid/query.rs#L55) | 55, 76 | — | — |
| [`geometry/enclosing_circle.rs`](../../crates/ares-core/src/geometry/enclosing_circle.rs#L71) | 71, 114 | — | — |
| [`geometry/line.rs`](../../crates/ares-core/src/geometry/line.rs#L17) | 17, 53, 57, 62, 66 | — | — |
| [`geometry/medial_axis.rs`](../../crates/ares-core/src/geometry/medial_axis.rs#L213) | 213 | — | — |
| [`geometry/medial_axis/validate.rs`](../../crates/ares-core/src/geometry/medial_axis/validate.rs#L91) | 91 | — | — |
| [`geometry/mesh_normals.rs`](../../crates/ares-core/src/geometry/mesh_normals.rs#L19) | 19, 29 | — | — |
| [`geometry/mutable_polygon.rs`](../../crates/ares-core/src/geometry/mutable_polygon.rs#L50) | 86, 87, 101, 104 | — | 50 |
| [`geometry/mutable_polygon/duplicates.rs`](../../crates/ares-core/src/geometry/mutable_polygon/duplicates.rs#L14) | — | — | 14, 23 |
| [`geometry/mutable_polygon/narrow.rs`](../../crates/ares-core/src/geometry/mutable_polygon/narrow.rs#L124) | 124, 132, 133 | — | — |
| [`geometry/point_ordering.rs`](../../crates/ares-core/src/geometry/point_ordering.rs#L95) | 95 | — | — |
| [`geometry/point_ordering/crossing_grid.rs`](../../crates/ares-core/src/geometry/point_ordering/crossing_grid.rs#L39) | 39 | — | — |
| [`geometry/point_ordering/nearest_points.rs`](../../crates/ares-core/src/geometry/point_ordering/nearest_points.rs#L29) | 29 | — | — |
| [`geometry/point_ordering/or_opt.rs`](../../crates/ares-core/src/geometry/point_ordering/or_opt.rs#L110) | 110 | — | — |
| [`geometry/point_ordering/two_opt.rs`](../../crates/ares-core/src/geometry/point_ordering/two_opt.rs#L19) | 19, 67 | — | — |
| [`geometry/polygon.rs`](../../crates/ares-core/src/geometry/polygon.rs#L15) | 15 | — | — |
| [`geometry/polyline.rs`](../../crates/ares-core/src/geometry/polyline.rs#L61) | 61, 68 | — | — |
| [`geometry/quickhull.rs`](../../crates/ares-core/src/geometry/quickhull.rs#L31) | 31, 78, 95 | — | — |
| [`geometry/rotation_scaling.rs`](../../crates/ares-core/src/geometry/rotation_scaling.rs#L79) | 79, 89, 95 | — | — |
| [`mesh_slicer/chaining.rs`](../../crates/ares-core/src/mesh_slicer/chaining.rs#L230) | 230 | — | — |
| [`mesh_slicer/chaining/gaps.rs`](../../crates/ares-core/src/mesh_slicer/chaining/gaps.rs#L138) | 138 | — | — |
| [`mesh_slicer/chaining/gaps/spatial.rs`](../../crates/ares-core/src/mesh_slicer/chaining/gaps/spatial.rs#L73) | — | — | 73, 179 |
| [`mesh_slicer/coplanar.rs`](../../crates/ares-core/src/mesh_slicer/coplanar.rs#L37) | 37 | — | — |
| [`mesh_slicer/slabs.rs`](../../crates/ares-core/src/mesh_slicer/slabs.rs#L97) | 97 | — | — |
| [`options/bed_exclusion/intersection.rs`](../../crates/ares-core/src/options/bed_exclusion/intersection.rs#L196) | 196 | — | — |
| [`options/flush_volume/color_distance.rs`](../../crates/ares-core/src/options/flush_volume/color_distance.rs#L30) | 30, 31, 35, 38, 39, 60, 81, 83, 88 | — | — |
| [`options/flush_volume/mod.rs`](../../crates/ares-core/src/options/flush_volume/mod.rs#L90) | 90, 102 | — | — |
| [`options/legacy/wiping_volumes.rs`](../../crates/ares-core/src/options/legacy/wiping_volumes.rs#L18) | 18 | — | — |
| [`options/pellet.rs`](../../crates/ares-core/src/options/pellet.rs#L32) | 32 | — | — |
| [`options/registry/definitions/metadata.rs`](../../crates/ares-core/src/options/registry/definitions/metadata.rs#L15346) | 15346 | — | — |
| [`perimeters/fuzzy_skin_noise.rs`](../../crates/ares-core/src/perimeters/fuzzy_skin_noise.rs#L25) | 25, 111, 128, 189, 211, 316 | — | — |
| [`perimeters/fuzzy_skin_seeded.rs`](../../crates/ares-core/src/perimeters/fuzzy_skin_seeded.rs#L33) | 33 | — | — |
| [`perimeters/seams.rs`](../../crates/ares-core/src/perimeters/seams.rs#L110) | 110 | — | 110 |
| [`perimeters/simplification.rs`](../../crates/ares-core/src/perimeters/simplification.rs#L52) | 52 | — | — |
| [`perimeters/thin_walls.rs`](../../crates/ares-core/src/perimeters/thin_walls.rs#L81) | 81 | — | — |
| [`plater/auto_orient/candidates.rs`](../../crates/ares-core/src/plater/auto_orient/candidates.rs#L37) | — | — | 37 |
| [`plater/auto_orient/score.rs`](../../crates/ares-core/src/plater/auto_orient/score.rs#L93) | 93 | — | — |
| [`plater/eigen.rs`](../../crates/ares-core/src/plater/eigen.rs#L67) | 67, 86, 108, 144, 153, 158, 166, 239 | — | — |
| [`plater/transformation.rs`](../../crates/ares-core/src/plater/transformation.rs#L81) | 81 | — | — |
| [`profiles/filament_slots.rs`](../../crates/ares-core/src/profiles/filament_slots.rs#L55) | 55 | — | — |
| [`project_slice/centering/rotation.rs`](../../crates/ares-core/src/project_slice/centering/rotation.rs#L27) | 27, 44 | — | — |
| [`project_slice/contour_z/mesh.rs`](../../crates/ares-core/src/project_slice/contour_z/mesh.rs#L109) | 109 | — | — |
| [`project_slice/contour_z/path.rs`](../../crates/ares-core/src/project_slice/contour_z/path.rs#L43) | 43, 99 | — | — |
| [`project_slice/elephant_foot/distance.rs`](../../crates/ares-core/src/project_slice/elephant_foot/distance.rs#L217) | 217, 324 | — | — |
| [`project_slice/elephant_foot/profile.rs`](../../crates/ares-core/src/project_slice/elephant_foot/profile.rs#L58) | 58, 63 | — | — |
| [`project_slice/extrusion_islands/volume.rs`](../../crates/ares-core/src/project_slice/extrusion_islands/volume.rs#L140) | 140, 166 | — | — |
| [`project_slice/fill_entities/adaptive/body_mesh.rs`](../../crates/ares-core/src/project_slice/fill_entities/adaptive/body_mesh.rs#L172) | 172 | — | — |
| [`project_slice/fill_entities/gap_residual/insert.rs`](../../crates/ares-core/src/project_slice/fill_entities/gap_residual/insert.rs#L154) | 154, 199 | — | — |
| [`project_slice/gcode_emit/adaptive_pa/number.rs`](../../crates/ares-core/src/project_slice/gcode_emit/adaptive_pa/number.rs#L77) | — | — | 77, 82, 84, 88 |
| [`project_slice/gcode_emit/brim/auto.rs`](../../crates/ares-core/src/project_slice/gcode_emit/brim/auto.rs#L92) | — | — | 92, 133, 134 |
| [`project_slice/gcode_emit/conflict/intersect.rs`](../../crates/ares-core/src/project_slice/gcode_emit/conflict/intersect.rs#L43) | 43 | — | — |
| [`project_slice/gcode_emit/cooling/feedrate/parse.rs`](../../crates/ares-core/src/project_slice/gcode_emit/cooling/feedrate/parse.rs#L240) | 240, 262, 264 | — | — |
| [`project_slice/gcode_emit/fan_mover/motion.rs`](../../crates/ares-core/src/project_slice/gcode_emit/fan_mover/motion.rs#L28) | 28 | — | — |
| [`project_slice/gcode_emit/footprint/first_layer.rs`](../../crates/ares-core/src/project_slice/gcode_emit/footprint/first_layer.rs#L52) | — | — | 52 |
| [`project_slice/gcode_emit/layers/farthest.rs`](../../crates/ares-core/src/project_slice/gcode_emit/layers/farthest.rs#L95) | 95 | — | — |
| [`project_slice/gcode_emit/motion/arc/simplify.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/arc/simplify.rs#L158) | — | — | 158 |
| [`project_slice/gcode_emit/motion/auto_lift.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/auto_lift.rs#L110) | 110 | — | — |
| [`project_slice/gcode_emit/motion/clip.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/clip.rs#L29) | 29, 36 | — | — |
| [`project_slice/gcode_emit/motion/format.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/format.rs#L43) | — | — | 43 |
| [`project_slice/gcode_emit/motion/loop_paths.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths.rs#L307) | 307 | — | — |
| [`project_slice/gcode_emit/motion/loop_paths/adaptive_pa.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/adaptive_pa.rs#L64) | 64, 80, 88 | — | — |
| [`project_slice/gcode_emit/motion/loop_paths/inward.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/inward.rs#L158) | 158, 216, 285 | — | — |
| [`project_slice/gcode_emit/motion/loop_paths/small_perimeter.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/small_perimeter.rs#L16) | 16 | — | — |
| [`project_slice/gcode_emit/motion/loop_paths/wipe_inward/math.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/wipe_inward/math.rs#L41) | 41, 133, 159, 162 | — | — |
| [`project_slice/gcode_emit/motion/loop_paths/wipe_inward/side.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/wipe_inward/side.rs#L50) | 50 | — | — |
| [`project_slice/gcode_emit/motion/overhang/curl.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/overhang/curl.rs#L241) | 241, 284, 309 | — | — |
| [`project_slice/gcode_emit/motion/overhang/curl/geometry.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/overhang/curl/geometry.rs#L36) | 36, 38, 90, 328 | — | 112 |
| [`project_slice/gcode_emit/motion/overhang/sections.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/overhang/sections.rs#L139) | 139, 141, 199 | — | — |
| [`project_slice/gcode_emit/motion/path.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path.rs#L118) | 118 | — | — |
| [`project_slice/gcode_emit/motion/path/avoid_crossing/boundary.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/boundary.rs#L107) | 107, 224 | — | — |
| [`project_slice/gcode_emit/motion/path/avoid_crossing/boundary/resample.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/boundary/resample.rs#L26) | 26, 45, 215 | — | 113 |
| [`project_slice/gcode_emit/motion/path/avoid_crossing/router/path_shape.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/router/path_shape.rs#L32) | 32 | — | — |
| [`project_slice/gcode_emit/motion/path/contour.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path/contour.rs#L26) | 26, 34, 52 | — | — |
| [`project_slice/gcode_emit/motion/path/variable.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/path/variable.rs#L67) | 68 | — | 67 |
| [`project_slice/gcode_emit/motion/scarf.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/scarf.rs#L322) | 322, 330 | — | — |
| [`project_slice/gcode_emit/motion/scarf/eligibility.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/scarf/eligibility.rs#L52) | 52 | — | — |
| [`project_slice/gcode_emit/motion/scarf/split.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/scarf/split.rs#L15) | 15, 59 | — | — |
| [`project_slice/gcode_emit/motion/scarf/trim.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/scarf/trim.rs#L91) | 91 | — | — |
| [`project_slice/gcode_emit/motion/support.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/support.rs#L158) | 158, 167, 202 | — | — |
| [`project_slice/gcode_emit/motion/travel/lift.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/travel/lift.rs#L224) | 224 | — | — |
| [`project_slice/gcode_emit/motion/travel/lift/spiral.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/travel/lift/spiral.rs#L36) | 36 | — | — |
| [`project_slice/gcode_emit/motion/travel/wipe_path.rs`](../../crates/ares-core/src/project_slice/gcode_emit/motion/travel/wipe_path.rs#L77) | 77, 84, 114 | — | — |
| [`project_slice/gcode_emit/processor/arc_accounting.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/arc_accounting.rs#L52) | 52, 54 | — | — |
| [`project_slice/gcode_emit/processor/material_statistics.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/material_statistics.rs#L42) | — | — | 42 |
| [`project_slice/gcode_emit/processor/motion_util.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion_util.rs#L67) | 67 | — | — |
| [`project_slice/gcode_emit/processor/motion/arc.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/arc.rs#L20) | 20, 82, 174, 176 | — | — |
| [`project_slice/gcode_emit/processor/motion/limits.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/limits.rs#L67) | 67 | — | — |
| [`project_slice/gcode_emit/processor/motion/parse.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/parse.rs#L128) | 128, 146, 154 | — | 146 |
| [`project_slice/gcode_emit/processor/motion/planner/centripetal.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/centripetal.rs#L16) | 16, 21, 22, 24, 31 | — | — |
| [`project_slice/gcode_emit/processor/motion/planner/junction.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/junction.rs#L29) | 29, 33, 62, 150 | — | — |
| [`project_slice/gcode_emit/processor/motion/planner/klipper/mod.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/klipper/mod.rs#L109) | 109, 110, 111 | — | — |
| [`project_slice/gcode_emit/processor/motion/planner/klipper/prepare.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/klipper/prepare.rs#L16) | 16, 83, 111, 140, 141, 151, 155 | — | — |
| [`project_slice/gcode_emit/processor/motion/planner/mod.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/mod.rs#L190) | 190, 201 | — | — |
| [`project_slice/gcode_emit/processor/motion/planner/trapezoid.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/trapezoid.rs#L10) | 10 | — | — |
| [`project_slice/gcode_emit/processor/motion/segment.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/motion/segment.rs#L20) | 20 | — | — |
| [`project_slice/gcode_emit/processor/moves.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/moves.rs#L276) | 276 | — | — |
| [`project_slice/gcode_emit/processor/moves/actual_speed.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/moves/actual_speed.rs#L175) | 175 | — | — |
| [`project_slice/gcode_emit/processor/moves/dynamics.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/moves/dynamics.rs#L87) | 87 | — | — |
| [`project_slice/gcode_emit/processor/moves/extrusion.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/moves/extrusion.rs#L56) | 56 | — | — |
| [`project_slice/gcode_emit/processor/used_filaments.rs`](../../crates/ares-core/src/project_slice/gcode_emit/processor/used_filaments.rs#L223) | — | — | 223 |
| [`project_slice/gcode_emit/skirt/generate.rs`](../../crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs#L99) | — | — | 99 |
| [`project_slice/gcode_emit/skirt/geometry.rs`](../../crates/ares-core/src/project_slice/gcode_emit/skirt/geometry.rs#L39) | 39 | — | 39 |
| [`project_slice/gcode_emit/tool_ordering/collect.rs`](../../crates/ares-core/src/project_slice/gcode_emit/tool_ordering/collect.rs#L195) | 195 | — | — |
| [`project_slice/gcode_emit/tool_ordering/wiping/source.rs`](../../crates/ares-core/src/project_slice/gcode_emit/tool_ordering/wiping/source.rs#L166) | 166 | — | — |
| [`project_slice/gcode_emit/tool_transition/change/dynamic/mod.rs`](../../crates/ares-core/src/project_slice/gcode_emit/tool_transition/change/dynamic/mod.rs#L118) | — | — | 118 |
| [`project_slice/gcode_emit/wipe_tower/estimate/footprint.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/estimate/footprint.rs#L155) | 155 | — | — |
| [`project_slice/gcode_emit/wipe_tower/estimate/statics.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/estimate/statics.rs#L158) | 158, 161, 165, 188, 207, 210 | — | — |
| [`project_slice/gcode_emit/wipe_tower/filament.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/filament.rs#L55) | — | — | 55, 84 |
| [`project_slice/gcode_emit/wipe_tower/generate.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/generate.rs#L51) | 51, 136, 144 | — | — |
| [`project_slice/gcode_emit/wipe_tower/geometry.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry.rs#L66) | 66, 81, 141 | — | 66 |
| [`project_slice/gcode_emit/wipe_tower/geometry/bottom.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry/bottom.rs#L56) | 56 | — | — |
| [`project_slice/gcode_emit/wipe_tower/geometry/gaps.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry/gaps.rs#L20) | 20, 41, 142, 143, 177 | — | 20 |
| [`project_slice/gcode_emit/wipe_tower/geometry/rounding.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry/rounding.rs#L100) | 100 | — | — |
| [`project_slice/gcode_emit/wipe_tower/prime.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/prime.rs#L21) | 23 | — | 21, 110, 111, 114 |
| [`project_slice/gcode_emit/wipe_tower/result.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/result.rs#L35) | 35 | — | — |
| [`project_slice/gcode_emit/wipe_tower/route.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/route.rs#L150) | — | — | 150 |
| [`project_slice/gcode_emit/wipe_tower/type1/dynamic.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/dynamic.rs#L46) | — | — | 46 |
| [`project_slice/gcode_emit/wipe_tower/type1/filament.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/filament.rs#L33) | — | — | 33, 35 |
| [`project_slice/gcode_emit/wipe_tower/type1/plan.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/plan.rs#L111) | 288, 309, 325 | — | 111 |
| [`project_slice/gcode_emit/wipe_tower/type1/result.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/result.rs#L81) | 81 | — | — |
| [`project_slice/gcode_emit/wipe_tower/type1/writer.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/writer.rs#L106) | 106, 177, 218 | — | — |
| [`project_slice/gcode_emit/wipe_tower/type1/writer/path.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/writer/path.rs#L122) | 122, 191 | — | — |
| [`project_slice/gcode_emit/wipe_tower/writer/moves.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/writer/moves.rs#L31) | 31, 52, 183 | — | — |
| [`project_slice/gcode_emit/wipe_tower/writer/path.rs`](../../crates/ares-core/src/project_slice/gcode_emit/wipe_tower/writer/path.rs#L43) | 43 | — | — |
| [`project_slice/hole_to_polyhole/detect.rs`](../../crates/ares-core/src/project_slice/hole_to_polyhole/detect.rs#L102) | 102 | — | — |
| [`project_slice/perimeters/arachne/fuzzy/junctions.rs`](../../crates/ares-core/src/project_slice/perimeters/arachne/fuzzy/junctions.rs#L43) | 43 | — | — |
| [`project_slice/perimeters/arachne/fuzzy/ripple.rs`](../../crates/ares-core/src/project_slice/perimeters/arachne/fuzzy/ripple.rs#L27) | 27, 42, 113 | — | — |
| [`project_slice/perimeters/arachne/order.rs`](../../crates/ares-core/src/project_slice/perimeters/arachne/order.rs#L75) | 75 | — | — |
| [`project_slice/perimeters/arachne/sandwich.rs`](../../crates/ares-core/src/project_slice/perimeters/arachne/sandwich.rs#L299) | 299 | — | — |
| [`project_slice/perimeters/arachne/variable_width.rs`](../../crates/ares-core/src/project_slice/perimeters/arachne/variable_width.rs#L199) | 199 | — | — |
| [`project_slice/perimeters/classic/extra_perimeters/mod.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/extra_perimeters/mod.rs#L229) | 229 | — | — |
| [`project_slice/perimeters/classic/extra_perimeters/ordering.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/extra_perimeters/ordering.rs#L86) | 86, 182 | — | — |
| [`project_slice/perimeters/classic/gap_extrusion/variable_width.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/variable_width.rs#L172) | 172 | — | — |
| [`project_slice/perimeters/classic/shortest_path/improve.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/shortest_path/improve.rs#L45) | 45 | — | — |
| [`project_slice/perimeters/classic/shortest_path/improve/nearest.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/shortest_path/improve/nearest.rs#L24) | 24 | — | 123 |
| [`project_slice/perimeters/classic/shortest_path/polyline_chain/advanced/types.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/shortest_path/polyline_chain/advanced/types.rs#L171) | 171 | — | — |
| [`project_slice/perimeters/classic/shortest_path/polyline_chain/two_exchange.rs`](../../crates/ares-core/src/project_slice/perimeters/classic/shortest_path/polyline_chain/two_exchange.rs#L302) | 302 | — | — |
| [`project_slice/perimeters/flow.rs`](../../crates/ares-core/src/project_slice/perimeters/flow.rs#L153) | 153, 309 | — | — |
| [`project_slice/perimeters/unsupported.rs`](../../crates/ares-core/src/project_slice/perimeters/unsupported.rs#L37) | 37 | — | — |
| [`project_slice/prepare_infill/bridge_over_infill/automatic_bridge_angle.rs`](../../crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/automatic_bridge_angle.rs#L81) | 81, 88 | — | — |
| [`project_slice/prepare_infill/external_surfaces/parameters.rs`](../../crates/ares-core/src/project_slice/prepare_infill/external_surfaces/parameters.rs#L39) | 39 | — | — |
| [`project_slice/pressure_equalizer/emitter.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/emitter.rs#L79) | — | — | 79 |
| [`project_slice/pressure_equalizer/emitter/split.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/emitter/split.rs#L38) | 51 | — | 38, 39, 40 |
| [`project_slice/pressure_equalizer/limiter.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/limiter.rs#L177) | 177, 191 | — | — |
| [`project_slice/pressure_equalizer/line.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/line.rs#L89) | 89 | — | — |
| [`project_slice/pressure_equalizer/line/commands.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/line/commands.rs#L76) | 76 | — | — |
| [`project_slice/pressure_equalizer/pass.rs`](../../crates/ares-core/src/project_slice/pressure_equalizer/pass.rs#L147) | 147 | — | — |
| [`project_slice/raft/fill_params.rs`](../../crates/ares-core/src/project_slice/raft/fill_params.rs#L32) | — | — | 32 |
| [`project_slice/raft/fills/geom.rs`](../../crates/ares-core/src/project_slice/raft/fills/geom.rs#L15) | 15, 25 | — | — |
| [`project_slice/raft/fills/tree.rs`](../../crates/ares-core/src/project_slice/raft/fills/tree.rs#L282) | 282 | — | — |
| [`project_slice/region_slices/painted/graph/vertices.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/graph/vertices.rs#L70) | — | — | 70, 75, 79 |
| [`project_slice/region_slices/painted/math.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/math.rs#L33) | 33 | — | — |
| [`project_slice/region_slices/painted/painted_lines.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/painted_lines.rs#L61) | 61, 69, 81, 85, 141, 147, 162 | — | — |
| [`project_slice/region_slices/painted/preprocess.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/preprocess.rs#L32) | — | — | 32 |
| [`project_slice/region_slices/painted/projection_visitor.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/projection_visitor.rs#L39) | — | — | 39, 45, 54 |
| [`project_slice/region_slices/painted/tiles.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/tiles.rs#L21) | 21 | — | — |
| [`project_slice/region_slices/painted/top_bottom.rs`](../../crates/ares-core/src/project_slice/region_slices/painted/top_bottom.rs#L228) | — | — | 228 |
| [`project_slice/seam_candidates.rs`](../../crates/ares-core/src/project_slice/seam_candidates.rs#L301) | 301 | — | — |
| [`project_slice/seam_placement/alignment.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment.rs#L299) | — | — | 299 |
| [`project_slice/seam_placement/alignment/context.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/context.rs#L64) | 64 | 141 | — |
| [`project_slice/seam_placement/alignment/context/unsupported.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/context/unsupported.rs#L34) | 34 | — | — |
| [`project_slice/seam_placement/alignment/contour_continuation/geometry.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/contour_continuation/geometry.rs#L19) | 19, 26, 60, 61, 102 | — | — |
| [`project_slice/seam_placement/alignment/loop_normal.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/loop_normal.rs#L76) | 76, 116, 143, 152 | — | — |
| [`project_slice/seam_placement/alignment/paint_rules.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/paint_rules.rs#L115) | 115 | — | — |
| [`project_slice/seam_placement/alignment/string_fit.rs`](../../crates/ares-core/src/project_slice/seam_placement/alignment/string_fit.rs#L46) | — | — | 46 |
| [`project_slice/seam_placement/mesh.rs`](../../crates/ares-core/src/project_slice/seam_placement/mesh.rs#L34) | 34, 41 | — | — |
| [`project_slice/seam_placement/paint/contour.rs`](../../crates/ares-core/src/project_slice/seam_placement/paint/contour.rs#L48) | 48 | — | — |
| [`project_slice/seam_placement/paint/intervals.rs`](../../crates/ares-core/src/project_slice/seam_placement/paint/intervals.rs#L18) | 18, 137 | — | — |
| [`project_slice/seam_placement/placement.rs`](../../crates/ares-core/src/project_slice/seam_placement/placement.rs#L124) | 184 | 124, 128 | — |
| [`project_slice/seam_placement/runtime/nearest.rs`](../../crates/ares-core/src/project_slice/seam_placement/runtime/nearest.rs#L101) | 101, 102, 156, 256 | — | 180, 256 |
| [`project_slice/seam_placement/sampling.rs`](../../crates/ares-core/src/project_slice/seam_placement/sampling.rs#L54) | 54 | — | — |
| [`project_slice/seam_placement/spline.rs`](../../crates/ares-core/src/project_slice/seam_placement/spline.rs#L44) | 44, 57, 62 | — | — |
| [`project_slice/seam_placement/spline/qr.rs`](../../crates/ares-core/src/project_slice/seam_placement/spline/qr.rs#L215) | 215 | — | — |
| [`project_slice/seam_placement/split.rs`](../../crates/ares-core/src/project_slice/seam_placement/split.rs#L24) | 24 | 111, 132 | — |
| [`project_slice/seam_placement/visibility.rs`](../../crates/ares-core/src/project_slice/seam_placement/visibility.rs#L64) | 64, 94, 110 | — | — |
| [`project_slice/state/print_volume/circle.rs`](../../crates/ares-core/src/project_slice/state/print_volume/circle.rs#L54) | 54 | — | — |
| [`project_slice/support_material/extrusion_path/loop_split.rs`](../../crates/ares-core/src/project_slice/support_material/extrusion_path/loop_split.rs#L45) | 95, 99 | — | 45 |
| [`project_slice/support_material/interface_loops/centers.rs`](../../crates/ares-core/src/project_slice/support_material/interface_loops/centers.rs#L112) | 112 | — | — |
| [`project_slice/support_material/overhangs.rs`](../../crates/ares-core/src/project_slice/support_material/overhangs.rs#L274) | 274 | — | — |
| [`project_slice/support_material/project/region.rs`](../../crates/ares-core/src/project_slice/support_material/project/region.rs#L255) | 255 | — | — |
| [`project_slice/support_material/remove_bridges.rs`](../../crates/ares-core/src/project_slice/support_material/remove_bridges.rs#L114) | 114 | — | — |
| [`project_slice/support_material/tree/non_organic/contacts.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/contacts.rs#L65) | 172, 173 | — | 65, 157 |
| [`project_slice/support_material/tree/non_organic/drawing.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/drawing.rs#L83) | — | — | 83 |
| [`project_slice/support_material/tree/non_organic/drawing/contour_intersection.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/drawing/contour_intersection.rs#L76) | 76 | — | — |
| [`project_slice/support_material/tree/non_organic/drawing/node_shapes.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/drawing/node_shapes.rs#L62) | 62 | — | — |
| [`project_slice/support_material/tree/non_organic/dropping.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/dropping.rs#L186) | 186 | — | — |
| [`project_slice/support_material/tree/non_organic/grouping.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/grouping.rs#L90) | 90 | — | — |
| [`project_slice/support_material/tree/non_organic/movement.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/movement.rs#L30) | 30, 101, 104 | — | — |
| [`project_slice/support_material/tree/non_organic/sampling.rs`](../../crates/ares-core/src/project_slice/support_material/tree/non_organic/sampling.rs#L15) | 15, 63 | — | — |
| [`project_slice/support_material/tree/organic/branch_mesh/vector.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/branch_mesh/vector.rs#L11) | 11, 21 | — | — |
| [`project_slice/support_material/tree/organic/drawing/clipping.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/drawing/clipping.rs#L33) | — | — | 33 |
| [`project_slice/support_material/tree/organic/drawing/contacts.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/drawing/contacts.rs#L87) | — | — | 87, 89 |
| [`project_slice/support_material/tree/organic/initial_areas.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/initial_areas.rs#L70) | 70 | — | — |
| [`project_slice/support_material/tree/organic/pair_merge.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/pair_merge.rs#L109) | — | — | 109 |
| [`project_slice/support_material/tree/organic/roof_sampler.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/roof_sampler.rs#L95) | — | — | 95 |
| [`project_slice/support_material/tree/organic/single_area.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/single_area.rs#L55) | — | — | 55 |
| [`project_slice/support_material/tree/organic/single_area/radius.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/single_area/radius.rs#L141) | — | — | 141 |
| [`project_slice/support_material/tree/organic/smoothing/constraints.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/smoothing/constraints.rs#L75) | 75 | — | — |
| [`project_slice/support_material/tree/organic/smoothing/movement.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/smoothing/movement.rs#L25) | 25, 67 | — | — |
| [`project_slice/support_material/tree/organic/tip_sampling.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/tip_sampling.rs#L22) | 22, 84, 93 | — | — |
| [`project_slice/support_material/tree/organic/tip_sampling/next_point.rs`](../../crates/ares-core/src/project_slice/support_material/tree/organic/tip_sampling/next_point.rs#L28) | 28, 47 | — | — |
| [`project_slice/support_material/tree/overhangs/first_pass.rs`](../../crates/ares-core/src/project_slice/support_material/tree/overhangs/first_pass.rs#L126) | 126 | — | — |
| [`project/domain/plate_selection.rs`](../../crates/ares-core/src/project/domain/plate_selection.rs#L164) | 164 | — | — |
| [`project/domain/volume_offsets.rs`](../../crates/ares-core/src/project/domain/volume_offsets.rs#L18) | — | — | 18 |
| [`project/export_stl.rs`](../../crates/ares-core/src/project/export_stl.rs#L101) | 101 | — | — |
| [`project/export_stl/geometry.rs`](../../crates/ares-core/src/project/export_stl/geometry.rs#L193) | 193 | — | — |
| [`project/export_stl/repair.rs`](../../crates/ares-core/src/project/export_stl/repair.rs#L140) | 140, 152 | — | — |
| [`project/load/mesh_prepare.rs`](../../crates/ares-core/src/project/load/mesh_prepare.rs#L113) | 113 | — | — |
| [`skirts/mod.rs`](../../crates/ares-core/src/skirts/mod.rs#L313) | 313 | — | 313, 346 |
| [`stl/repair/normals.rs`](../../crates/ares-core/src/stl/repair/normals.rs#L108) | 108 | — | — |
| [`crates/ares-vgcode/src/sequential_view/actual_speed_profile.rs`](../../crates/ares-vgcode/src/sequential_view/actual_speed_profile.rs#L14) | 14 | — | — |
