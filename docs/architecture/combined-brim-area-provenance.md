# Combined brim provenance barrier (before implementation)

RED `3c160a64bb7261babc0af6b495be294685654ad0`, parent stack
`4351a644d6683555ae48e44ea7f898a66f75d585`. No production modification at
barrier inspection. Runtime evidence `${TMP_ROOT}/ares-global-combined-brim-red-3c160a64b`.

## Concrete producer/consumer trace

Input target is exactly the paired control except combine_brims1→0. 3MF
objects2/3 each reference cube10.stl_1.model, with instance translations
(80,110,2)/(140,110,2), Z scale0.4, one instance per object. Both contribute
outer_only width8/gap0. Print resolution0.012, skirt_height1, skirt_loops2,
draft_shield disabled, combined skirt, ByLayer. Global source gate is true.
Native accepts both, emitting two spatial brim islands but a SINGLE target
TYPE:Brim block versus TWO control blocks. Target first brim motion
X127.841 Y103.322 is at the right object's outer area; the second island's
first brim motion at line1161 is X92.353 Y105.195. Thus global mode is not
merely hiding the second per-object brim: it orders both islands through a
single collection before model extrusion (first model line2124).

`Print.cpp:2520-2530` builds objPrintVec from the NULL-start-near
`chain_print_object_instances(print)` (:2517), de-duplicating owners and
filtering objectExtruderMap membership. Do NOT substitute schedule's
wipe-tower-seeded REVERSED per-tool order (`layers/instances.rs::collect`).
`ShortestPath.cpp:2015-2042` uses the existing greedy endpoint chain with
nullptr; :190-224 has no distinguished seed in this case; :320-327 chooses
first/last from the residual queue. Existing Rust object_order supports only
a distinguished seed, so its shared private implementation needs the source
None path for carrier selection; existing seeded calls must stay unchanged.

`Brim.cpp:424-689` generates per-object areas, clips no-brim and bed masks,
retains contact, storing object brim area maps BEFORE extrusion path generation.
`make_brim:945-956` copies footprint maps with plate shift before merge. Their
owner identity and spatial occupied hulls must remain per-object. Current Rust
`brim.rs::generate` has those clipped/contact-retained local ExPolygons but
immediately consumes them to produce separate paths; orchestration maps this
once per object. No current combine consumer. `bed_area::clip` returns local
coordinates after global clipping. Source merge therefore requires integer
instance translation into a common plate frame BEFORE union/cleanup, not
merging already-local paths or offsetting output decimal strings.

`Brim.cpp:979-998`: append nonempty object areas, union_ex, then offset2_ex
(+delta,-delta,jtRound,scaled_resolution). Here lattice4096/mm, config resolution
0.012 => scaled_resolution49.152 (double), delta=max(float49.152,float0.4096).
Neither delta is the source infill simplification constant: tryExPolygonOffset
:813-834 uses 0.0125/SCALING_FACTOR (=51.2 double) and flow.scaled_spacing()
for stepped offsets. No arbitrary coordinate-snap/epsilon addition permitted.
The two offset2 stages must use the existing source offset2 primitive (raw
first-stage paths to second stage), not two unrelated boolean-normalized passes.
`makeBrimInfill:836-886` is called ONCE after cleanup, then applies plate origin.

`Brim.cpp:1000-1018` selects first contributing object in objPrintVec, with
source fallback to first contributor when no member is found. Only this
carrier owns the merged extrusion collection. Noncarrier footprint maps stay
intact; do not transfer the merged hull to a singleton owner's skirt footprint.
`GCode.cpp:4512-4517` global-single-map branch emits on FIRST eligible call,
regardless of caller identity; :4491-4508 once-only bookkeeping, origin0.
Thus selecting a carrier is not permission to delay extrusion until that
carrier's model entry. Current `layers/brim_emit.rs` selects only current
object and has no global once-only state; this is the precise consumer seam.
Keep original path/covered bounds APIs for normal objects, empty noncarrier
emission paths only under the source global gate, and retain per-object hulls.

## Evidentiary limit and implementation decision

This is an input/runtime-output plus pinned SOURCE provenance trace, not a
claim to have captured Native's private ExPolygon vertices or ObjectID values
in a debugger. The 0.001 first difference alone does not establish which
intermediate vertex changed. What is established is a missing entire owning
source branch, its actual input/gate, area lifetime/frame, cleanup constants,
NULL-start carrier derivation, global-single-map emission and two-island
runtime behavior. Implement that exact sequence at the existing private brim
seam, keeping the control path byte-stable, then use full public comparison
as the deciding check. If comparison still diverges, STOP and retain artifacts;
do not iteratively snap coordinates or widen this slice.
