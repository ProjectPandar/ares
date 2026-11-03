# ARD: BBL Type1 wipe-tower rewrite milestone proposal

## Status, ownership and priority

W5 owns this coordinator-accepted stock parity gap. This document proposes
source slices; it does not authorize tests or implementation. The existing W5
integration stack remains frozen at `55a81063b7613f83f5b59f4b10aa503457589163`.
Docs are isolated on `w5/type1-milestone-proposal`. Separate approvals are
required for SPEC, public RED, GREEN and integration/FULL.

Prioritize the missing tower for stock BBL AMS prints before further literal
placeholder edge cases. Follow [ARD-0105](ard-0105-smooth-type2-timelapse-port.md)
for existing shared integration ownership, not its independent Type2 algorithm.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`; all source paths below
are relative to `OrcaSlicer/src/libslic3r`.

## Accepted witness, not yet a registered RED

Stock X1 Carbon 0.4, default 0.20mm Standard process, Bambu PLA Basic and Bambu
PLA Matte, two cube10 objects assigned separate AMS slots. Stock templates are
unchanged. Official Native exported the two-slot project after slot metadata
materialization. Input SHA256:
`ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506`.

Discovery bundle:
`${COORD_ROOT}/evidence/w5-stock-multi-materialized/`;
`PROPOSAL-RR.md`, `0/verified.json`, input, raw streams, diffs and launch receipts
are retained there. Official and seeded Native each succeed twice and agree
apart from generated-by normalization; Ares succeeds twice with stable output.
Both issue 25 T1 commands; Native has 101 WIPE_TOWER_START blocks, Ares none.
The unmodified stock start template yields Native
`G29 A X9.75224 Y115 I125.248 J134.29` versus Ares
`G29 A X121 Y115 I14 J26`. Full streams also differ in header timing, labels
and motion. This is not a one-line placeholder fix or an isolated GREEN claim.

## Non-negotiable architecture

- Port `Print`, `WipeTower` and `WipeTowerIntegration` source semantics. Existing
  `ares-core::project_slice::gcode_emit` is a temporary compatibility shell,
  not an independently designed tower pipeline.
- `Print.hpp:1086::wipe_tower_type` forces BBL to Type1 even when the serialized
  option says Type2. Preserve `is_BBL_printer` identity and source activation,
  not an Ares capability predicate. Never simply remove `!bbl` and run Type2.
- No new unsupported guards, silent omissions, feature flags or fallback.
  No fabricated tower rectangle, G29 patch or placeholder-specific correction.
- Port the live caller: `Print.cpp:3537` calls `WipeTower::generate_new`, NOT
  the older `generate` algorithm. Audit its `_new` toolchange/finish helpers;
  older helpers are included only when actually consumed (e.g. final result
  construction). Do not port ~4.5k lines indiscriminately or substitute Type2.
- Reuse ARD-0105's scheduling, collection/copy ownership, result carrier,
  runtime handoff, footprint and processor seams only where source semantics
  agree. Type1 `append_tcr` is distinct from Type2 `append_tcr2`.
- Rust core remains filesystem/terminal/UI/OpenGL-free on browser WASM,
  Windows, macOS and Linux. CLI owns fixture I/O. Keep private source-sized
  modules below 400 LOC; no new public tower API or candidate crate.
- Exact arithmetic, float/scaled conversions, command order, tool/region/copy
  identity and live accounting remain source-owned. No synthetic expected
  Native output, comparator relaxation or timing/header masking.

## Ordered source slices and GUI-reachable RED contracts

These are dependent implementation/design checkpoints, NOT independently
shippable partial activation. Every slice retains a whole-output public RED;
source-local progress does not turn that RED green. The final consumed chain
must pass before integration. Only the baseline witness is measured today;
variants below require fresh Native qualification, never assumed RED results.

### T1. Printer identity, effective type and activation

Source: `Print.hpp:1085-1086`; `Print.cpp:3290-3301::has_wipe_tower`,
`:3390-3444::_make_wipe_tower`; `ToolOrdering` construction there.
Destination: existing `gcode_emit::{tags,tool_ordering,wipe_tower}` activation
and schedule preparation. Replace Type2-only activation ownership with source
printer/type dispatch, leaving actual Type2 behavior intact.

Included: BBL override, enabled multi-filament condition, source smooth
timelapse/wrapping gates and tool-ordering handoff. Deferred to T2–T4: generation,
writer and hull consumers; activation alone is not a usable port.
RED: accepted X1C two-object Basic/Matte project with enabled tower and stored
Type2 option; Native succeeds and prints Type1 tower, current Ares omits it.
GUI control: disable prime tower in the same stock project. A control's full
PASS must be measured, not presumed from absence of a tower.

### T2. Type1 planning, material blocks and generated result ownership

Source: `Print.cpp:3444-3571` Type1 constructor, filament maps, purge allocation,
`plan_toolchange`, categories and generated data; `GCode/WipeTower.cpp`
`:1461::WipeTower`, `:1541::set_extruder`, `:2449::plan_toolchange`,
`:3516-3863` block/category/depth planning, `:3864::generate_new`.
Destination: private `gcode_emit/wipe_tower/type1/{planning,plan,blocks}.rs`
plus existing `wipe_tower::{planning,data}` and `tool_ordering::wiping` seams.
The names denote source boundaries, not a new generic planner framework.

Included: real layer/tools, AMS filament-map and category ownership, prime/
flush volume and wiping-collection allocation, block depth and source scalar
precision. Reuse existing whole-collection identities, not flattened path
substitutes. Retain source result order through schedule consumers.
Deferred: command production to T3, geometry consumers to T4; separate
multi-nozzle/non-BBL witness qualification is not certified by an X1C run.
RED: baseline plus GUI prime-volume change on the same X1C two-material job;
Native must show corresponding tower changes, Ares whole-output equality fails.
No hand-authored literal template or direct private-planner-only acceptance.

### T3. Type1 writer, tool changes, layer finish and G-code integration

Source: `GCode/WipeTower.cpp::WipeTowerWriter`, `construct_tcr`/`construct_block_tcr`
`:1240-1301`, `tool_change_new:2760`, `finish_layer_new:3036`,
`finish_block:3213`, `finish_block_solid:3320`, `toolchange_wipe_new:3390`,
`generate_new:3864`, `only_generate_out_wall:4190` and support-wall helpers.
`Print.cpp:3543-3564` constructs final purge/accounting; `GCode.cpp:712-1117`
`append_tcr`, `:1497-1599` tool-change/finalization dispatch consume results.
Destination: private `wipe_tower/type1/{writer,generate,tool_change,finish_layer}`
and existing `wipe_tower::{result,integration,runtime,postprocess}`, motion and
writer state handoff. Split further by upstream function when needed.

Included: actual movement/extrusion, temperatures, feedrates, stock filament
change templates, first/finish order, sparse layers, wipe/travel restoration,
filament statistics and processor-derived timing. Restore
`ToolChangeResult::is_finish_first` with its real `append_tcr:755` consumer,
explicitly deferred by ARD-0105. Do not copy Type2 priming: BBL removes that
priming call. Type1 final_purge construction does NOT imply emitted final
unload; upstream `finalize` emits it only for Type2.
Deferred: GUI-only preview tags/meshes with no live byte/geometry consumer;
unreachable legacy `generate` behavior is not a fallback target.
RED: baseline tool changes plus stock X1C GUI smooth-timelapse variant and
unequal object heights to exercise finish/same-tool layers. Qualify Native
reachability and success first; retain complete stream failures, not just
selected tower substrings. Finishing a writer helper alone is not GREEN.

### T4. Generated geometry to first-layer bounds and live consumers

Source: `Print.cpp:3020-3052::first_layer_wipe_tower_corners`,
`:3154-3164::finalize_first_layer_convex_hull`, `:3559-3571` data/fake outer wall,
`GCode.cpp:2926-2939` first-layer bounds; `Brim.cpp:651-656` Type1 outer-wall
consumer. Include ARD-0105's already identified construct_mesh bottom,
chamber-area and fake-path/conflict consumers where Type1 supplies their data.
Destination: existing `wipe_tower::{data,extents,fake,geometry}`,
`orchestration`, `footprint::{first_layer,islands}`, `brim` and placeholder
binding seams. Replace absent Type1 producer data, not downstream formatting.

Included: actual generated bbox, rib offset, rotation/plate translation,
brim/cone corners and convex hull; retain model/support/skirt/brim contributions.
Type2's absent outer_wall must not be generalized to Type1. Coordinate with
other brim/skirt/chamber owners before shared-file changes; do not absorb their
independent algorithms into this milestone.
Deferred: rendering triangle meshes and UI preview only; a live geometry
consumer cannot be deferred merely because it originally used a mesh.
RED: accepted baseline G29 difference plus GUI-moved/rotated prime tower on
stock X1C, unchanged start template. Native must export and slice successfully;
full Native/API/CLI equality, not a patched G29 assertion, is the exit.

### T4 precision audit after consumed checkpoint

The non-final consumed checkpoint is `fcf71a9cc`. Its frozen public witnesses
remain full-stream RED. The retained raw diff shows brim arc I/J/E differences
and G29 X `9.75224` versus `9.75223`; neither is assigned to W4.

`WipeTower.cpp:3171::finish_layer_new` calls `offset(outer_wall,
scaled(spacing))`. `Point.hpp:664–669` selects integer `coord_t` output for
`scaled(float)`: divide in float precision, then truncate to the integer
coordinate. Only then does the offset API convert the delta to float.
`type1/finish_layer.rs` must preserve that conversion boundary, rather than
send a fractional scaled float directly into the shared offset implementation.
This is the upstream lattice conversion, not output rounding or a G29 fix.
The generated polygon must continue into both arc fitting and live bounds.
No comparator or frozen fixture changes are permitted to qualify this correction.

The follow-up live bounds audit found identical integer bbox and float rib
offset in the official Native and Ares debuggers. `Print.cpp:3048` constructs
`Point(scale_(pt.x()), scale_(pt.y()))`; the double-argument constructor in
`Point.hpp:197` explicitly applies `std::round`, unlike `Point::new_scale` or
`scaled<float-input>()`'s integer overload. The shared `occupancy::placed_corners`
seam must preserve this constructor selection before convex-hull and placeholder
consumers. Do not change the G29 template, formatter or bbox to compensate.

## T4 outer-wall carrier and Brim exclusion handoff

Coordinator authorizes `Print.cpp:3565–3571,5171–5224` and
`Brim.cpp:651–659` as a separate consumed slice. Preserve every nonempty
print-Z outer-wall row as closed polylines, not just its bounding box. The
first map entry supplies the bbox (the source empty-map branch is a zero-point
box) and Brim exclusions; all rows supply conflict-layer paths and source
bottom-Z calculation. Both consumers use the source float `position +
rib_offset`, then `Point(double,double)` translation. Do not add a rotation
that these upstream consumers do not perform.

Pass tower exclusion polygons explicitly from orchestration through
`BrimPlan::generate_all/generate` and the existing bed-area geometry seam.
Append them to the forbidden areas once those areas are in the plate frame,
before the bed-border mask and area difference, never to already generated
paths or to global state. Preserve the W2 `mm3_per_mm` expression and leave
`motion/skirt_brim.rs` untouched. Keep `brim.rs` below400 LOC.

Pre-implementation Native-first supplemental RED discovery is retained under
`${COORD_ROOT}/evidence/w5-type1-t4-single-brim-ordered-discovery`:
stock enabled input, GUI-reachable outer-only brim20, tower(80,115), cube0
no-brim override. Official and seeded Native/CLI exits are0; complete raw
streams differ. Existing frozen public fixtures/tests remain unchanged.
The broader two-brim exploratory case and the rejected noncontiguous metadata
serialization are retained separately, not used as successful qualification.

## Type1 post-template travel coordinate boundary

The remaining enabled-witness entry travel differs even though the parsed
last template position is `(165,256)` in the debugger. `GCode.cpp:1013` calls
`gcode_to_point` for this nozzle-coordinate position; `GCode.cpp:8138–8145`
subtracts object origin, adds the active nozzle offset, then truncates via
`scaled<coord_t>(Vec2d)`. In contrast, the tower destination and avoidance
bbox corners use `wipe_tower_point_to_object_point` (`GCode.cpp:502–505`),
without nozzle-offset addition and with Point(double,double) construction.
`type1/travel.rs` must retain these distinct coordinate frames before computing
intersections. Do not change the last-position parser, emitted coordinates,
shared travel planner, or fixture nozzle offsets to compensate.

## Scope and qualification gates

1. Approve this docs proposal, then source-cited SPEC for the consumed chain
   with explicit reachable branch inventory. T1→T2→T3→T4 is dependency order;
   partially activated Type1 is never a release or integration boundary.
2. Author public E2E RED before production edits, using existing NativeCase /
   `native_stream_case::check(..., (false, |_, _| {}))`; retain input hashes,
   official then seeded fresh Native caches and Native-success receipts.
   Freeze approved tests after genuine RED. No committed Native outputs,
   expected byte constants, ignores or private-entry substitutes.
3. Qualify GUI variants rather than inventing gaps. Additional Type1 branches
   (different adhesiveness/soluble categories, rib walls, support/raft,
   wrapping, non-BBL Type1 and multi-nozzle) need source inventory and witnesses
   before a complete Type1 claim. X1C discovery alone does not certify them.
   Source-reachable branches may not be hidden by new rejection guards; if a
   dependency prevents the selected chain, STOP and request a scope decision.
4. Implement only after RED approval. Reuse Type2 seams with regression coverage
   and preserve all live downstream consumers. No emission-only or bounds-only
   milestone exit. Any remaining complete-stream difference is unresolved.
5. Exit: stock baseline and qualified variants whole-PASS official and seeded,
   ordered public API/CLI comparisons with raw API==CLI, unchanged Type2
   regressions, fmt, strict workspace/all-target Clippy, browser-WASM check,
   and coordinator-scheduled complete nextest validation on exact final tip.
   CPU11 serial, separate fresh Native caches, official→seeded; retain raw
   streams, input SHA, commands, exit codes and verifiable artifact locations.
   FULL/rebase/push/merge require coordinator authority; none run for this docs RR.

Deferred projects: general multi-transform PrintObject preparation (ARD-0105),
GUI/rendering, standalone Type2 redesign and unrelated placeholder arithmetic.
These are not excuses to suppress reachable Type1 behavior. Existing W5's nine
approved placeholder slices retain their independent frozen integration window.
