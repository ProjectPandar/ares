# Pre-external wipe: emit the upstream force-no-extrusion path

Status: SPEC approved; formal matrix narrowed by coordinator after an
independent matched-control residual. Native-first RED precedes production.
Parent: reviewed W6 stack tip `261c076794c07c2fd07686550dfb9ba94c5a414d`.
Branch: `worker6/pre-external-retract-spec`. No FULL window requested.

## Owning upstream boundary

Port `GCode.cpp::GCode::extrude_loop:5884-5888`: after the existing
`discoveredTouchingLines > 1` guard, upstream creates a two-point
`ExtrusionPath fake_path_wipe`, copies the external path's properties, sets
`force_no_extrusion=true`, sets `mm3_per_mm=0`, and calls ordinary
`extrude_path(..., "move inwards before retraction/seam", speed)`.

That call consumes the existing path-start pipeline:
`GCode.cpp::extrude_path:6109`, `_extrude:6368-6394`,
`travel_to:7415-7448`, and `retract:7654-7674`. Force-no-extrusion suppresses
material deposition, not routing/retraction/wipe/unretract or motion timing.
The owning behavior is this existing upstream path, not a new Ares travel
policy. The earlier pre-wipe eligibility, point computation and touching-line
search remain the producer boundary at `GCode.cpp:5823-5883`.

## Exact Rust boundary and temporary shell replacement

Rust destination: existing private
`project_slice/gcode_emit/motion/loop_paths.rs::append_wipe_before_external`,
with the consumed standard path-emission/start-travel boundary in
`motion/materialized.rs`, `motion/path.rs` and `motion/path/start_travel`.

Current code computes the pre-wipe point, then hand-emits travel/lift/descend
G1s and defers its return segment through
`EmitState::pending_wipe_before_external_target` / `path.rs` consumption.
This manual sequence is a temporary compatibility shell for the upstream
fake ExtrusionPath. Replace that shell with the source fake-path semantics
through the existing ordinary path machinery, rather than adding another
manual retract sequence or copying travel policy into the helper.

If the source `force_no_extrusion` property needs explicit internal plumbing,
keep it private and faithful to the named upstream flag. Do not disguise it
as a special filename, guessed flow, dummy positive extrusion, fallback path,
or a public API/configuration mode. Eliminate only state/imports/tests made
orphaned by replacing this shell. Do not refactor unrelated motion behavior.

## Native-first discovery from a real exported project

Use the existing single-volume exported 3MF unchanged in geometry and metadata.
Only project settings differ: enable `wipe_before_external_loop` and set the
physical extruder's `retraction_minimum_travel` to 0.2 mm.

Fixed Native double replay is stable in both official and seeded modes. The
complete Native/CLI target streams differ first at byte 3065, line 200:
Native `G1 X113.725 Y113.725 E-.12238`, Rust `M73 P10 R8`; full lengths are
101694 versus 100844 bytes. This is not admitted as a standalone M73 defect.
Closing the pre-wipe gate at the same threshold, or restoring the default
threshold with pre-wipe still enabled, gives complete-stream PASS controls.
The original two-volume export with only pre-wipe enabled also differs, but
is not needed to prove this smaller single-volume failure.

Read-only GDB confirms a matching path at Z=0.6:

- Native stack: `extrude_loop:5888 -> extrude_path:6109 -> _extrude:6378 ->
  travel_to:7440 -> retract:7654`, with the fake-path description.
- Rust manually emits a hop from `(113.585002,113.525002)` to
  `(114.175001,113.775001)`, distance 0.64077946284349 mm, with threshold
  0.2 and `retracted=false`, `retracted_amount=0`.

Evidence: `${EVIDENCE_ROOT}/w6-prewipe-fakepath-readonly-producer.json`.
Fixed replay artifacts:
`${EVIDENCE_ROOT}/w6-prewipe-fixed-{official,seeded}`.
These are discovery CLI receipts, not a substitute for the authorized public
API RED. No Native output or probe is committed. No GREEN has been attempted.

## Deferred W5 order/Z diagnostic and prerequisite

The real exported W5 scarf/min-50 witness is **not in this slice's formal
matrix**. It is retained outside Git as a diagnostic: Native finishes the fake
path at `X119.775 Y100.225` before the real scarf-head travel to `Y100.305 Z0.2`,
whereas the temporary pending marker consumes the finish after that travel.
After GREEN, confirm motion order/Z alignment with GDB/ordered motion records
and report the result, explicitly not as whole-output acceptance.

