# Ground a saved 3MF on a chosen face

Reference: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Upstream owners and Rust destination

- `libslic3r/PrintConfig.cpp:13002–13028` declares the three transforms. `libslic3r/Config.cpp:1844–1909` keeps the first occurrence of each key in the execution order and takes the final scalar value. `OrcaSlicer.cpp:5084–5141` dispatches in that order, before auto-orientation and arrangement.
- `libslic3r/LayOnFace.cpp:21–158` builds the object-space convex hull of transformed model parts, excluding modifiers and other non-parts. It joins adjacent facets whose unit normal components differ from the starting facet by less than 0.001. In each instance's frame (without translation), it discards single triangles with an edge shorter than 1 mm, outlines smaller than 5 mm², and polygons with an interior angle below 1°.
- `LayOnFace.cpp:160–214` selects the largest face, preferring the most downward instance normal among faces within 0.1% of the largest area; the object-space outward normal closest to the supplied direction; or the largest eligible outline containing the object-space point. Point selection uses a 0.01 mm tolerance in the instance's plane frame, for both plane distance and outline edges.
- `LayOnFace.cpp:216–242` rotates each selected instance to place the face normal down (inverse transpose for normals, an X half-turn for straight up). It preserves XY offset and translates that instance by its exact minimum Z, regardless of auto-drop (`Model.cpp:1778–1793`).

`ares-core` owns hull selection, the per-instance rotation and drop, and the failure if a point misses every eligible instance. The existing `CliProjectTransforms` chain carries face operations alongside the volume operations; the latter remain the adapter for the existing `ModelObject` transformations. `ares-cli` translates option values and presents invalid-parameter results. An instance with no eligible face stays unchanged and does not count as a point miss. A miss leaves that instance alone when another instance accepts the point. The normal and point each require exactly three finite comma-separated components; the normal must have length at least `1e-4` (`libslic3r/libslic3r.h:57`).

## Scope and verification

Saved project 3MF inputs only. Standalone STL and other raw input formats are deferred until their option-driven CLI input path is available. `--inspect-mesh`, `--inspect-paint` and `--info` (read-only actions in `OrcaSlicer.cpp:6335–6387`), GUI gizmos, and `--rotate-x`/`--rotate-y`/`--ensure-on-bed` are outside this change.

The public seam is slicing a saved 3MF with the CLI. Compare the entire G-code stream with live Orca output (only the generator label may differ), compare the result report, and repeat the native invocation in each oracle mode. Use unequal box dimensions so every selector changes the stream relative to slicing the same input without that selector. Include selector validation, point misses, and a transform-order witness.
