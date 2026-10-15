# Frozen input-only regional ZAA Line direction fixture

Approved SPEC bb1d0fe5cc2fbbaf9b285454f7f83b41ce16bad3.
Source provenance:the admitted FULL14 wide-slab LockedZag shift input
(height2.4mm,40%,relative E,wall_loops0,infill direction45).
Preserve mesh/profile members; set sparse pattern line,top/bottom layers and
thickness0,empty rotation templates. This avoids internal solid ceilings
and the separately deferred Line bridge-anchoring context.

The three cases differ only in the two ZAA switches; shared min_z0.05 and
minimize_perimeter_height35 remain unchanged. ZIP metadata fixed at
2026-10-15T00:00:00. These are input3MFs only; no Native G-code is committed.

| Input | ZAA | dont_alternate | SHA256 |
| --- | --- | --- | --- |
| flag-false.3mf |1|0|e2f8c2b3b86bf2b4f760363a9e7384758ea86c7e6831b0c44d5c5cc289d4a7eb|
| flag-true.3mf |1|1|05cd4c39abf5a2c8deee9b6ade8880d8b855014217d9f41bc129bc309a5f03b3|
| disabled-zaa-flag-true.3mf |0|1|2be6d2499d7318a66d57c32ba0ef427e1478647b815c8927030b73310ec85393|

Fill.cpp:1235-1242 sets the runtime regional flag. FillBase.cpp:105-118
passes _infill_direction (:311-313) to FillLine::_fill_surface_single
(FillLine.cpp:10-118), which inherits the normal layer-angle rule.

Before RED, official+seeded each accepted6 fresh references:3 cases x2
repeats. No Internal Bridge output. Both controls passed unchanged strict
whole-output API+CLI comparison. Native flag1 has different undirected
Sparse geometry on layer2/object layer1, while Ares flag1 equals flag0.
The public assertion compares exact layer2 Sparse start/end XY+relative E
before unchanged strict whole-output assertions; repeats remain live.

The first differing motion is its Sparse entry travel; the old TYPE Skirt
persists until after travel (GCode.cpp:6378/6788-6790). Earlier M73 placement
is retained by the comparator; GCodeProcessor.cpp:1232-1257 derives it from
elapsed/global total time. No motion,progress or timing lines are filtered.
