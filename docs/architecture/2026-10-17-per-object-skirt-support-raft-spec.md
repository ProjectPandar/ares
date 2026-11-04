# Per-object skirt support and raft admission SPEC

## Source boundary

This slice ports the support/raft part of OrcaSlicer’s per-object skirt
admission. The owner is `OrcaSlicer/src/libslic3r/Print.cpp::Print::_make_skirt`
(`2646-2990`). Its occupied-point collection includes object and support
footprints at `2669-2696`; the per-object membership and fixed-point envelope
merge are `2784-2905` (with the object/instance grouping details at
`2794-2803`), and loop construction is `2718-2778`. The final outermost-first
loop order is established at `2985`. `OrcaSlicer/src/libslic3r/GCode.cpp::generate_skirt`
(`4388-4451`) owns skirt-layer selection; `find_skirt_brim_group_idx` is at
`4455-4463` and `generate_object_skirt_group` at `4465-4485`. The seam start
calculation is the separate `Skirt::find_start_point` at `4334-4359`.

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
  `GCode.cpp:4465-4485`; `project_slice/gcode_emit/layers/tool_entry.rs:92-112`
  and `object_skirt.rs:28` currently limit object-skirt emission to entries with
  an `object_layer` and `physical_layers()[layer]`.

## Included behavior

On the existing per-object, multi-object path, remove only these two Ares
admission disjuncts after a public RED:

1. `objects.iter().any(|object| object.object.enable_support.0 ||
   object.object.raft_layers.0 > 0)` in `skirt/generate.rs`; and
2. the top-level `raft.is_some()` disjunct in the same guard.

The existing `occupied.rs:61-80` support-fill and
`RaftSchedule::skirt_points` inputs must then participate in each object’s local
hull before instance shifts, membership merging, loop generation, and emission.
The separate raft carrier is a compatibility shell: upstream supplies raft
support layers through `Print.cpp:2687-2691`, rather than a separate geometric
policy. The same GREEN must replace the `entry.object_layer`-only emission gate
in `layers/tool_entry.rs:92-112` and `object_skirt.rs:28`, so per-object skirt
paths are emitted on raft/support-only chunks with the upstream first-layer and
skirt-height timing. Preserve the current
fixed-point envelope grouping, object/instance translations, layer limit,
outermost-first loop order, flow, seam clipping, and per-group emission. No new
support or raft geometry policy is introduced; this is the missing admission
boundary around already-present footprint inputs.

## Deferred boundaries

The wipe-tower/multi-filament disjuncts in this guard are a separate slice:
`enable_prime_tower`, non-`ByLayer` print sequence, and more than one filament
diameter. They remain unchanged. Brim eligibility (`combine_brims`, brim type,
instance count), tower-corner input, draft-shield behavior, sequential printing,
and any raft geometry beyond the existing `skirt_points` carrier are also
deferred. This slice must not alter `occupied::items` semantics, grouping
algorithms, or any guard belonging to another slice.

The GREEN must also port the upstream emission coverage. `GCode.cpp:4398`
derives the first-layer decision from raft layers, `:4311-4318` lets raft layers
contribute to skirt height, `:4455-4463` selects the skirt/brim group, and
`:4465-4485` emits an object-skirt group. `:5329-5335` calls that path for every
`instances_to_print` entry, including raft/support-only chunks. Ares must not
require `entry.object_layer` or index only `physical_layers()[layer]` for those
chunks.

## Native RED plan

Freeze three input-only 3MFs with identical machine and
skirt settings and two objects in `PerObject` mode. Every input must explicitly
set `brim_type=no_brim`, `combine_brims=0` (`PrintConfig.cpp:1744`), one
filament, `ByLayer`, no prime tower, and one instance per object so the result
is attributable only to this slice:

1. a support-positive case (`enable_support=true` on one object, with ordinary
   support geometry) that exercises the support footprint;
2. a raft-positive case (`raft_layers>0`) that exercises the raft skirt points
   and raft/support-only emission chunks; this setting intentionally trips both
   admission disjuncts;
3. a control with support disabled and `raft_layers=0`, while all tower/filament and other
   disjuncts are false.

Add an extra, separately reported control: one object with per-object skirt and
`raft_layers>0`. Ares currently routes this through the combined path
(`gcode_emit/drive.rs:150`); compare it with Native, but if it fails, report a
separate single-object finding rather than folding it into this two-object
guard/emission result.

Each case must be accepted by Native. The public test will use
`OrcaRunner::from_env()` and compare complete API and CLI output to Native with
the unchanged strict comparator; the control must pass and each positive case
must fail at the current unsupported guard. Run official and seeded with fresh
roots/cache/target and the canonical environment. The test
must record the first divergence and classify it as admission, omitted
support/raft footprint, group membership, instance shift, or unrelated output;
it must not commit Native artifacts or weaken assertions. If Native rejects,
crashes, or produces no support/raft footprint for an input, stop and report
that result instead of inventing a RED.

## Failure inventory and exit

The candidate failure inventory is: Ares rejects a Native-accepted positive
case; support or raft points are absent from the local hull; fixed-point group
membership differs; instance shifts differ; per-group loop ranges or seam
emission differ; per-object skirt is missing or mistimed on a raft/support-only
layer (including the first-layer decision at `GCode.cpp:4398` or skirt-height
accounting at `4311-4318`); API and CLI diverge; or the control changes. A RED is valid
only when the control is strict-equal and the positive Native witness is active.
Production work is limited to the two guard disjuncts above plus that
raft/support-layer emission change (`tool_entry.rs:92-112`,
`object_skirt.rs:28`) and follows a separate GREEN.
