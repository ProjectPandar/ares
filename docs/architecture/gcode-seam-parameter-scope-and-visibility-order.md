# SeamPlacer: parameter scope, layer height and visibility operation order

A0/A share one Native RED and joint GREEN validation in
[the current-object report](2026-10-08-seam-current-object-mode-validation.md).
B/C remain separate slices. Execute in order **D → A0 → A → B → C**;
each implemented boundary requires committed whole-output Native RED and GREEN,
with the A0/A attribution and split rule below.

## Upstream boundary and common decisions

Pinned OrcaSlicer revision:
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Upstream paths below are relative to `OrcaSlicer/src/libslic3r/`;
Rust paths are relative to `crates/ares-core/src/project_slice/`.
These are ports of existing `GCode/SeamPlacer.cpp/.hpp` behavior into the
existing seam compatibility shell, not new Ares seam policies.

| Slice | Owning upstream boundary | Rust destination |
|---|---|---|
| D | `SeamPlacer::calculate_overhangs_and_layer_embedding`, `.cpp:1084–1091`; threshold `.hpp:121–122`; actual height/slice-z `PrintObjectSlice.cpp:24–40,50–80` | `seam_placement/alignment/context.rs::populate, populate_overhang`; height threading through `alignment/preparation.rs` and existing traversal records |
| A0 | `GCode.cpp:2533–2534,5330–5343,5771–5773`, export defaults followed by current-object configuration before perimeter seam placement | `gcode_emit/motion/options/build/fields/assemble.rs`, `gcode_emit/layers/object_entry.rs::emit` |
| A | `SeamPlacer::init`, `.cpp:1431–1441`; `raycast_visibility`, `:130–132,165–167` | `seam_placement.rs::apply`, per-object construction of `GlobalVisibility` |
| B | `process_perimeter_polygon`, `.cpp:456–471`; `LayerRegion.cpp:21–28`; `PrintRegion.cpp:6–22,25–53` | `seam_placement/alignment/preparation.rs::layer_regions, prepare_layer`; `seam_candidates.rs::RegionPerimeters, generate_regions, process_polygon` |
| C | `raycast_visibility`, `.cpp:159–185`, AlignedBack adjustment and per-ray decrement | `seam_placement/visibility.rs::from_mesh, sample_visibility` |

Non-negotiable: preserve candidate order, strict/weak comparison rules,
visibility sample count30000, ray grid5×5, RNG, mesh frame, angle weights and
the already merged distance Gaussian. Do not add a coordinate special case,
epsilon/tie policy, fallback, ignored test, unsupported-feature guard, probe
or relaxed comparator. Core remains platform-neutral for WASM, Windows,
macOS and Linux. Keep any new test helper narrowly scoped; files exceeding
400 LOC must be split without unrelated cleanup.

## D — current layer height in overhang penalty

### Source discrepancy and precise port

Upstream `.cpp:1085–1091` computes overhang from signed previous-layer
distance, `.65f * perimeter.flow_width`, and
`tan(SeamPlacer::overhang_angle_threshold) * current_layer->height`, then
clamps negative overhang to0. The threshold is45 degrees (`.hpp:122`).
`PrintObjectSlice.cpp:24–40` places regular slice-z at the layer midpoint;
this is distinct from the layer's actual thickness.

Rust `runtime.rs:51–63` correctly constructs midpoint slice-z, but
`alignment/context.rs:18–24` derives height from adjacent midpoint z values.
For consecutive thicknesses h_prev/h_current that difference is their
average, not h_current. `:40` subtracts that derived value directly.

Included: thread the authoritative existing traversal/layer thickness into
the overhang calculation, retaining midpoint candidate z; reproduce the
upstream threshold coefficient, evaluation types/rounding and negative
clamp. Determine the actual C++ tan overload and intermediates before
transcribing it; do not infer that a mathematical tan45 substitution is
automatically float-identical. Do not widen/narrow the entire scoring chain
as a convenience. Do not change signed-distance geometry merely to hide a
failure elsewhere.

Deferred: layer planning, adaptive/profile support, ZAA, signed-distance
algorithms, embedding policy, unsupported-distance data not needed for this
penalty, and spline/nearest comparator behavior.

### Reachability and RED

Current `capabilities.rs:12–21` rejects painted layer-height profiles and
ranges carrying layer-height overrides; `planning.rs:50–51` uses the fixed
profile planner. An adaptive profile rejected by Ares is **not** a seam RED.
First try a supported first-layer height different from normal layer height
(e.g. .3 then .2), with an unpainted stepped/sloping overhang whose candidate
ordering can change on the second layer. Confirm both producers actually
use those thicknesses and the same seam mode. This reaches unequal adjacent
heights without adding adaptive support. Existing `ClassicTraversalRecord`
already exposes actual layer_height (`runtime.rs:59`).

