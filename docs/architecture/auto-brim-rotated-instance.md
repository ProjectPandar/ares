# Automatic brim height for a rotated build instance

## Problem and intended behavior

A saved project containing one model instance can rotate a model so that its
transformed height is exactly 10 mm while its contact footprint remains narrow.
Automatic brim must use that transformed height. Because the upstream narrow-
footprint rule is a strict `> 10 mm` test, the exact-10-mm case must not gain
the additional 5 mm brim width. A transformed height above 10 mm must retain
that additional-width behavior.

This slice ports the automatic-brim volume-group height measurement from
OrcaSlicer 2.5.0 at upstream revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Upstream contract

- `src/libslic3r/Brim.cpp:301-319`,
  `configBrimWidthByVolumeGroups`, iterates model-part volumes in a group,
  composes the owning object's first instance matrix with each volume matrix,
  and merges each transformed mesh box. The resulting Z size is `groupHeight`.
  This is an unpadded, unclipped mesh extent, not a local-mesh height, a
  first-layer height, or a solid-volume measurement.
- `src/libslic3r/TriangleMesh.cpp:456-461`,
  `TriangleMesh::transformed_bounding_box(const Transform3d&)`, evaluates every
  stored vertex in double precision and merges the transformed points. The
  overload used by the brim calculation does not clip at the bed or inflate
  the result.
- `src/libslic3r/Brim.cpp:320-345` uses this height with first-layer contour
  moments, contour bounds, thermal reference, adhesion, and maximum speed to
  compute the raw width. `:526-552` quantizes that width to pairs of flow
  spacings.
- `src/libslic3r/Brim.cpp:562-576` adds the quantized 5 mm width only when
  `groupHeight > 10.`, the width is below 5 mm and one flow spacing, and the
  polygon area-to-contour-length ratio is below 1.1 mm. The per-polygon cap
  and final quantization still apply.

## Ares destination and defect

`crates/ares-core/src/project_slice/gcode_emit/brim/groups.rs:252-289`,
`model_mesh_bbox`, currently obtains the owning object and first instance,
composes the instance and volume transforms, and then calls the shared region
bounds helper. `crates/ares-core/src/project/region_graph/bounds.rs:3,42-63`
uses `f32` transformed points and inflates Z by `1e-4` mm at both ends. Thus a
stored mesh spanning `[0, 10]` mm can reach approximately `10.00020027` mm in
this path even with no XY offset.

`crates/ares-core/src/project_slice/gcode_emit/brim/auto.rs:12-77`
uses the mesh-box Z size for the upstream width formula, while
`:139-183::polygon_width` already implements the correct strict `group_height
> 10.0` comparison. The comparator and the shared region-graph helper are not
the defect. The defect is supplying the automatic-brim path with padded,
reduced-precision bounds.

## Implementation boundary

Add an automatic-brim-specific group-height measurement over stored mesh
vertices. Reuse the existing project/volume mapping and the composed owning
instance and volume matrices, but evaluate points in double precision and
merge the Z extrema of all selected model-part volumes before taking their
difference. Pass this one unpadded height to both the width calculation and
`polygon_width` so all automatic-brim width consumers use the same value.

For this slice, a printable group uses the owning model object's first
instance, matching the current single-instance ownership boundary. Do not
change instance admission or selection, shared-object ownership, group
assignment, elephant-foot outline selection, material or speed lookup, width
quantization, brim geometry/emission, or saved 3MF archive writing. Multi-
instance selection and shared-instance policy remain deferred.

Do not change `region_graph::bounds::transformed_mesh_bbox`, add a comparison
tolerance, subtract a constant from its result, or otherwise alter bounds used
by overlap and slicing logic.

## Verification contract

Use an asymmetric, manifold 32 x 10 x 2 mm beveled prism with one normal
volume, one printable build item, stock X1C PLA settings, and automatic brim.
Store a quarter-turn-about-X build matrix with translation onto the bed so the
contact footprint is 32 x 2 mm and the transformed Z extent is exactly 10 mm.
Slice the saved project directly through the plain CLI route; do not re-export
or arrange it first.

Run the same fixture in both existing comparison lanes against pinned Native.
Assert actual positive-extrusion brim moves and their absence, rather than
relying only on feature comments. Require ordered G-code equality after the
established producer-banner normalization for matching cases. Include these
controls:

- the same rotated model with brim disabled;
- an unrotated automatic-brim model;
- rotated 9 mm and 11 mm height witnesses.

The exact-10-mm, disabled, unrotated, and 9 mm cases must not print the extra
brim; the 11 mm witness must still print it. Existing automatic-brim, brim,
and saved-project comparisons remain unchanged.

The paired CLI comparison at the clean base revision completed successfully
in both lanes. Both lanes reproduced the same result: the exact-10-mm rotated
case emitted no Native brim moves but emitted Ares brim moves, while the
brim-disabled, unrotated, 9 mm, and 11 mm controls matched. The two producer
outputs in the divergent case differ by the expected brim block and resulting
progress/timing lines; no comparator masking was used. The two default config
values that differed in the initial synthetic run were set explicitly in this
paired fixture and no longer account for the result.
