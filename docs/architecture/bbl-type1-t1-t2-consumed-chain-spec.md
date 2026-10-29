# SPEC: T1–T2 BBL Type1 activation and consumed planning chain

Status: docs/SPEC review request only. Owner W5. Milestone approved and committed
as `8a22fa596`; see [Type1 ARD](ard-bbl-type1-wipe-tower-port.md).
No public RED or production work authorized yet. Original W5 window stack stays
frozen at `55a81063b`; this specification lives on the separate docs branch.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, paths below relative to
`OrcaSlicer/src/libslic3r`.

## Contract and completion boundary

Port `Print::wipe_tower_type` / `has_wipe_tower` through real `ToolOrdering`,
Type1 `Print::_make_wipe_tower` planning, and `WipeTower::plan_tower_new` into
private `ares-core::project_slice::gcode_emit` modules. Existing preparation is
only a compatibility shell around those upstream concepts.

T1 and T2 constitute one consumed preparation chain, not standalone public
features. Its output must feed the actual `generate_new` consumer and existing
schedule/runtime/footprint consumers through subsequent T3/T4 work. No dormant
planner facade, public planning API, dead fields or Type2 substitute qualifies.
T1/T2 may be reviewed as source/design checkpoints, but cannot be integrated or
called GREEN while public whole-stream RED remains. T3/T4 are explicit required
completion dependencies, not permission to implement them under this SPEC.

## Baseline and reproducibility

Use the accepted official-exported stock X1C project, two cube10 objects,
Bambu PLA Basic + Bambu PLA Matte, distinct AMS slots on one nozzle. Unchanged
stock process/machine/filament templates. Input SHA256:
`ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506`.
Evidence root `/home/indexyz/ares-coord/evidence/w5-stock-multi-materialized/`;
`0/input.3mf`, `0/verified.json`, raw streams/receipts and
`t1-t2-baseline-settings.json`. This is discovery evidence, not a registered RED.
The slot-materialization and official re-export history is retained; do not
silently substitute a different preset flattening/export path for public RED.

Native official/seeded succeeds, repeats stably, and emits tower blocks;
current Ares succeeds but omits them. Header, motion, labels and stock G29 differ.
No expected Native byte constants belong in tests. Output values in discovery
are evidence, not matching heuristics or an implementation recipe.

## T1: separate identity, activation, effective type and priming order

Sources: `Print.hpp:1084-1086`, `Print.cpp:3290-3301,3390-3444`,
`GCode/ToolOrdering.hpp:236`, `ToolOrdering.cpp:343-371,921-954`.

1. Retain existing printer identity at `gcode_emit::tags`; effective type is
   Type1 for BBL, otherwise configured type. Do not infer BBL from a single
   printer-model string or rewrite serialized `wipe_tower_type` to force it.
2. Apply source `has_wipe_tower` conditions and real layer/tool availability.
   Keep effective type distinct from enablement. Stock BBL has Type1 enabled
   even though its stored option is Type2.
3. Preserve `prime_multi_material = effective_type == Type2` passed into
   `ToolOrdering`. Type1 does NOT apply Type2's all-tools priming reorder in
   `collect_extruder_statistics`. Tower-enabled is not this priming boolean.
4. Dispatch source-owned planning after ordering; Type1 starts from actual
   `first_extruder`, not Type2's last-primed tool. BBL front-edge priming remains
   absent as in `Print.cpp:3453-3455`.

Rust destination: `gcode_emit::{wipe_tower,tool_ordering,tags}`, existing
`tool_ordering::statistics`, schedule preparation callers. Replace the narrow
meaning of `type2_requested` / `ToolOrdering::type2_tower` where they currently
own all tower availability. Use source-type dispatch, not parallel contradictory
booleans or a new generic pipeline. Keep existing Type2 behavior/regressions.

## T2: ordered source transformations

Sources: `Print.cpp:3444-3541`; `GCode/WipeTower.cpp:1461-1611,2449-2510`,
`:3516-3806,3864-3872`; shared structures in `GCode/WipeTower.hpp`.

