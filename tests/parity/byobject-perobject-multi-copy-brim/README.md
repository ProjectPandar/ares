# ByObject perobject-skirt + multi-copy owner brim: public RED

Source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`Print.cpp::_make_skirt:2646,2794-2920`, `Brim.cpp:914-977`,
`GCode.cpp:4465-4531,5334-5335`.

Frozen input-only 3MFs: two owners, three placements X50/125/200,Y110,Z2,
scale0.4, ByObject, outer_only brim8/gap0, combine0, skirt2/height1,
one filament; no support/raft/tower. `manifest.json` records complete SHA/member
hashes. Combined control changes ONLY skirt_type; width0 changes ONLY brim_width.
No Native output is committed.

All cases run Native FIRST and require successful Native output, positive model
extrusion at all three centers, two sequential origin transitions, real positive
skirt blocks (target/width0:2, combined:1). Active-brim cases require two owner
brim blocks and positive brim extrusion at all three centers; width0 requires0.
The test then requires validated API and actual CLI to slice successfully,
raw API==CLI, and COMPLETE ordered Native equality through unchanged
`golden::compare_ordered_bytes_generator_only` (generator line only).

Before the port, target genuinely FAILS because API
returns `UnsupportedProjectFeature("skirt_type per-object")`, CLI1/no G-code.
Both controls must PASS whole-output comparison. Tests never assert rejection
as success. Source count guard is untouched in this RED commit.

Production scope is only retirement of the
ByObject multi-instance disjunct and its orphaned comment/binding. Other kind,
combine, tower/flush and ByObject shared-group guards remain. No pipeline,
geometry, cursor, M73, estimator, parser, comparator or normalization changes.