Use a small finite set of geometric slopes/overhang depths and supported
height pairs, recording every attempted case and result. Prefer nearest or
short aligned strings to isolate overhang ranking from spline fitting.
The complete Native bytes must differ before edits, with retained first
diff and layer/seam context. A changed internal overhang scalar alone is
not RED. GREEN must restore whole-file equality, including timing/wipe
consequences. Equal-height input is a sibling control.

## A0 — emit seam_position belongs to the current PrintObject

### Source discrepancy and precise port

Upstream `GCode.cpp:2533–2534` initializes the export configuration from
default object/region settings. Inside the current-instance loop
(`:5330–5332`), `:5341–5342` applies default region configuration and then
`m_config.apply(instance_to_print.print_object.config(), true)` before
assigning the current layer (`:5343`). Perimeter emission subsequently
calls `m_seam_placer.place_seam(m_layer, ...)` at `:5771–5773`. Object
configuration is refreshed for every object entry, not inherited from the
first object in the project.

Rust `gcode_emit/motion/options/build/context.rs:39` selects
`resolved.objects.first()`. `fields/assemble.rs:326–328` uses that object's
seam_position to initialize the shared MotionOptions. The current-object
entry in `gcode_emit/layers/object_entry.rs::emit` does not refresh it.
Consequently `motion/perimeter.rs:97–118` can skip Nearest selection for a
later Nearest object when the first object is AlignedBack. Object-indexed
nearest penalties already reach this entry through `layers/entry_builder.rs`;
that handoff does not correct the stale motion mode.

Included: initialize seam_position from the actual global default
`full.process.object.seam_position`, removing this field's dependency on
the first resolved object. Immediately before emitting each object's layer,
refresh `state.options.seam_position` from
`traversal.resolved.objects[object_index].object.seam_position`. Use the
existing authoritative object index; add no optional lookup or fallback.
Check option replacement by `motion/tool_selection.rs:16` so every object
entry refresh occurs after its tool selection and before perimeter emission.
The existing emit compatibility shell temporarily carries the one upstream
object setting needed by this slice; no new Ares configuration policy is
introduced.

Deferred: other MotionOptions fields currently derived from the first
object/region, generalized object configuration application, parsing and
resolution, tool scheduling, support/skirt/brim behavior, and the separate
visibility bias boundary A below. This slice removes the
first-element substitution for seam_position only.

### RED, attribution and split gate

A0+A share one RED. Its three E2E tests retain
the complete mixed-object fixture, both-Nearest control and single-AlignedBack
control. Fresh Native comparison gave two PASS and one DIVERGENT. For the
mixed fixture, the first normalized difference is byte2322, raw line105:
Native `G1 X120.31 Y106.31 E.22677` versus Ares `M73 P60 R0`.
The later Nearest object's first travel at raw line120 is Native
`G1 X90.418 Y107.695 Z.6` versus Ares `G1 X93.69 Y119.69 Z.6`.
The M73 timing difference remains part of the whole-file RED and GREEN.

An isolated, uncommitted A visibility-scope trial did not change Ares's
complete output SHA (`9e14003642f98b6003b5d385ca606ddacb934b4a2b592b6cec12d2bd678c64ef`)
and did not fix this RED. That trial was reverted. It establishes masking
in this fixture, not that visibility scope is correct or unreachable.

Implement A0 alone first and rerun the
unchanged three whole-file tests with fresh Native output. If A0 alone
restores complete equality, commit and validate A0 as its own GREEN; do
not include A's visibility edit. Then seek an independent reachable Native
RED for A on the corrected A0 baseline. If A0 leaves a residual mismatch,
retain both the original RED and the A0-only output and determine whether
the residual is attributable to A before implementing that boundary in a
separate commit. Never weaken the comparator or attribute every remaining
mixed-object difference to visibility without evidence.

## A — visibility bias belongs to each PrintObject

### Source discrepancy and precise port

Upstream loops over PrintObjects, reads each object's seam_position, then
passes that setting into occlusion computation (`.cpp:1431–1441`). Only
that object's AlignedBack mode adds front_adjustment (`:165–167`).
Rust `seam_placement.rs:59–61` computes any(AlignedBack) over all placements
and passes the same boolean into every object's visibility model (`:65–78`).
This can bias an unrelated Nearest/Aligned object.

Included: derive bias from the current placement while constructing that
object's existing visibility model. Preserve object/visibility indexing and
the existing front-adjustment arithmetic. No shared cross-object bias flag.
Deferred: C's operation-order correction, visibility sampling/raycasts,
transforms, seam overrides parsing, object ordering and all Gaussian logic.

