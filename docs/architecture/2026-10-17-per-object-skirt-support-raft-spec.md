# Per-object skirt support and raft admission SPEC

## Source boundary

This slice ports the support/raft part of OrcaSlicer’s per-object skirt
admission. The owner is `OrcaSlicer/src/libslic3r/Print.cpp::Print::_make_skirt`
(`2646-2990`). Its occupied-point collection includes object and support
footprints at `2669-2696`; the per-object membership and fixed-point envelope
merge are `2784-2905` (with the object/instance grouping details at
`2794-2803`), and loop construction is `2718-2778`. The final outermost-first
loop order is established at `2985`. `OrcaSlicer/src/libslic3r/GCode.cpp::generate_skirt`
(`4388-4451`) selects a member group’s loop range at `4417-4424` and emits the
seam-clipped loops using the skirt path boundary at `4334-4359`.

The current Rust rewrite already has the relevant upstream-shaped boundaries:

- `project_slice/gcode_emit/skirt/generate.rs:24-58` owns per-object admission;
- `skirt/occupied.rs::items` collects object contours, support fills, raft skirt
  points, instance shifts, and local convex hulls (the existing
  `Print.cpp:2669-2696,2794-2803` citation);
- `skirt/member_groups.rs::merge` owns the fixed-point member envelopes
  (`Print.cpp:2784-2900`);
- `skirt/loop_generation.rs::make_loops` owns the per-group loop budget
  (`Print.cpp:2728-2778`); and
- `SkirtPlan::emit_group`/`emit` select and emit the group range corresponding to
  `GCode.cpp:4417-4424,4334-4359`.

## Included behavior

On the existing per-object, multi-object path, remove only these two Ares
admission disjuncts after a public RED:

1. `objects.iter().any(|object| object.object.enable_support.0 ||
   object.object.raft_layers.0 > 0)` in `skirt/generate.rs`; and
2. the top-level `raft.is_some()` disjunct in the same guard.

The existing `occupied::items` support-fills and `RaftSchedule::skirt_points`
inputs must then participate in each object’s local hull before instance shifts,
membership merging, loop generation, and emission. Preserve the current
fixed-point envelope grouping, object/instance translations, layer limit,
outermost-first loop order, flow, seam clipping, and per-group emission. No new
support or raft geometry policy is introduced; this is the missing admission
boundary around already-present footprint inputs.

## Deferred and ownership boundaries

W5 owns the wipe-tower/multi-filament disjuncts in this guard:
`enable_prime_tower`, non-`ByLayer` print sequence, and more than one filament
diameter. They remain unchanged. Brim eligibility (`combine_brims`, brim type,
instance count), tower-corner input, draft-shield behavior, sequential printing,
and any raft geometry beyond the existing `skirt_points` carrier are also
deferred. This slice must not alter `occupied::items` semantics, grouping
algorithms, or any guard owned by another worker.

## Native RED plan

After SPEC approval, freeze three input-only 3MFs with identical machine and
skirt settings and two objects in `PerObject` mode:

1. a support-positive case (`enable_support=true` on one object, with ordinary
   support geometry) that exercises the support footprint;
2. a raft-positive case (`raft_layers>0`) that exercises the raft skirt points;
3. a control with support disabled and `raft_layers=0`, while all W5 and other
   disjuncts are false.

Each case must be accepted by Native. The public test will use
`OrcaRunner::from_env()` and compare complete API and CLI output to Native with
the unchanged strict comparator; the control must pass and each positive case
must fail at the current unsupported guard. Run official on CPU13 and seeded on
CPU14, with fresh roots/cache/target and the canonical environment. The test
must record the first divergence and classify it as admission, omitted
support/raft footprint, group membership, instance shift, or unrelated output;
it must not commit Native artifacts or weaken assertions. If Native rejects,
crashes, or produces no support/raft footprint for an input, stop and report
that result instead of inventing a RED.

## Failure inventory and exit

The candidate failure inventory is: Ares rejects a Native-accepted positive
case; support or raft points are absent from the local hull; fixed-point group
membership differs; instance shifts differ; per-group loop ranges or seam
emission differ; API and CLI diverge; or the control changes. A RED is valid
only when the control is strict-equal and the positive Native witness is active.
Production work is limited to the two guard disjuncts above and follows a
separate approved GREEN. No RED, production edit, or broad gate starts before
SPEC approval.
