# SPEC RR: painted MMU → beam interlocking call order

Status: bounded discovery complete; request independent SPEC approval before public RED.
No production edits or RED authorization implied. Base `2ed045a7f235a592ca5332ece6fdd21b685837e1`.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
[Non-negotiable decisions](ard-painted-beam-call-order.md).

## Reconciled backlog and upstream owner

Legacy #70–#73 carrier/RED/segmentation work is superseded by merged public RED
`8c25b6741`, complete consumed MMU `f569a2cea`, review fixes `4b9269ccc`,
complete-unit SPEC `29df9fc13`, and side-cut SPEC/GREEN `2be465ff3`/`85552bc43`.
All are ancestors of this base. Do not revive old parser rejection or rewrite
historical sealed branches. This new slice covers only the deferred MMU+beam
combination and its owning upstream call-order handoff.

Paths below are under `OrcaSlicer/src/libslic3r/`:
- `PrintObjectSlice.cpp:1207–1224` detects MMU painting and calls
  `apply_mm_segmentation` (helper874–1030 replaces actual LayerRegion slices).
- `1227–1241` applies fuzzy painting if present; this adjacent unsupported
  annotation domain remains deferred, not silently combined with MMU here.
- `1243` invokes `InterlockingGenerator::generate_interlocking_structure`;
  compensation/make_slices follows1250–1254. Order is MMU → fuzzy → interlocking.
- `MultiMaterialSegmentation.cpp:1955–2194 segmentation_by_painting`,
  `2197–2207 multi_material_segmentation_by_painting`: beam flag suppresses
  side cutting2169–2170, not the complete producer, caps, merge or apply.
- `Feature/Interlocking/InterlockingGenerator.cpp:25–65` owns enabled/numeric
  gates, kernels and ordered region pairs using external-perimeter extruders.
  Reuse the existing Rust port; this is not a new beam geometry engine.

GUI provenance (not an interactive GUI execution claim):
`src/slic3r/GUI/Tab.cpp:2691–2697` exposes beam controls;
`ConfigManipulation.cpp:987–991` switches beam/depth controls;
`PrintConfig.cpp:4226–4277` defines enable, width, angle, layer count, depth,
and boundary avoidance. All selected values are within those option ranges.

## Literal Rust destination and temporary compatibility shell

Paths below are under `crates/ares-core/src/project_slice/`:
- Remove the early `region_slices::interlocking::apply` call from
  `load_stages.rs::prepare_post_regions:120–125` in separately authorized GREEN.
- Invoke that SAME existing private consumer once for each real resolved
  print-object context after `prepare_post_painted` has applied MMU and before
  `compensation.rs::prepare_post_compensation` begins compensation. Prefer a
  private post-interlocking handoff in load_stages, directly consumed by the
  existing compensation handoff; no public phase API or alternate route.
- Preserve source order for unpainted objects too: do not retain an early
  unpainted path plus a new painted path, or apply interlocking twice. The
  current early call is temporary placement of the cited upstream operation.
- Retire only `painted/apply.rs::ObjectRegions::new` active-MMU+beam rejection.
  Remove the now-unused private options argument mechanically if appropriate;
  no API shim. Do not relax other annotation/admission checks or add a registry.
- Preserve `painted/mod.rs` beam suppression of side cut, all caps/merge/shared
  registry identities/ordinary-vs-strict exports, and actual apply output.
- Existing `region_slices/interlocking/{mod,micro,thin,voxel}.rs` remains the
  source-owned implementation. Do not change its geometry algorithms merely
  to force this test GREEN. A new algorithmic discrepancy requires STOP/RR.

The fuzzy stage remains in its upstream conceptual position between MMU and
interlocking; this slice does not add a fuzzy producer, dummy function or new
admission. Existing unsupported fuzzy input remains unsupported.

## Frozen bounded discovery (not public RED acceptance)

Evidence: `/home/indexyz/ares-evidence/worker1/painted-beam-discovery-2ed045a7f/`.
Reproduced existing frozen paint-color split/unpainted archives, changed only
`Metadata/project_settings.config`; all geometry/paint/other members retained.
Both pairs use width0.8mm, orientation22.5°, beam layers2, depth2, boundary
avoidance0; pair members differ only in beam enable. Width/depth side-cut
settings remain zero. Inputs/member/settings provenance is in `inputs.json`.

