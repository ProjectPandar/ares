# SPEC: PrintObject multi-instance brim area and collection

Status: **SPEC-REVIEW-REQUEST**. Proposal `676e8deb15df58429275d19d01fd791958d94613`
approved; this formal SPEC still requires review before public RED.
Production base `cf8597e183d44bd24a28959e4dbd904d3a90512f` (approved global
combined-brim chain). Proposal/SPEC are docs-only descendants of that exact base;
no production change since it. Source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Observed input and exact contract

Prior source/input provenance and failure inventory:
[multi-instance-brim-collection-proposal.md](multi-instance-brim-collection-proposal.md).
Fresh discovery at exact clean cf8597e18:
`/tmp/ares-copies-premise-cf8597e18/discovery-report.json`.
Native FIRST accepts all cases, both independent official/seeded modes, actual
Native/public descendants and threads CPU18, API==CLI raw.

Input has object2 instances (80,110,2)/(140,110,2) and object3 instance
(110,110,2), all cube10.stl/Z scale0.4; ByLayer, one filament, outer_only width8,
gap0, combine_brims0, combined skirt2, no support/raft/tower.

- Target with skirt: complete strict FAIL byte1608/line67. Native first skirt
  point X74.265 Y90.954, Ares X73.627 Y90.998. Native TWO brim blocks (one owner
  collection contains both copies), Ares THREE (one block per copy).
- Same-input brim_width0 control: complete strict Native/API/CLI PASS both modes,
  raw API==CLI. Non-config archive members unchanged; no other setting differs.
- Same-input skirt_loops0 localizer: complete strict FAIL byte6605/line213,
  Ares inserts M73 P8 R10 where Native emits the next brim G1. Native still TWO
  brim blocks/Ares THREE. It is a second failing witness, NEVER a passing control.

These observations persist after stack24/global-combine: combine_brims0 does
not execute the approved global merge branch. No claim of a new independent
Ares pipeline feature or defensive guard improvement. Coordinator approved this
slice after checking its proposal; other workers' skirt/tower/support predicates
and queued chains are not implementation targets.

## Exact upstream ownership and Rust boundary

Port the following cohesive source lifecycle:

1. `OrcaSlicer/src/libslic3r/Brim.cpp:36-45::append_and_translate`: move each
   instance area using shift_without_plate_offset, subtract destination occupied
   areas, append under the instance's PRINT OBJECT ID, not a per-copy identity.
   `outer_inner_brim_area:573-581` expands ALL object instances and associated
   no-brim/holes/contact islands. Keep source clipping/contact frame and order
   through :636-689; no cloning already-clipped final emitted polylines.
2. `Brim.cpp:914-956::make_brim`: object/brim bounds and occupied area maps retain
   every instance; store plate-translated per-owner footprints before extrusion
   path construction. `Print.cpp:2794-2816::_make_skirt` expands object hulls per
   instance then consumes ALL contours in those occupied brim maps.
3. Ordinary combine0 `Brim.cpp:964-977`: `makeBrimInfill` ONCE per PrintObject's
   complete area. Use existing source infill primitives :813-886. Different
   owners stay separate; this is NOT global union/cleanup into a single carrier.
   Preserve the already-approved global branch :979-1018 unchanged in behavior.
4. `GCode.cpp:4486-4531::generate_object_brim`: print the complete owner's/group's
   collection at origin0, erase emitted owner IDs, do not re-emit for later copies.
   Preserve group ownership/order from `Print.cpp:2847-2971` and call placement
   before model extrusion at `GCode.cpp:5334-5335`.

Rust destination is the existing private
`crates/ares-core/src/project_slice/gcode_emit/` preparation/emission seam:
`brim.rs::BrimPlan::generate`, `brim/bed_area.rs::clip`, `brim/emit.rs`,
`brim/combined.rs` only where shared area lifetime/frame demands it,
`skirt/occupied.rs::items`, `layers/brim_emit.rs::emit`,
`motion/state.rs` for source once-per-owner bookkeeping, and
`orchestration.rs::emit` for preparation wiring. Current ordinary local brim
paths/hull are a temporary singleton compatibility shell, not a new source-owned
pipeline. Replace their multi-instance assumptions with the cited owner-area
representation/consumer. Keep model-local, plate-offset-free area and final
plate frame distinctions explicit; don't double-shift occupied points or paths.
Changes outside this boundary require STOP/amendment, not a speculative helper.

## Red-first sequence and admission gate

After SPEC approval ONLY, freeze three input-only cases plus SHA/member manifest:
with-skirt target, no-skirt target, and width0 paired control. Public tests use
OrcaRunner::from_env and Native generated at runtime FIRST; require Native
acceptance, positive model extrusion, effective config, expected brim owner-block
count and skirt presence/absence. Counts/anchors supplement full comparison.
Then call validated API and real CLI on identical bytes/metadata. Assert raw
API==CLI and complete ordered Native equality with UNCHANGED
`golden::compare_ordered_bytes_generator_only` for EVERY case. Both targets
must genuinely FAIL before GREEN; control must PASS. No assert-ne, divergent-is-
success expectation, comparator relaxation, committed Native output, or M73 skip.
Freeze exact input hashes; archive and compare all non-config members/settings.

Fresh official/seeded roots/caches, actual descendant/thread CPU18, jobs2,
threads2, canonical snapshot/hash/reference settings, exact clean hash before/
after. Two genuine FAIL + one PASS expected per mode at public RED, then submit
RED REVIEW-REQUEST and await independent approval before production.

After RED approval, first record owner-area/frame/instance and emitted-ID
provenance; only then implement the source lifecycle. All three cases must pass
full comparison at exact GREEN hash, with width0 control raw API/CLI bytes stable
RED→GREEN. Existing singleton/grouped brim/skirt/global-combine tests unchanged;
run authorized targeted families, fmt, strict clippy, WASM. Submit GREEN
REVIEW-REQUEST with durable artifact path and residual limitations, no merge/push.

## Non-negotiable scope, failure modes and STOP

Included: this exact separated two-owner/three-instance geometry, ordinary
combine0 outer_only ByLayer and its same-input witnesses/control. Preserve source
per-owner occupied area/infill-once/emitted-ID lifecycle. Maintain all existing
perobject skirt multi-copy/combine predicates; this slice does NOT admit them.
No coordinate snap, output rewriting, added acceptance/defensive guard, arbitrary
fallback, core I/O/env, new public API or independent pipeline. Split private
modules only as necessary to keep each changed Rust file <=400 LOC.

Deferred: ByObject, perobject-skirt multi-copy admission, combine1+copies beyond
existing behavior, overlap/touching copies/new geometry, inner/auto/ears/painted,
support/raft/tower/multi-filament. Existing source-owned compatibility shells
outside the named owner boundary stay unchanged.

M73 is an observed symptom, NOT permission for estimator changes. If a faithful
owner-area/collection port leaves ANY residual whole-output difference (including
M73), STOP/report with first difference and exact artifacts. Do not iterate on
estimator, timing, normalization or another owner under this approval. Wrong
frame/owner, missing island, duplicated brim, changed singleton/control bytes,
or unexpected skirt footprint are real full-output failures, not tolerated noise.
If Native rejects or the input is undefined, NO-RED STOP for that witness;
compiler/environment/affinity/preparation faults are NON-GATE, retained and never
called parity RED. All-match discovery on a changed base means NO-RED STOP,
not fabricated RED. FULL freeze remains targeted-only, no full/workspace runs.
