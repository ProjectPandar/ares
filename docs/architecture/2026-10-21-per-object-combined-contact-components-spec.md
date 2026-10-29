# Draft SPEC: contact-connected brim components within per-object skirt groups

Status: **COORDINATOR-APPROVED SPEC**. Original discovery was external/read-only.
Authorized SPEC and RED branch: `worker2/combined-contact-components`.
Original frozen branch `worker2/brim-ears-efc-outline` remains unchanged at

e3eb8cbdb94460e81102b470788ec56b5fd26d5d

Explicit dependency: approved but not yet integrated ordered owner-area/shared
ShortestPath chain ending at that tip. Its original near-uncombined control now
matches complete output; this discovery is on that build, NOT current-main proof.
W2 waits behind W6→W5→W3→W4; no rebase/FULL during this discovery.

## Single upstream owning boundary

Checkout8500fcdccaa10b5099ac20d252af3a7c560046f1 under OrcaSlicer/src/libslic3r:

**Print.cpp:2898-2990, Print::_make_skirt::make_group_brims and installation into
SkirtBrimGroup**, with GCode.cpp:4490-4537::generate_object_brim consumption.
This is the source-owned grouped-brim collection rewrite, not an Ares-designed
pipeline or a whitelist-only admission feature.

- :2898-2917 filters group members to nonempty existing owner brim collections.
  With no combine or only one contributing owner, preserve original collection.
- :2919-2947 creates a union-find over those owners. For each i<j, contact is
  NONEMPTY intersection of offset_ex(area_i,coord_t(brim_flow.scaled_spacing*2),
  jtRound,SCALED_RESOLUTION) with area_j. Use assigned plate-coordinate owner areas,
  not bounding boxes, convex hulls, extrusion paths or skirt envelopes.
- :2949-2958 forms components in stable source/map-root order. A singleton keeps
  original owner paths verbatim. Merely sharing one skirt group does not combine
  brims: middle witness below is load-bearing.
- :2961-2970 unions a multi-owner component's areas, round offset2 cleanup using
  scaled print resolution and max(resolution,SCALED_EPSILON), then calls
  makeBrimInfillFromPlateCoordinates and retains all contributing owner IDs.
- :2972-2990 associates that component collection with the existing skirt group.
- GCode.cpp:4490-4514 emits a component once when an eligible owner is reached,
  plate origin0, then removes ALL component owner IDs from pending brim owners.
  :4520-4537 selects matching grouped collections for the current object.

Unchanged producer prerequisites: Brim.cpp:36-45,573-581 ordered assignment,
662-713 masks/retention,836-875 infill and884-886 plate-coordinate entry; previously
approved shared instance chain and owner subtraction must remain intact.

## Exact Rust destination boundary

Under crates/ares-core/src/project_slice/gcode_emit:

- skirt/generate.rs::SkirtPlan::generate already computes generation_groups with
  members. The new port belongs where those groups are known, mirroring the cited
  make_group_brims placement, not in an earlier global combine pass.
- skirt.rs::SkirtGroup/SkirtPlan is the current temporary skirt-only shell around
  upstream SkirtBrimGroup. Port grouped brim collections and contributing owner
  identities into that source-owned boundary; do not invent another Ares pipeline.
- brim/combined.rs supplies grouped contact-component construction using existing
  geometry and brim-path functions. Preserve current global combined behavior;
  a global BrimPlan::global carrier is NOT valid for per-object group components.
- layers/brim_emit.rs consumes component owners once, corresponding to
  GCode::generate_object_brim, without changing ordinary object order or motion.
- layers/object_skirt.rs continues existing once-per-group skirt emission. No
  new skirt hull/group-merging algorithm: skirt/member_groups.rs remains the
  producer of skirt memberships. Group membership and brim-contact connectivity
  are distinct upstream concepts and must not be collapsed.
- Minimal orchestration/signature wiring, if necessary to connect grouped brim
  preparation after skirt membership exists, must be reported for authorization
  before GREEN. Source-owned construction belongs in the above modules, not a
  growing independent orchestration layer. Keep occupied source owner areas for
  extents/grouping; do not replace them with merged emission paths/hulls.

Current gap: brim/combined.rs::enabled deliberately excludes active per-object
skirts; skirt/generate.rs rejects combine_brims when multiple objects have plans.
No component-owner collection is carried by current SkirtGroup. Removing that
error alone would emit independent owner brims and miss the Native near change.
Globally combining every skirt member would incorrectly merge middle/far.
No guard has been removed and no experimental production patch was made.

## Included / deferred behavior

Included witness envelope: ByLayer, one PLA/logical tool, two distinct objects,
one instance each, explicit OuterOnly brim width5/gap0, per-object2skirts,height2,
minimum length0, classic walls; support/raft/tower/shield inactive; no local speed
or EFC changes. Port stable contact-component grouping, singleton retention,
component union/cleanup/plate infill and once-owner consumption.

Deferred: new global combine behavior, ByObject/shared-group admission, repeated
instances, AutoWidth/material/thermal/EFC changes, active supports/raft/tower,
per-material flow differences, new print-object order, global bed masks, travel,
seam, ironing/fill, M73/time correction. Existing adjacent behavior remains
unchanged; no Unsupported guards/fallbacks/feature flags or new policy controls.
Core remains rendering/I/O-neutral and works on browser WASM/Windows/macOS/Linux.
Split touched source when needed to remain<=400LOC; no duplicated harness/KD.

## Fixed inputs: no position fishing

ALL9 inputs copied byte-for-byte from the previously paused discovery:
/home/indexyz/ares-coord/evidence/worker2-perobject-combined-discovery-20261021T103537/

