# Belt printing with multiple placements

Source: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Source citations are relative to `src/libslic3r`; Rust paths are relative to
`crates/ares-core/src`.

## Decision

A belt plate may contain multiple printable instances of one or more model
objects. Each printable placement is a separate `PrintObject` even when the
mesh, orientation and scale are identical to another placement. Port this
in order: by-layer placements, sequential placement transitions, and per-owner
brim. Each uses the existing object slicing and G-code pipeline. Do not share
sliced layers across belt placements, concatenate complete one-object G-code
files, or order whole objects by their belt coordinate in by-layer mode.

Across these slices, preserve the selected plate's single logical filament and
slot constraints, one `ModelPart` volume per model object, and existing support,
raft and region restrictions. General multi-volume and multi-filament printing,
belt purge, supports, vase mode, and interactive apply/cache invalidation are
outside these slices. Cartesian grouping and output remain unchanged.

## By-layer placements

In `project_slice/belt.rs::admit`, lift the one-object and one-instance limits
for by-layer printing. Check volume, support, raft and region restrictions for
every participating object, not only the first. Ignore non-printable instances
as in `PrintApply.cpp:193–230`. Retain the existing by-object rejection. When
there is more than one printable placement, reject an active belt brim with
`UnsupportedProjectFeature("belt_printer: multi_placement_brim")` until
per-owner brim is available. This is an Ares capability boundary, not a Native
validation result. Existing single-placement brim and skirt admission are
unchanged.

### Print-object placement and slice frame

- **Instance separation.** `PrintApply.cpp:193–230,1952–1983`
  (`print_objects_from_model_object`, `Print::apply`) takes XY out of the
  compensated transform and uses a one-based printable ordinal to perturb its
  existing Z translation by `1e-10` before inserting it into the ordered
  transform set. It does not replace bed-drop Z. Carry the instance identity
  explicitly through `project/effective_config/{grouping,phases}.rs` and resolved
  print objects; preserve upstream transform ordering and the geometry-side Z
  perturbation without relying on the perturbed value as the sole identity.
  `Print.cpp:3359–3413` disables sharing slices between equivalent belt objects.
  The flat transform groups remain the non-belt path.
- **Height and lift.** `PrintObject.cpp:141–179` centers the base transform and
  adds the center to the instance's output shift. `PrintObject::update_slicing_parameters`
  (`PrintObject.cpp:4463–4499`, `BeltTransform.cpp:89–193`) computes slicing height
  and an initial floor from the centered model-part bounds, retaining bed-drop
  Z. `PrintObjectSlice.cpp:199–206,899–921,1738–1743` and
  `BeltSliceStrategy.cpp:12–79` use the centered transform and model-part
  vertices for the exact rotated mesh and floor-based lift. `trafo_sliced()`
  reproduces this local transform, not the instance placement or the motion
  transform. Geometry consumers that need globally shifted print Z add the
  object's offset separately (e.g. the adaptive infill octree at
  `PrintObject.cpp:1191–1197`). In `project_slice/{planning,belt,layers,raw_intersections}.rs`,
  prepare the belt slice, parameters/profile and layer plan for each resolved
  placement; replace the current first-group and constant-ordinal assumptions.
- **Placement after intersection.** `PrintObjectSlice.cpp:961–1042` computes
  `offset_z = min(min_z, 0) + (T * center).z + ((T - I) * placement).z`,
  where `T` is the forward belt rotation, the center is the XY offset removed
  before slicing, and `placement` is the instance shift minus that center.
  Apply a significant offset to `Layer::print_z` and to the floor shift, not
  to slice planes. Keep the three frames distinct: centered slice geometry,
  placement-adjusted layer Z, and remapped machine XYZ. Belt build-volume height
  validation is separate: `Print::validate` compares model Z (with shrinkage),
  not tilted or global layer Z (`Print.cpp:2365–2422`). Use
  `project_slice/print_height.rs` for this distinction.
- **One machine transform.** `BeltGCode::init_belt_writer` and `on_set_origin`
  (`BeltGCode.cpp:13–19,34–48`) install one configured kinematics chain and
  rotate the selected instance's XY origin at each instance visit. The writer
  uses the same inverse slice rotation, axis remap and machine shear/scale for
  all objects (`GCode/BeltBackTransform.cpp:8–32`, `GCode/BeltKinematics.cpp:10–27`,
  `GCode/MachineFrameTransform.cpp:11–88`). There is no per-instance writer
  transform or extra machine-axis translation. Reuse `belt/kinematics.rs` and
  `gcode_emit/motion/{origin,belt_writer}.rs`; update the origin, owner and floor
  at each visit. `GCode.cpp:11615–11640` takes the floor from the active print
  object (or the explicit brim owner), including on a layer shared by objects.

### Layer schedule and visits