The coordinator-requested matched min-50 gate-off control was produced directly
by official Native CLI `--wipe-before-external-loop=0 --export-3mf`; its only
ZIP payload/JSON difference is the wipe switch, with min-50 preserved. However,
both modes expose an independent complete-stream discrepancy: `M73 P29 R18`
is inserted before rather than after the first two SparseInfill moves at
line 5757. It remains a strict FAIL, not a PASS control or a normalization rule.
The coordinator excluded this pair from formal RED until its GCodeProcessor /
time-estimate insertion-point prerequisite is addressed in a separate slice.
Do not search or change input settings to evade it.

Frozen diagnostics: `${EVIDENCE_ROOT}/w6-w5-prewipe-handoff/` and
`${EVIDENCE_ROOT}/w6-prewipe-min50-cli-control/exported.3mf`.
Residual/pair proof:
`${EVIDENCE_ROOT}/w6-pre-external-matched-control-blocker.json`.
No W5 diagnostic archive is submitted by the current RED. Restore the second
formal target only after the independent prerequisite is fixed and authorized.

## Included source behavior and non-negotiable boundaries

Included: the source-owned pre-external two-point no-extrusion path uses the
normal routing and retraction decision, wipe, deferred lift, unlift/unretract,
return segment and writer/cursor updates exactly once. The fake return must
complete at the ordinary layer Z before any following scarf-head XYZ travel;
no pending finish may leak into the next real path's slope/height context.
Downstream cooling and
progress must follow from those source motions, not from output correction.
Preserve the fake path's source role/properties and the surrounding real loop.
Ordinary extrusion and existing retraction thresholds remain unchanged.

Deferred: changing pre-wipe eligibility, angle rays, point rounding, nozzle
cap, touching-line selection, generic route/intersection algorithms, global
retraction rules, cooling arithmetic, M73 placement, scarf eligibility and all
interlocking/centering algorithms. No new unsupported guard or option clamp.

Any residual whole-output mismatch means STOP and report. Do not erase M73,
change comparator normalization, add an epsilon, or expand into adjacent path
or speed policies to force this slice GREEN.

## Public matrix and failure inventory

Reuse `runner/native_stream_case.rs` and existing single/default archives.
Freeze only the two unique external target/matched-control inputs below:

| Case | Initial verdict | SHA256 |
| --- | --- | --- |
| single-wipe-threshold02 | target RED | `ba4ce2095ec47e5f171ab8f1fd4be9af976e36f30a41c33dfd9c52b4be1254d7` |
| single-off-threshold02 | pre-wipe-disabled PASS | `ca9e648a6249b33973f7685923e65e3bf8222b9d115dfce5075a56c064d6b1d8` |
| original single volume | existing PASS | `1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651` |
| default two-volume beam1 | existing PASS | `04c7d2a58b7b21909b867542faad149a6b9d0b1bb976eab3fec57b7b60ff082c` |

Both official and seeded modes: Native twice before validated byte-in/options
API and real CLI; frozen SHA assertions, raw API/CLI equality, complete ordered
Native/API and Native/CLI generator-only comparisons unchanged. Keep original
launch/input/output/comparison artifacts; never remove progress commands.

Failure inventory: missing qualifying retraction; double retract/unretract or
lift consumption; emitting material on a no-extrusion segment; stale cursor
or wipe path after the hop; regressing the disabled/short-hop controls;
correcting timing independently of actual motions. Cover through the public
seam before production; no unit tests written after the implementation.

## Review sequence, size and integration gates

1. Review this docs-only source boundary/evidence before test or code changes.
2. Separately authorize and commit genuine Native-first public RED/controls in
   both modes, reusing shared harness and existing control archives.
3. Production-only GREEN ports the fake-path boundary above and makes the
   formal target and every complete control stream match. Afterwards record
   the deferred min-50 order/Z diagnostic separately, not as acceptance.
   No partial-GREEN or masked-prerequisite claim.
4. Every changed file stays below 400 lines. If a necessary extraction is
   pure movement, commit it separately before behavior with exact body and
   frozen raw-output invariance evidence. No mixed refactor/behavior commit.
5. fmt and workspace/all-target clippy `-D warnings` clean. Preserve browser
   WASM/native core separation; no filesystem/UI/terminal behavior in core.
6. Reviewed endpoint → inward → safe-zone → mesh-center stack stays unchanged
   and queued. No FULL is started/requested by this SPEC. Current order is
   W4 → W2 → W3 full stack → W6 → W5; integration later requires authorized
   then-current-main rebase and exact-hash dual FULL 0 FAIL/0 skipped.
7. No stash, push, self-merge, stale-base exemption, new allow/expect/ignore,
   fallback or committed Native output. Required coauthor on every commit.