- Construct Type1 state from the actual initial tool, plate origin, last layer
  height, config and used filaments; retain constructor f32 casts, perimeter
  width and filament-specific properties. Read vectors with source get_at
  semantics; do not index one-nozzle vectors as if there were two nozzles.
- Retain filament slots separately from nozzle indices. Map entries are 1-based;
  per-nozzle current filament uses the source unset state, not a default tool.
  Build nozzle-specific flush matrices with source cast/multiplication order.
- For each tower layer, add the same-tool layer marker, then source-ordered real
  transitions. Skip unchanged filament. Compute previous-nozzle flush volume,
  multiplier and whole-collection wiping allocation; subtract grab-volume and
  clamp as upstream. Pass prime_volume and purge_volume separately.
- Preserve `ensure_perimeters_infills_order` after allocation. Do not copy
  Type2's minimum-purge subtraction/addition or prime-volume matrix substitution.
  Reuse ARD-0105 collection/copy identities and existing wiping allocator, not
  independently flattened paths. No reordered model/tool buckets.
- Honor source last/next-partition stop versus wrapping/timelapse continuation.
  Retain sorted used-filament IDs and configured adhesiveness categories.
- `plan_toolchange` adds layers with WT_EPSILON, records first layer, returns
  for equal tools, computes ceil-based required depth from prime volume, and
  retains distinct nozzle-change depth only for actual nozzle transitions.
- The live `generate_new` first sets actual tower height then calls
  `plan_tower_new`. Port block accumulation by category, per-layer maxima,
  four-layer solid-fill lookahead, backward depth propagation and framework
  behavior from `generate_wipe_tower_blocks`; no single-category special case.
- Rib planning is required for the baseline: initial block construction,
  square-width alignment, recalculated transition depths, second block build,
  height-derived depth/spacing handling, first-layer solid-fill adjustment,
  `update_all_layer_depth`, rib length and width clamps. Preserve source operation
  order, float math and integer conversions; do not replace it with a rectangle.

Destination: private `wipe_tower/type1/{planning,plan,blocks}.rs`, splitting by
source function below 400 LOC, plus existing `wipe_tower::{planning,data}` and
`tool_ordering::wiping` consumers. Shared result/data carriers remain internal;
only source-shared semantics are reused from Type2. No native FFI/fallback,
filesystem, terminal or UI in core. WASM/Windows/macOS/Linux remain Tier1.

## Reachable-branch inventory for the X1C baseline

Classification: **config** = derived from exported input plus cited predicate;
**observed** = supported by retained successful Native output; **dynamic** =
source branch in the live chain whose exact sub-outcome needs later diagnosis.
This is not a claim to have traced internal execution with a debugger.