### Reachability and RED

Construct two distinct objects with stored object-level seam_position
metadata: one aligned_back, one nearest. Use existing 3MF object-metadata
construction patterns (`ares-cli/tests/orca_parity/runner/multi_object_skirt.rs`)
and verify effective object options through the public project/config
boundary before asserting output. `ProcessSeamPosition` wire values are
defined at `options/process_options/object_source/enums.rs:67–78`.

The nearest object must have competing candidates with different front
exposure; adding a plain cube with an unchanged winning corner is not proof.
Retain the original two-object whole-file mismatch and identify which
object's path changes. Use sibling configurations with both objects nearest
and with only the aligned_back object to separate global bias leakage from
object labels/ordering and from C. Do not suppress object-id comment
differences; classify an unstable Native object-id oracle separately.
GREEN must match the complete mixed-object Native output. After A0 is
correct, follow its split gate above: a fixture already made PASS by A0
cannot independently justify A's production edit.

## B — angle arm comes from the region's external flow nozzle

### Source discrepancy and precise port

Upstream `.cpp:463–471` uses
`region->flow(frExternalPerimeter).nozzle_diameter()` for polygon angle
sampling; only a nullptr region gets0.5. `LayerRegion.cpp:21–28` delegates
to region flow; `PrintRegion.cpp:12–13,50–53` selects the nozzle for that
region's outer-wall role, not the first global nozzle unconditionally.

Rust `seam_placement.rs:82–101` threads a single global first-nozzle value
through all regions. `alignment/preparation.rs:113–124` already has the
region's ext_perimeter_flow, but copies only its width. That flow contains
nozzle_diameter (`perimeters/types.rs:7–11`). `seam_candidates.rs:36–50`
then supplies one angle_arm_mm for every polygon.

Included: carry the authoritative region external-flow nozzle through the
existing RegionPerimeters/polygon association to angle sampling. Preserve
external flow width and source ordering. Remove the obsolete global-nozzle
threading where this change makes it unused. Preserve upstream null-region
semantics only where a real corresponding sentinel polygon path exists;
do not globally replace0.4 with0.5 as a shortcut.
Deferred: flow generation, filament/nozzle mapping, tool scheduling, Arachne
geometry, fallback flows and candidate-angle algorithm changes.

### Reachability and RED

Start from the existing two-physical-nozzle Snapmaker J1 reference setup
(`ares-cli/tests/orca_parity/runner/compatible_regions/inherited_filament.rs`),
assign different supported diameters (e.g. .4/.6), and make a non-first
region's external wall use nozzle2. Verify Native's effective nozzle vector,
outer-wall role assignment and Ares's public resolved configuration; same
diameter multi-filament output does not exercise this discrepancy.

Use a concave/short-edge contour sensitive to angle arm length. First prove
both producers can slice the intended unequal-nozzle configuration without
unrelated tool/flow differences. Capture whole-file RED and candidate-angle
path context; a private parameter assertion or unknown Native mapping is
insufficient. A same-diameter sibling input is the control. If a separate
existing flow/mapping defect blocks the witness, report that boundary and
skip this slice rather than absorb that implementation into B.

## C — AlignedBack addition precedes occlusion decrements

### Source discrepancy and precise port

Upstream `.cpp:159–167` starts result at1, adds front_adjustment, then
subtracts1/25 for each qualifying ray hit (`:184–185`). Rust
`visibility.rs:32–35,109–116` completes the decrements first, then adds the
adjustment. The real-number formula is equivalent; the f32 sequence differs.

Included, **only after real RED**: move the existing bias contribution to
the upstream point in that sample's accumulation. Preserve the bias formula,
f32 decrement, ray order/hit predicate, normals, sample generation and
neighbor interpolation. Deferred: exp/log/PI policy, SIMD/FMA decisions,
mesh decimation, negative-volume ray behavior and alignment score grouping.

### Reachability, RED or UNREACHABLE

Use one aligned_back object so A cannot be the cause. Choose supported
concave/occluded surfaces with nontrivial front-adjustment and ray hits;
vary a documented finite set of tilt angles/notch depths/orientations.
Prefer fewer than6 layers initially to avoid an alignment string hiding the
candidate change. Retain exact geometry/config/producer hashes for each
attempt. F32 scalar examples only explain risk; they do not establish RED.

If no complete Native-visible mismatch attributable to C is found, record
the searched input domain, attempts, whole-file verdicts and any masking
conditions as **UNREACHABLE in the tested supported domain**, then skip C
without production edits. This bounded result is not a proof over all
possible meshes. Apply the same no-RED/no-edit disposition to D/A/B if
their witness cannot be constructed.

