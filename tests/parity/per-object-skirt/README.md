# Per-object skirt RED fixtures

These are input 3MFs, not Native output artifacts. They derive from the
retained `runner/multi_object_skirt.rs` combined fixture whose SHA is in
`manifest.json`. Inputs contain two or three 10mm-square, 4mm-high prisms
(and a repeated-instance case), two skirt loops, 6mm skirt distance,
perobject mode, skirt height1, single filament, by-layer printing, and
no brim/support/raft/tower/draft/minimum-filament extension.

`manifest.json` pins each exact input SHA and build object IDs/x coordinates
(y110, z2; component z scale0.4). Expected Native skirt groups: disjoint2,
overlap1, transitive1, copies1. Copies at x80/140 share object2; object3
at x110 lies inside their occupied convex hull. Transitive has x80/99/118:
adjacent expanded envelopes intersect while the endpoints alone do not.

Native-first CPU18 official/seeded preflight produced exit0 on all four
inputs, with positive XY skirt extrusions and distinct separated hulls
in disjoint. Evidence is external:
`${TMP_ROOT}/ares-skirt-native-witness-9BH3dU/{official,seeded}/<case>/`
and `witness.json` there. No Native output is committed.

The public test reruns Native before API/CLI, retains inputs, launch metadata,
witness, raw streams/errors and strict comparisons in external artifacts.
The expected RED is Ares rejecting `skirt_type per-object`; the final
assertions require successful API/CLI and unchanged strict Native parity,
not the rejection itself. The same tests will be used unchanged for GREEN.
