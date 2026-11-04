# SPEC request: ByObject transition comments, then per-object skirt

Two independent slices, in order. Discovery ran on
`41f2c90f765d42c6620583121296bb61a2cb839c`; pinned Orca source
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Verified discovery and fixture identity

Discovery retains fresh official
and seeded Native-first runs, input hashes, exact binaries/commands and public
API/CLI comparisons. Production was unchanged.

Source `tests/parity/per-object-skirt/disjoint.3mf`: two distinct objects at
x80/x140,y110, height4; one filament; skirt2/height1; no_brim; combine_brims0;
support0,raft0,tower0. Geometry/non-config ZIP members unchanged. Cases:

- ByLayer/perobject control: whole Native/API/CLI PASS both modes, raw API=CLI.
- ByObject/combined comments-on: Native accepts, API/CLI accept, but whole-file
  fails at byte55670/line1916. Native safety Z has no comment, next lift has
  `normal lift Z`; Ares writes empty ` ; ` descriptions. Both modes identical.
- ByObject/perobject comments-on: Native accepts, 40 layers and two skirt blocks;
  Ares UnsupportedProjectFeature("skirt_type per-object"), CLI1/no G-code.
  Native target also contains the same transition lines: comment repair is a
  prerequisite for its eventual whole-output GREEN, not an ignorable control.

### combine_brims verification (current pin, not generic version assumption)

The key IS registered in this checkout at
`OrcaSlicer/src/libslic3r/PrintConfig.cpp:1744` (`add("combine_brims", coBool)`),
and typed in `PrintConfig.hpp:1631`. Both official and seeded Native target
footers explicitly contain `; combine_brims = 0` (line3852). Thus fixture text
refers to a verified source-pin/runtime key, not an inferred upstream-release
feature. Do not claim every stock build labeled 2.4.2 necessarily has it; retain
source and executable identities. The scope is no_brim, so no combined-brim
behavior is being exercised or added.

## Slice 1 FIRST: sequential transition-comment port

Upstream exact owner:
- `OrcaSlicer/src/libslic3r/GCode.cpp:3321-3322`: safety `travel_to_z` has default
  empty comment; subsequent `travel_to` describes `move to origin position for
  next object` and invokes the ordinary lift path.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:266-267::emit_comment`: only emit a
  comment when comments are allowed AND the supplied description is nonempty.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:678`: ordinary lift description is
  `normal lift Z`. Preserve travel origin description under the same comment
  option. No fabricated empty semicolon, no changed coordinates/feedrate/state.

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/layers/transition.rs:63-86::emit`
and its existing immediate writer consumer
`crates/ares-core/src/project_slice/gcode_emit/motion/tower.rs::writer_z` only
if required to express the source empty-comment rule. The current helper emits
`format!(" ; {comment}")` whenever comments are enabled, even for empty input;
transition passes empty strings for both Z commands and hard-codes the origin
move without forwarding its source comment. Any shared-helper change must
preserve other callers' source-owned nonempty comments and be explicitly audited.
No travel/retract/lift algorithm, scheduling, skirt guard, model, or formatter
numeric changes. Existing emitter is a compatibility shell for these GCode/
GCodeWriter calls, not an independent Ares transition policy.

Public Native-oracle RED: freeze the already failing
ByObject/combined comments-on input as input-only 3MF with SHA manifest; runtime
Native FIRST, then strict complete ordered Native/API/CLI equality and raw
API==CLI. Assert three transition anchors (uncommented safety Z, normal-lift
comment, origin description) in Native; do not replace whole-output comparison
with anchors. RED fails on the real full-file mismatch, not inverse assertions.
Control: same ByObject/combined input with comments-off, also strict whole-output
comparison, plus existing `crates/ares-cli/tests/orca_parity/runner/
print_sequence_by_object.rs` tests unchanged. Comments-off is an explicit paired
control, NOT a substitute for comments-on or a way to hide the bug.
If the off control has another failure, retain it and resolve it before GREEN.

Slice 1 GREEN is minimal and follows its RED. Same inputs and
assertions; controls raw bytes RED→GREEN unchanged. Slice 1 must be GREEN
before Slice 2 implementation. No incidental skirt admission repair.

## Slice 2 AFTER Slice 1 GREEN: perobject skirt × ByObject

Upstream boundary:
- `Print.cpp:2646::_make_skirt`, :2780-2977: instance-shifted object hulls,
  occupied footprints, nominal-reach grouping and own-group skirts; no blanket
  ByObject prohibition.
- **`Print.cpp:2548-2557`**: after grouping, ByObject/perobject SHARED groups
  throw `SlicingError`. Keep this case guarded; do not admit overlapping groups
  by merely deleting the blanket disjunct. If existing Rust group admission
  cannot enforce this rule, stop for an amendment/source-owned check before
  removing the blanket guard. No silently unsupported shared groups.
- `Print.cpp:1286-1303::validate`, :613::sequential_print_clearance_valid:
  smooth-timelapse/clumping/clearance/height/order constraints remain unchanged.
- `GCode.cpp:2730-2762` orders model instances/ToolOrdering; :3280-3374 emits
  the complete stack per instance and its transition; :5127-5136 preserves
  sequential per-filament instance ordering.
- `GCode.cpp:2036` clears `m_skirt_group_done` at export start; :5240 resizes it;
  :4465-4483::generate_object_skirt_group and :5334 feed each group's own history
  into `generate_skirt`. NO blanket group-history reset between objects.
  `skirt_done` is the group-vector reference; :4286-4328 gates empty-first and
  later skirt-height/strict Z advancement; :4395-4414 derives flow height.

Proposed Rust boundary:
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`
ONLY its sequence disjunct; existing grouping and
`crates/ares-core/src/project_slice/gcode_emit/layers/{object_skirt,drive}.rs`
remain source-owned consumers, not a new sequential pipeline. Shared-group
admission must stay guarded as above. Tower/multi-filament and support/raft
disjuncts stay unchanged.

Public RED: target ByObject/perobject input above, Native first at runtime;
assert Native acceptance, two skirt blocks, and second block AFTER the
`move to origin position for next object` transition into object two and BEFORE
its object extrusion. Bind distinct object IDs/order and positive skirt XY/E;
strict whole Native/API/CLI equality, raw API==CLI. Before GREEN must fail on
real Ares unsupported rejection. Controls: ByLayer/perobject and now-fixed
ByObject/combined, both strict whole equality. Keep shared-group rejection
covered.

## Failure inventory and gates (both independent slices)

Invalid clearance/shared groups, unsupported export, absent Native product or
wrong settings: stop; no admission bypass. Missing input/env/compiler faults,
timeout/cancel: NON-GATE, never parity RED. Wrong comments or
order remain full-output failures; no comment stripping, comparator/tolerance/
classification relaxation. No expected output copied from Ares. If already
matching, NO-RED STOP rather than a passing-only repair. If scope exceeds cited
consumer boundaries, request amendment before touching production.

Each slice: test-first frozen input-only manifest, public RED,
then minimal GREEN, unchanged tests/inputs, fresh serial official+seeded runs,
exact clean hash, existing relevant controls and
fmt/strict/WASM, changed Rust <=400 LOC. No Native outputs in Git.

Deferred: shared-group SUPPORT (rejection stays), repeated copies of one object,
extra skirt-height/draft/minimum combinations, brim, support/raft, tower/multiple
filaments and geometry/placement/clearance rewrites. Existing architecture stands;
no new architecture or ARD needed.
