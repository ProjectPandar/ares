# Arachne almost-collinear vertex tolerance

## Upstream and Rust boundary

OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c` adds
`Arachne/utils/ExtrusionLine.hpp:34-40::colinear_vertex_tolerance`, returning
`coord_t(SCALED_EPSILON)`. The distance early-outs in
`Arachne/utils/ExtrusionLine.cpp:139-144::ExtrusionLine::simplify` and
`Arachne/WallToolPaths.cpp:176-179::simplify` consume that integer tolerance.
The old `8500fcdccaa10b5099ac20d252af3a7c560046f1` code uses scaled 0.005mm.

`libslic3r.h:57,65-66,80,105-109` owns EPSILON=1e-4, coordinate scale,
scale/unscale and SCALED_EPSILON. The beta helper casts epsilon to coord_t,
then compares squared integer height and double distance against that integer
value. It must not be replaced with an arbitrary numeric tolerance or rounding.

Rust destinations are the corresponding early-outs in
`arachne::extrusion_line` and `arachne::wall_toolpaths::outline`.
The existing `five_micron_tolerances` name and outline constant are temporary
compatibility shells for the old upstream policy, not an Ares simplifier design.
A future approved port must replace/rename that policy rather than preserve a
legacy fallback. Production implementation is not authorized by this RED contract.

## Included versus deferred

Included is only the cited Arachne distance early-out policy in both simplify
passes, with source integer cast, squared-height guard and distance threshold.
User wall-resolution/deviation and extrusion-area-error checks remain unchanged.
The separate `removeColinearEdges(0.005)` angular cleanup is not a 5um distance
and must not be globally replaced. Geometry kernels, scale/rounding contracts,
beading, Voronoi generation, chaining, emission, cooling/progress and configuration
admission remain outside the proposed slice.

Beta also removes old Clipper1 SimplifyPolygons calls in fixSelfIntersections.
The Rust outline's even-odd cleanup is a separate source question, not permission
to change it as compensation for a residual after the tolerance port. Any geometry
kernel question remains with the geometry owner; do not retune global epsilon.

## Public qualification

Reuse the unchanged existing ledger input
`validation/arachne-open-wall-order/nearest-control-neck0.25-r0.3mf`, SHA256
`ce5caae069fdf344382fc395f1ec85f576623bd33fbe483706a9b980705bcc41`.
No new input, geometry change, seed, threshold search or synthetic flag is needed.
The old complete-file ledger comparator remains unchanged and its reproduction
is not itself a qualified double-replay/validated-API RED.

The new public case qualifies two fresh official beta Native exports and calls
`ares_core::slice_project_plate_validated` directly once. It compares the whole
first-layer object using the existing shared extractor. All open/closed wall
motions, XYZ/E, widths, feed, M73, retract and wipe commands remain byte-visible.
There is no sorting, rotation, tolerance, normalization or motion omission.

The source contract gap is definite; its being the sole cause of every observed
geometry/E difference is not claimed before a controlled source port. If Native
is unstable or the public mismatch is confounded by a foreign boundary, keep the
candidate unqualified. After a qualified RED, implementation needs separate review.
A future GREEN must match the entire Native object; otherwise retain an open RED
and localize the residual without tuning other stages. No original complete-file
ledger row is closed by a focused boundary result.