### C1 — signed-distance precision chain audit (2026-10-08)

C is expanded to audit the `_dist` double chain after D. This is
separate from C's visibility accumulation order and needs its own Native
RED attribution before any distance implementation changes.

Upstream `Line.hpp:251–267`: despite its name, `Linef` stores Vec2d endpoints
and its Scalar is double. `ExPolygon.hpp:175–193` constructs these lines by
unscaling integer contours into Vec2d. `SeamPlacer.cpp:1083–1085` takes the
candidate's already-float XY position, then promotes it to double. Candidate
quantization is upstream behavior; do not replace candidate positions with
double as part of this slice.

`AABBTreeLines.hpp:300–304,329–352` uses double Floating for Linef, obtains
squared segment distance, takes sqrt in double, applies the outside sign,
and returns double. `Line.hpp:43–69` computes edge, squared length,
projection, endpoint branches and squared residual in double. The interior
residual is `(t * v - va).squaredNorm()`; changing only the final return type
would leave earlier precision loss and would not port this source boundary.

Ares `alignment/context.rs:51–95,118–130` currently narrows unscaled contour
endpoints to f32, computes segment projection/residual in f32 (including an
explicit mul_add and hypot), reduces with f32::min, then returns f32.
`populate_overhang:44–47` promotes that rounded result back to f64.
`populate_embedded_distance:103–104` adds the width contribution directly in
f32. Upstream `.cpp:1086–1097` instead keeps signed distance double through
the addition, and stores each final overhang/embedded value as float. The
`0.65f * flow_width` term remains f32 before double promotion; actual layer
height remains double and tan(float) retains the D policy.

The audit also identifies two adjacent boundaries, not automatic fixes:
Ares determines inside from a rescaled integer candidate whereas upstream
outside() receives the promoted float candidate in double. Also upstream
`.cpp:1092,1563` stores/returns unsupported_dist separately; the current
Ares LayerPlan has overhangs and embedded_distances only. Neither sign
classification nor unsupported-distance propagation may be silently added
to an arithmetic-only GREEN; report an independently visible discrepancy.

Conditional destination: existing `alignment/context.rs` distance helpers
and final assignment sites. Preserve candidate f32 storage, region/contour
association, source endpoint ordering and D's actual layer height. Port
segment arithmetic/rounding from Line.hpp rather than globally replacing
f32 types or inventing a generic geometry API. No production edit without
an attributable complete Native RED for this added boundary.

RED preparation: use nearest-mode sheared/rotated single-object Dumbbell
siblings to exclude AlignedBack accumulation bias, preserve first/regular
heights0.3/0.2, vary a finite documented slope/orientation set, and keep an
unsheared control. For original C visibility use one aligned_back object,
uniform heights and concave neck/orientation variants. Retain all byte
verdicts and producer/input/output hashes. A shared failing control or an
unattributed seam difference is not either sub-slice's RED. If neither
arithmetic issue is observable, retain the bounded UNREACHABLE disposition
above; do not edit both formulas together in search of a passing result.

## Per-slice TDD and artifacts

1. Recheck the current slice on top of the earlier merged slices. Never
   reuse another slice's RED, except the shared A0+A witness under its
   attribution gate. Keep the small-loop/scarf region untouched.
2. Build/retain exact 3MF witnesses with CLI-side fixture construction only;
   compare the public byte slicing API against freshly generated identified
   Native. Only `; generated by` may be normalized. Record input, output,
   Native binary/resources/wrapper and Ares executable hashes, first byte
   and raw line difference, command receipts and process deadlines.
3. Commit RED separately before production edits. No
   unsupported input, Native timeout, UB or nondeterministic oracle result
   may be called the slice's successful RED. First explain unexpected
   producer changes; do not select a favorable retry or weaken assertions.
4. Implement only that source boundary. Require complete-byte
   GREEN for the same input. If timing, wipe or other residual differences
   remain, report them; do not turn a seam-only assertion into GREEN.
5. For each implemented slice, run fmt, strict workspace/all-target/all-
   feature clippy, workspace nextest and wasm32 check. Run same-round fresh
   seeded and official base/candidate controls with independent caches;
   list every status and reason change and require no unexplained
   candidate-only failure. Keep raw Native timeout INCONCLUSIVE and oracle
   nondeterminism non-PASS. Any infrastructure change must be recorded on
   both sides.
6. Scarf is outside this slice.

No preemptive helper framework, generic seam API, unit tests after
implementation, unrelated cleanup or Ares-owned pipeline design is included.