- **By layer.** `GCode::collect_layers_to_print` (`GCode.cpp:2314–2490,2492–2551`)
  handles lead-in and empty-layer diagnostics per `PrintObject`, removes entries
  without any extrusion and sorts the remaining entries on adjusted print Z.
  It merges heights within `EPSILON` of the lowest height in a group, uses the
  midpoint of the minimum and maximum heights as group Z, and retains the
  per-object entries. A gap *between* objects is not an internal object gap.
  Keep owner IDs in `gcode_emit/layers/{schedule,group}.rs`; share the generic
  instance and island visitation paths in `layers/{instances,island_visits}.rs`.
  `GCode.cpp:3680–3689,7264–7374` and `ShortestPath.cpp:2533–2561` use ordinary
  print order and per-filament island tours on the participating entries (or
  model order for `AsObjectList`), not a belt-position tie-break.
  `GCode.cpp:7665–7672` selects the next copy's origin and requests the external
  motion planner on object/copy changes; do not reset the writer on each visit.

### Public witnesses

Run single-volume, single-filament synthetic projects through the public
project slicing API/CLI in both reference modes. Compare result and G-code
using the existing parity comparators, with placement and order assertions
based on emitted layer markers, object labels and machine-coordinate body
motion. Displace same-mesh copies along Y, across X at equal Y, and across X
with a noncoincident Y stagger. Include distinct meshes and reversed model
order; preserve the single-placement, internal-empty-gap and non-belt controls.

In both Native modes at 45° X tilt, copies displaced 80 mm on bed Y begin at
sliced Z `46.1477` and `102.716`. Reversing model order does not change which
placement prints first. Their first walls share machine height and are offset
by 80 mm along the machine belt axis, as in standalone controls. Copies at the
same Y and different X both print on 42 shared layer heights. A 3 mm Y stagger
interleaves two noncoincident layer sequences. The narrower, taller block and
cube keep their distinct layer counts and standalone positions.

## Sequential placement transitions

After by-layer placement parity, admit by-object sequencing with ordinary
clearance and per-instance transition behavior. Multi-placement brim remains
outside this slice. `Print.cpp:2144–2160,2238–2265` does not ban the sequence on a
belt. The normal clearance path checks instances and assigns `arrange_order`
using model-object order (`Print.cpp:895–1005,1115–1124,1154–1209`); the
position-scored sort above it is disabled. `GCode.cpp:3313–3345,3614–3618`
consumes that order. Separate it from transform and layer order, and do not
infer the order of copies from an unstable tie between sort keys. Reuse
`print/validate/sequential_clearance.rs` and the per-instance phases in
`gcode_emit/layers.rs` and `layers/drive.inc`. Each phase consumes only that
placement's retained layers, even when the next phase goes to a lower global
print Z. Follow `GCode.cpp:4663–4716` for retraction, transition at the maximum
layer Z plus hop, origin travel before the descent, start temperatures, cooling
reset and belt first-layer-band reset. Keep the selected sequence's layer
counts consistent with its retained passes.

### Public witnesses

Compare accepted sequential slices of distinct meshes and two copies in both
Native modes. Check ordered visit completion, boundary travel and restart,
and a lower-print-Z second phase without a plate-wide Z re-sort. In the probe,
the cube at larger Y prints first in model order while the lower-Y block goes
first in by-layer mode. A touching by-object pair fails the normal conflict
check and emits no G-code. Assert the public validation result for that case.

## Per-owner belt brim

After sequential transitions, lift the multi-placement brim restriction for
both sequences. Include coincident bands, apron-only entries, and clipping
against other model placements. Keep supports and regular skirt outside this
slice. Native skips regular skirt generation (`Print.cpp:3892–3904,4030–4036`),
so there is no per-object skirt to reproduce here.

`BeltBrim.cpp:403–442,561–599` owns per-placement coincident bands and apron-only
entries. At each band it considers other objects' layer contours near that Z,
translated by the relative instance shift into the owner's local frame, before
clipping. Generate and retain plans per owner in `project_slice/belt/brim/`;
replace the current single-plan lookup and first-entry assumption in
`gcode_emit/layers/{schedule/belt_brim,belt_brim}.rs` and `layers/drive.inc`. The
apron can share a print Z with a different object's apron or model layer. Use
the special brim-only layer branch only when there is no object/support layer
in that group. Emit a band's paths once for its owner, including when that owner
has no object extrusion on the layer, using that owner's origin and floor
(`GCode.cpp:6019–6055,6253–6288,6620–6625,7452–7464,7586–7618,8134–8162`).

### Public witnesses

Compare brims around each placement in separated and side-by-side synthetic
projects. Include a nearby object whose contours clip a band, two aprons at the
same print Z, an apron sharing another object's model layer, and a sequential
plate so each apron stays in its own phase. With X-separated copies staggered
by four layer steps, both Native modes emit four apron bands from the later
placement on the same layer markers as the earlier placement's model extrusion.
Requesting ordinary skirt loops does not add Native skirt extrusion. Do not
store Native G-code in the repository.
