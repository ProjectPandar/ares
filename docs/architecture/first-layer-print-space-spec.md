# First-layer print bounds in print space

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/GCode.cpp:3894-3908`, inside `GCode::_do_export`: outside the
  pressure-advance pattern calibration, every point of `print.first_layer_convex_hull()`
  goes through `print.translate_to_print_space(pt)` before it is stored in
  `first_layer_print_convex_hull`, and `first_layer_print_min`, `first_layer_print_max`
  and `first_layer_print_size` come from the bounding box of those translated points.
- `src/libslic3r/Print.cpp:4543-4552`, `Print::translate_to_print_space`: subtracts the
  plate origin `m_origin` and the IMEX slice offset (zero unless IMEX firmware-managed
  zones are on, `Print.cpp:4522-4536`).
- `src/libslic3r/Print.cpp:4644-4654`, `Print::finalize_first_layer_convex_hull`, with
  `first_layer_islands` (`:4437`) and `first_layer_wipe_tower_corners` (`:4462`, which
  adds `m_origin` at `:4491`): the hull is built in the world frame, where every plate
  but the first sits one plate stride away.
- `src/libslic3r/Print.hpp:1294`, `set_plate_origin`, called from
  `PartPlate::set_print` (`src/slic3r/GUI/PartPlate.cpp:4411`) with the origin of
  `PartPlateList::compute_origin` (`:6018-6025`).
- `src/libslic3r/GCode.cpp:3945-3950`: `adaptive_bed_mesh_min` and
  `adaptive_bed_mesh_max` clamp the same translated bounding box.

The head-wrap check at `GCode.cpp:3909-3935` keeps using the untranslated hull, with the
detect zone shifted by the writer offset, so the hull itself stays in the world frame.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/footprint/first_layer.rs`,
  `first_layer_bounds`: its inputs (first-layer islands with the instance shift, skirt,
  brim and tower corners) are in the world frame like upstream's hull. It subtracts
  `traversal.project.plate_origin()` from the bounds it returns, as
  `translate_to_print_space` does, so `placeholders/bounds.rs` renders
  `first_layer_print_min`, `first_layer_print_max`, `first_layer_print_size` and the
  adaptive bed mesh from print-space values.
- `crates/ares-core/src/project_slice/gcode_emit/motion/writer_state.rs` renders
  `first_layer_print_convex_hull` from `EmitState::first_layer_hull` with the same
  subtraction. `first_layer_hull` itself stays in the world frame for the head-wrap check.

The existing bounds code is a temporary compatibility shell around the hull; this slice
only moves its output into the frame upstream renders.

## Observable behavior

On a project whose only object is on plate 2 of an X2D bed, the machine start template's
`G29 A1 X{first_layer_print_min[0]} Y{first_layer_print_min[1]} I{first_layer_print_size[0]} J{first_layer_print_size[1]} R`
and its `G29 A2` line render an `X` that is one plate stride (1.2 bed widths) larger than
Native's. `Y`, `I` and `J` match because plate 2 sits beside plate 1 along X. Plate 1
projects are unaffected: their origin is zero.

## Failure modes

- Translating only the bounds: `first_layer_print_convex_hull` keeps world coordinates
  on later plates.
- Translating the hull stored in `EmitState`: the head-wrap check loses its frame.
- Subtracting the origin per input instead of once: tower corners or brim are shifted
  twice or not at all.
- Translating the adaptive bed mesh a second time after the bounds were translated.

## Test plan

A Native comparison test derives a project at runtime from
`tests/parity/validation/print-bed-two-plates.3mf`, setting the object's
`sparse_infill_filament_id` to 1 so the plate uses one filament and neither side prints a
wipe tower (the tower decision depends on the post-region normalization in other work).
It slices plate 2 with Native and with Ares in both modes and compares the executable
block from `; EXECUTABLE_BLOCK_START` up to Native's `; MACHINE_START_GCODE_END`,
without the `M73` progress lines, whose remaining-time values differ for unrelated
reasons. Premises, read from the input at runtime: the build item's X lies beyond the
printable area's width, so the object is placed on a later plate in the world frame, and
the machine start template references `first_layer_print_min`. The test stores no Native
coordinates.

## Deferred

- `; MACHINE_START_GCODE_END`, which Native prints after the start template and Ares
  does not; it is the first difference after this region.
- The `M73` remaining-time estimates in the start region.
- `first_layer_center_no_wipe_tower` (`GCode.cpp:3968-3975`), which upstream renders
  from untranslated object boxes.
- The IMEX slice offset in `translate_to_print_space`: zero for this printer.
- Wipe tower presence on the two-filament plate (post-region normalization) and the
  configuration keys that differ in the configuration block.
