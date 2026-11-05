# Region-graph volume extents keep the volume offset

Rewrite slice of `trafo_for_bbox` (`PrintApply.cpp:622-630`) as used by `update_volume_bboxes` (`:972-1031`) for `generate_print_object_regions` (`:1043-1260`), at OrcaSlicer v2.5.0-beta `345efec64`. The volume extents that decide whether a modifier creates a print region are taken in the object frame with only the object's bed placement removed. The volume's own XY offset inside the object stays.

## Problem
`tests/parity/modifier-sparse-loss/A-C2.3mf` is a 20 mm cube part with an `adaptivecubic` 60 % / 0.60 mm sparse-infill modifier placed outside it. The part's matrix translates it to X 118, the modifier's to X 145.5.
- **Native:** the modifier's extents do not touch the part's, so it creates no region. The print has one region and one extrusion-width block in the header.
- **Ares:** removes the XY translation from the whole object-times-volume product, so every volume is centred on the object origin. The modifier then overlaps the part and adds a second region. The header gains a second width block with `; infill extrusion width = 0.60mm`; nothing else differs.

## Upstream boundary
- **`trafo_for_bbox`** (`PrintApply.cpp:622-630`) zeroes the X and Y translation of the object transform, then multiplies by the volume matrix: `m = object_trafo_local * volume_trafo`.
- **`update_volume_bboxes`** (`:972-1031`) builds every `layer_range.volumes` extent from it: `transformed_its_bbox2d` for one layer range (`:994`), `transformed_its_bboxes_in_z_ranges` for several (`:1026`).
- **`generate_print_object_regions`** (`:1156-1198`) reads those extents through `find_volume_extents` and `find_modifier_volume_extents`. A modifier gets a region only for a parent whose extent `intersects` its own and whose config it changes (`:1183-1188`).
- **The header** writes one width block per print region (`GCode.cpp:4321-4334`), so the region count is visible in the output.

## Ares destination
`project/region_graph.rs` `object_region_graph`, which builds the extents consumed by `build_region_graph` and `add_modifier`:
- **Port `trafo_for_bbox`:** take the first print object's transform without its XY translation, then multiply by the volume transform. The current code removes the XY translation after the multiplication.
- **Precision stays as upstream.** `trafo_for_bbox` composes the product in double and returns it as `Transform3f` (`.cast<float>()`), and `transformed_its_bbox2d` / `transformed_its_bboxes_in_z_ranges` transform the float vertices with it. `bounds.rs` already matches: the product is composed in `f64`, and `Transform3d::transform_point_f32` casts each coefficient and vertex to `f32` before the float multiply-add.
- Nothing else changes. `add_modifier`'s intersection test and `register_region` already follow `:1156-1198`.

## Included and deferred
- **Included:** `PrintApply.cpp:622-630` for the region-graph extents (`:994`, `:1026` → `:1156-1198`).
- **Deferred:**
  - **`project_slice/volume_bounds.rs`.** It composes the slice-stage volume bounds the same way. Those bounds feed the slice-stage overlap tests and other consumers, and changing them in the same way changes the output of 10 multi-part brim and skirt cases away from Native. They are a separate slice.
  - **The `verify_update_print_object_regions` re-check** (`:815-969`, called at `:2132`). It only runs when an existing print is re-applied, which the byte-in slicing API never does.

## Oracle and RED
The existing whole-output test `runner/modifier_sparse_loss.rs` `adaptive_outside_modifier_preserves_complete_native_sparse_output` compares A-C2 with Native. It fails on main with the second width block as the only difference and passes with this port. Its controls `A-C0` (no modifier) and `A-C1` (a modifier that changes nothing) already pass and must stay passing. No new test is needed.
