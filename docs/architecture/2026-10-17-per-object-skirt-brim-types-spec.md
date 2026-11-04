# Per-object skirt InnerOnly and Auto brim admission SPEC

## Premise and bounded witnesses

The premise baseline is `4351a644d6683555ae48e44ea7f898a66f75d585`.
The fixture is one through-hole mesh duplicated into two separated objects at the
same transforms for every case. It sets `skirt_type=perobject`, two skirt loops,
`combine_brims=0`, one instance per object, one filament, ByLayer printing, no
prime tower, support and raft disabled. Only the brim type differs between the
OuterOnly control and the candidate.

The fresh Native InnerOnly run was accepted with rc 0. Its G-code contains two `;TYPE:Brim`
blocks and two `;TYPE:Skirt` blocks, so the per-object hull consumes nonempty
brim footprint. The same-geometry OuterOnly control was accepted with rc 0 and also has two Brim and two Skirt blocks.
Ares was run on these exact inputs with the existing CLI. OuterOnly completed rc
0; InnerOnly returned rc 1 with
`UnsupportedProjectFeature("skirt_type per-object")`. This is an empirical
admission witness, not a source-only inference.

Auto was also probed on the same two-object geometry (and on narrow/tall and
support-enabled variants) with Native rc 0, but Native emitted no `;TYPE:Brim`
block, and Ares accepted the corresponding input. Auto
therefore has no nonempty-brim witness and is excluded from this slice pending a
fixture that activates Brim.cpp's auto-width path. It is not claimed as RED.

## Upstream boundary

`OrcaSlicer/src/libslic3r/Print.cpp::_make_skirt` (`2780-2977`) builds the
per-object occupied hulls. Its object footprint and fixed-point grouping, which
include each object's computed brim footprint, are `2847-2971`.
`Brim.cpp:448-460` constructs the InnerOnly hole ring; shared support/no-brim
mask handling is `590-634`. Auto and painted-ear paths are separate at
`Brim.cpp:964-977` and `354-412`/`456-458` respectively.

The Rust admission boundary is `gcode_emit/skirt/generate.rs:48-56` on that baseline:
`plan.is_some()` then combines `combine_brims`, the brim-type match, and
`group.instance_indices.len() != 1`. The remaining second disjunct is only the
active tower/flush condition (`tower_active` plus
`flush_into_objects/infill/support`), which is a separate slice. There is no remaining
print-sequence or prime-tower/multi-filament branch to change here. Existing
`skirt/occupied.rs:61-80` consumes `BrimPlan::covered_hull()` after support/raft
points and before shifts/group merging.

## Included and deferred behavior

After a public RED, remove only the brim-type exclusion for the Native-valid
InnerOnly case. Preserve `combine_brims`, the instance-count condition, and the
entire tower/flush disjunct. Preserve hull construction, fixed-point grouping,
instance shifts, and emission ordering. Auto remains deferred until a nonempty
Native auto-width brim witness exists. Painted and BrimEars remain deferred
pending their own Native witnesses.

## Public RED plan

The public RED will use frozen input-only InnerOnly and same-geometry OuterOnly
fixtures. Each independent test invokes Native through `OrcaRunner::from_env`,
then compares complete API and CLI output with the unchanged strict comparator;
the control must pass and InnerOnly must fail at the Ares guard. The test also
records Brim and Skirt block presence to prevent an admission-only fixture.
Official and seeded modes use fresh roots. No Native artifacts are
committed and no assertions are weakened. Native rejection/crash, missing brim
footprint, changed skirt grouping or shifts, API/CLI divergence, and control
changes remain failure classes.