| Source branch | Baseline selection/evidence | Obligation |
| --- | --- | --- |
| Print.hpp effective type | config: BBL identity; stored type2 → Type1 | T1 override without config mutation |
| Print.cpp has_wipe_tower | config: enabled1, spiral0, diameter vector length2; observed tower output | ordinary multi-material path |
| Smooth/wrapping activation | config: timelapse0, wrapping0 | not baseline coverage; retain source gates |
| ToolOrdering statistics | config/source: prime_multi_material=false for Type1 | no Type2 priming reorder |
| Ordering has_wipe_tower | observed Native tower; exact per-layer metadata not traced | actual first tool/front-layer predicate, never unconditional enable |
| Constructor wall/spacing | config: rib, width35, brim3, gap150%, rib width8, extra rib0, fillet1 | rib path IN scope, spacing1.5 initialization |
| Constructor framework | config: framework0 | false branch; do not hardcode false |
| Sparse selection | config: no_sparse_layers0 | ordinary first-plan-layer selection |
| Material setup | config: PLA/PLA, category100/100, soluble0/0, support0/0, wipe_tower_filament0 | source two-slot state; one category block inferred |
| Filament/nozzle transition | config: nozzle count1, map1/1; observed repeated T1 | same-nozzle transition; no nozzle-change added depth |
| Flush accounting | config: off-diagonal280, multiplier0.3, grab0, prime45 | f32 flush product and separate prime/purge fields; no swapping them |
| Wiping allocation | config: into-infill0, into-objects0, into-support1 but support disabled/raft0 | retain allocator call; no positive support-purge coverage |
| Layer marker / changed-tool | source plus observed two tools | both equal marker and changed transition paths; exact sequence remains runtime-owned |
| Last-layer / next partitions stop | dynamic; wrapping/smooth continuation false | preserve both stop predicates, do not guess plan length from output count |
| Rib width recalculation | config rib=true, live plan_tower_new | initial and repeated block generation both required |
| Auto brim | config brim3 | negative-width auto branch not exercised |
| Category blocks | config both100; source accumulation | one category expected; retain general accumulation and backward propagation |
| Zero-depth/solid lookahead | dynamic per-layer depths | retain 4-layer lookahead, not assumed inactive |
| Height minimum comparison | dynamic computed depths/height | retain exact comparison, do not assert unmeasured result |
| First-layer extra-spacing | configured initial1.5; source may increase it | retain first-layer solid-fill arithmetic and later-layer multiplication |
| Wrapping/timelapse depth fallback | config false | not baseline coverage |
| T3 flags carried by constructor | config skip_points1, flat_ironing0, interface_features0, interface_cooldown0 | retain real consumers in T3, not dummy reads |
| T3/T4 live outputs | observed tower blocks and expanded G29 bounds | required later consumers; T1/T2 alone cannot pass |

Same-category X1C does not qualify multi-category hash traversal ordering.
Do not introduce a sorted order as an assumed equivalent for upstream unordered
category maps. Qualification of a multi-category stock witness and exact native
ordering is a later source gate, not permission to add a one-category guard.
Multi-nozzle, TPU, soluble/support tower material, auto brim, framework, smooth,
wrapping, sparse and non-BBL Type1 paths are not covered by this baseline.
Their implementation/qualification remains in the parent milestone inventory;
no runtime restriction or claim of complete Type1 parity follows from this SPEC.

## Public RED plan — permission still required

After SPEC approval, register the accepted input through existing CLI parity
NativeCase/`native_stream_case::check(..., (false, |_, _| {}))` conventions.
Compare complete ordered Native/API/actual CLI streams and raw API==CLI; require
Native success first. Input hash only, never checked-in Native outputs or expected
G-code constants. No private planner tests as an acceptance substitute.

Baseline public RED must reproduce missing Type1 output in official then seeded
with separate fresh caches and retained raw streams, launch receipts and exits.
If exporting through the shared fixture path changes the accepted input, stop
for qualification rather than silently claiming the old input's evidence.
A second GUI prime-volume variant and prime-tower-disabled control require
fresh Native qualification before registration; their status is unmeasured,
not predetermined RED/PASS. No literal probes or template overrides.

Freeze approved RED tests before production. Do not turn whole-output failure
into an expected-failure PASS, mask headers/timing, or assert just G29/tower
markers. Residual label/geometry/template differences are blockers to whole
GREEN, not reasons to widen this SPEC silently.

## Exit and deferred work

This docs stage exits with reviewed source inventory and exact SPEC hash only.
Later T1/T2 review requires real connected producer/consumer implementation,
not an unused planning library; final milestone GREEN requires T3 emission and
T4 live geometry with complete Native parity. No partial activation merges.

T3 owns writer/toolchange/finish, `append_tcr` and `is_finish_first`;
T4 owns generated bbox/rib/outer-wall/hull and live consumers. Neither may be
faked to make preparation appear complete. Preview-only metadata, multi-transform
PrintObject preparation and unrelated placeholder fixes remain deferred as in
ARD-0105. Coordinate shared skirt/brim/chamber edits with their owners.

CPU11 serial; `cargo nextest` only when RED is authorized, official→seeded
fresh caches. Final production validation includes fmt, strict workspace/all-
target Clippy, browser-WASM and coordinator-window FULL at exact integrated tip.
No tests, production edits, FULL, rebase, push or merge for this SPEC request.