| Case | Archive SHA256 |
|---|---|
| painted-split-beam-off |011b786ebc55a3208c554e4f87c50c35d2473a8cb737485f1b35001f728eceb2|
| painted-split-beam-on |703e1d4cf4ade1e7aa93beb0a5ae7950e9633ad76fa0a610f06548ee36963bf8|
| unpainted-beam-off |a50315e5965d3d000c4e946756bcdbb8167c76f03b5c6163f32667742882a564|
| unpainted-beam-on |682b9a6d5eb164d407a567d61de929f21564da22c2c7839853913e51a9584330|

16 actual Native processes: four inputs × two modes × reference/replay,
all exit0 and complete generator-only replays equal, before any Ares call.
Both tools have positive model XY+extrusion on painted inputs. In BOTH modes,
beam-off activity is T0=3674/T1=900; beam-on T0=6329/T1=3369. Ordered XYE
command sequence changes (5062→10745); unpainted sequences remain unchanged
(4017). These are diagnostic observations, NOT committed Native constants,
movement goldens or a requirement to match fixed counts. Beam setting readback
and identical paired geometry provide source-active configuration evidence;
no private geometry capture or interactive GUI/cross-platform proof claimed.

Four real Ares CLI calls using the FULL-qualified identical-tree binary:
painted beam-on rejects `painted beam compositions` (exit1), while the other
three cases succeed and match complete Native bytes in each mode with only
existing generator-line allowance. This is discovery of current boundary,
not expected-error PASS, API qualification or public RED.

Initial discovery driver stopped after the first CLI control because it used
the Native generator regex on the Ares header. Raw diff was the generator line
only; continuation used the exact Ares pattern in existing golden.rs, retained
all prior outputs and launched no extra Native processes. Both logs/exits are
retained; failure is not semantic RED or successful gate credit.

## Test-first RED plan after SPEC approval

Use existing shared public NativeCase/native_stream_case and activity helpers;
no copied process harness or new comparator. Freeze only necessary target input
plus beam-off positive control; reuse existing unpainted/interlocking tests where
adequate. Discovery's unpainted single-material on/off is an inactive negative
control, NOT proof of preserving active unpainted two-material interlocking.
Select existing active unpainted interlocking cases as regression obligations.

Global Native-first: both modes, each selected frozen input, reference/replay
before ANY API/CLI call. Require positive model XY AND positive extrusion for
both painted materials, beam-setting readback, repeat equality, and paired beam
activity without hardcoded Native counts or output filtering. Retain every raw
stream and launch/resource/affinity identity. Then run public success-required
full ordered comparisons against API and real CLI, raw API==CLI unchanged.

Expected actual RED: painted+beam must FAIL public success/output assertion;
beam-off and existing controls must PASS. Unsupported is not an expected-error
success condition. Any Native/resource/replay/activity/control failure is
STOP/non-gate. Preserve CLI output/error even if API rejects. No private-entry
phase acceptance, should_panic, mocks, tolerances, extra normalizations or goldens.
Submit RED RR and wait for explicit GREEN authority.

## Pre-code failure inventory and acceptance

Failure modes: early rejection lifted with pre-MMU geometry; duplicate consumer;
wrong context or resolved nozzle mapping; top-empty/conical order drift; wrong
region/extruder pairing; beam accidentally enabling side-cut; parent/default
state loss; cap/merge/compensation drift; active unpainted interlocking regression;
claims based only on selectors or inactive interfaces; hidden comparator change.

GREEN later must change only literal ordering + combination-specific rejection,
with mechanical call adaptation; pass frozen public whole-output cases in both
modes and existing painted, side-cut, conical, unpainted interlocking controls.
No production implementation before independently reviewed RED and GREEN go.
Final exact-tip fmt, strict all-target Clippy, WASM compile and dual canonical
FULL require separately assigned integration window. CPU18/jobs1/test-threads1/
ARES_PARITY_THREADS1. Files≤400LOC; no include tricks or new crate/public API.

Deferred: fuzzy painting and combined annotations; new support/raft/tower/custom
multi-object domain expansion; UI/cache framework; geometry backend changes;
Windows/macOS/browser runtime claims. Existing core stays portable data-only.
