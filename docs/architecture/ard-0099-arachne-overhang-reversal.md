# ARD-0099: Consume source-owned Arachne steep-overhang flags

Upstream boundary: `PerimeterGenerator.cpp::detect_steep_overhang` (58–99),
`traverse_extrusions` (374–377,415–450,520–524), `reorient_perimeters`
(1117–1145), and its `process_arachne` call (2464–2477).

Rust destination: `project_slice/perimeters/arachne/{traverse,materialize}.rs`
and a small `steep_overhang.rs`. Detection uses the ordinary clipped paths
before overhang paths are appended, grouped by exact flow width. Thresholds
resolve against each group's width, then test open polylines against offset
raw lower slices, not the nozzle-grown clipping polygons. Preserve odd-layer,
raft, contour/hole and internal-only decisions.

Reuse the existing source-owned `reorient_perimeters` leaf function in
`classic/perimeter_append/reorient.rs`; do not dispatch through the Classic
pipeline or reinterpret Arachne variable-width walls as Classic walls. That
namespace and shared extrusion vocabulary remain temporary transport shells
around the common upstream static function, not a legacy fallback.

Included: clipped ordinary-path steep detection and threshold special case,
ordinary unsplit flags, Arachne flag consumption, shared loop-only reversal,
public unchanged-stock Prusa and sensitive option full-output controls. Remove
the obsolete overhang_reverse rejection only with the complete consumer;
remove its obsolete rejection assertion and dead-field suppression.
Actual-overhang public RED requires the adjacent `clip_extrusion` owner
(`PerimeterGenerator.cpp:280–357`): port its width ZFill callback into
geometry/clipper/{z,engine}.rs and its zero-width subject projection repair
into z_clip.rs. Enable that callback only for clip_open_path_widths; keep
contour-index collection and marked support ZFill behavior unchanged.
Truncate interpolated width changes exactly where upstream casts coord_t;
do not repair sentinel values by guessing a constant width. Existing
width-rejection scaffolding is removed only with this source-owned consumer.
Included now: genuine crossing width interpolation and zero-width projection
repair. The crossing tracer also reaches the non-loop dispatch boundary:
GCode.cpp::extrude_entity (6087–6100) routes MultiPath to extrude_multi_path,
not extrude_loop/place_seam (5773). Destination seam_placement/placement.rs
must leave non-loop paths unsplit for their existing emitter, preserving
candidate bookkeeping and all extrusion output. Remove the obsolete
MultiPath unreachable scaffold; do not invent seams for open walls.
GCode.cpp::extrude_multi_path (6038–6085) owns the subsequent emission:
rewrite its ordered path dispatch and joined/reversed wipe storage in
gcode_emit/motion/perimeter/multi_path.rs, and include MultiPath geometry
in the existing wipe-before-external touching-lines view. Existing flat path
emission is the temporary shared `_extrude` transport shell; never coerce
an open MultiPath to a Loop or discard its paths. Adaptive PA weighted-flow
state in this upstream function remains deferred with the existing adaptive
PA owner, not claimed by these non-adaptive public fixtures.
Open-wall start selection (`PerimeterGenerator.cpp:472–497`) depends on
PointHash map traversal (`Point.hpp:496–500`), not insertion order. Reuse the
existing selected-reference map ordering rewrite from TreeSupport's
position_map, moving it to geometry/point_hash_map.rs with a generic payload
for its two real consumers. Keep bucket growth and linked iteration unchanged
for Tree; add mutable payload lookup for endpoint occurrence accumulation.
This remains the fixed Linux-reference ordering contract used by existing
Tree parity, producing the same canonical order on all Rust platforms;
it does not claim every C++ standard-library implementation uses that order.
Deferred, not waived: active Arachne fuzzy-skin generation,
alternate-extra-wall and adjacent matrix residuals.
The remaining seam dependency includes `GCode/SeamPlacer.cpp:485` and
`LayerRegion.cpp:21–28`: candidate flow width belongs to the actual layer
region's nominal external Flow, including first-layer width selection, not
the first emitted variable-width path. In alignment/preparation.rs pair each
island's perimeter collection with its existing perimeter_regions identity,
then read that region/layer's PerimeterInputRecord.ext_perimeter_flow. Preserve
collection/source ordering and use the same contract for Classic and Arachne;
never substitute region zero or a neighboring record. Candidate association
and scoring remain unchanged until the complete-output tracer proves a need.
Do not change the complete comparator, fixture geometry or motion assertions
to obtain parity. Existing Classic behavior must remain unchanged.