No new placement, brim width or process setting was tried. Source tracked GUI
fixtures: tests/parity/per-object-auto-brim-admission/{near,far}-perobject-auto.3mf,
SHA respectively33879016885adcc85533e345a52a46d6ef4fbe74dd63156305f07bc1981d2d6a
and339a797d71afc111b8857aae4db0ef405eb6a1534f1404a9a6762395651d4390.
Generators retain meshes/model identities, set explicit width5, brimOuter/None,
combine0/1. Positions remain X80/88 near,80/94 middle,80/140 far,Y110. GUI path:
move object, choose brim type/width, enable combine checkbox, per-object skirt.

Current inputs/inputs.json stores all9 derived SHA256 and source provenance.
The three candidate RED target hashes are:
- near-combined:0ebfc787f6cbaa1b2cc170964e0a96d6b72cbb7a2e8e0052b8a35ce992b1a801
- middle-combined:c6ef1d191f8a70d63970483109e64fb3c303348cecb8fcf41892ab57c69758ad
- far-combined:6180b58d4a30c4682af41cce20191078f92570b21a3c7fe2718af30974698393

Six controls are the existing frozen ordered-brim-owner fixtures already in Git;
a prospective RED should reuse those paths, not duplicate them. All inputSHA values
are asserted during external verification; future public tests must freeze SHA.
Only input3MFs may enter fixture directories, never Native outputs or probes.

## Fresh Native-first verification on e3eb8cbdb

Existing CLI SHA256:
a4cf14095dfdc8598498461b84a2e42d0a3385606227b7dbf0d97bdfd40f962d

18 Native runs (9inputs×official/seeded) rc0;9 actualCLI runs. AllCPU13, serial,
seededORCA_REPLAY_CPU13/seed0. No builds, test runs or independent API harness runs.
Generator-line-only whole-output comparison, with M73 and all metadata retained.

| case | Native first-layer skirt/brim blocks | all positive Brim XY | all positive Skirt XY | CLI |
|---|---|---:|---:|---|
| near-uncombined |1/2|877|156|0, strict whole equal both modes|
| near-combined |1/1|924|156|1, existing per-object guard|
| near-no-brim |2/0|0|132|0, strict whole equal both modes|
| middle-uncombined |1/2|943|148|0, strict whole equal both modes|
| middle-combined |1/2|943|148|1, existing per-object guard|
| middle-no-brim |2/0|0|131|0, strict whole equal both modes|
| far-uncombined |2/2|942|264|0, strict whole equal both modes|
| far-combined |2/2|942|264|1, existing per-object guard|
| far-no-brim |2/0|0|131|0, strict whole equal both modes|

Native counts/order and positiveBrim-motion hashes are identical across modes.
Near combine changes Native actual Brim motion877→924 AND2→1blocks, same skirt.
Middle combine does NOT change actual Brim motion/hash943, despite ONE skirt group;
far likewise retains942/two skirt groups. This distinguishes contact components
from skirt membership and rules out unconditional group/global path union.
No-brim negatives verify admission is driven by actual brim plans, not the flag
alone. All3CLI failures explicitly UnsupportedProjectFeature("skirt_type per-object"),
not Native rejection, nondeterminism, empty Brim, environment failure or M73-only.
The former near-uncombined M73/geometry control failure is now strictly absent.

TYPE runs may collapse adjacent same-role object walls; do not infer missing
objects from near-combined's final Outer wall TYPE run. Positive motions are real
G1 with XY and E>0, not comments, travel or retraction.

## Proposed approval sequence and stop conditions

1. Coordinator approved this SPEC and public RED on a new branch based on the
   explicit ordered-owner/shared-chain dependency above; original branch stays frozen.
2. Commit this source-cited SPEC; freeze3combinedinput3MFs and
   reuse6existingcontrol3MFs; reference all9 with SHA, no unused inputs.
3. Public RED uses runner::native_stream_case::check UNCHANGED with
   assert_two_tools=false: Native double replay before public API + actualCLI,
   both complete ordered generator-only comparisons and rawAPI/CLI equality.
   No copied harness, ignore/allow/inverted result or geometry/time masks.
   Expected each mode3targetFAIL+6controlPASS. Native positive near change and
   middle singleton proof are externally evidenced before compare; not magical
   hard-coded result assertions. Report exact RED before requesting GREEN.
4. GREEN only approved group component/ownership boundary plus separatelyapproved
   minimal wiring. Inputs/tests/helper/assertions freeze. Any strict residual STOP;
   do not broaden into order/geometry routing/time repair or another Ares pipeline.
5. Directed9legs + existing brim/skirt/Auto/Ears/per-object and multi-instance
   order regressions both modes; fmt, strict workspace all-target clippy, wasm32.
   FULL only a future coordinator-allocated W2window, exact tip after rebase.

## Replayable artifacts and audit

This evidence root contains run.sh, retained generators,first-layer.js,verify.js,
inputs/manifest, all caseNative stdout/stderr/exit/result/G-code, actualCLI outputs
andguarderrors, first-layer-summary.json,verified.json,verification.txt,provenance.
Replay uses a NEW external root with those same frozeninputs and scripts; do not
regenerate/tune positions. Native wrappers explicitly select official2.4.2 or
seeded2.4.2; every slicing command tasksetCPU13.
Branch/source/test/helper/ref unchanged. No FULL, no builds or probe instrumentation.
Coordinator authorized SPEC then RED only. Report exact RED before any GREEN;
no production changes or FULL are authorized by this approval.
Evidence root: /home/indexyz/ares-coord/evidence/worker2-combined-contact-resume-20261021T155517
