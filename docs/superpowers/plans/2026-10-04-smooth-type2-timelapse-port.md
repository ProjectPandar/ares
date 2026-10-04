# Timelapse state / Type2 implementation plan

1. Establish the BBL stock complete-output RED using untouched vendor presets.
   Inspect the upstream owner and repair only the incorrect state transition.
2. Extend the stock regression to the handed-off printer/nozzle families and
   adjacent controls. Obtain original and separate seeded full-output GREEN.
3. Review the historical Type2 source slice against current preparation,
   footprint, layer and tool interfaces. Port source-owned behavior via TDD;
   do not replay historical guards, region-zero assumptions or lint suppressions.
4. Run workspace Nextest, Clippy (zero new warnings), fmt and WASM. All Cargo
   commands use `CARGO_TARGET_DIR=/tmp/ares-pi-tower-target`.
5. Commit/push completed sub-boundaries (including authorized RED checkpoints).
   Only after activated Type2 complete-output GREEN and full gates, print
   `REVIEW-REQUEST <hash> <summary>` and stop for coordinator review. After
   `APPROVED` and explicit release of any main-integration freeze, fetch/rebase
   current main, rerun gates and ff-push main. No force-push.

## Complete-activation sub-boundaries (coordinator decision)

The coordinator selected **complete activated Type2 integration**, not an
isolated writer/geometry publication. The pinned sources contain 2647 lines
in `GCode/WipeTower2.cpp`, 382 in its header, and 1863 in `ToolOrdering.cpp`
(only its owning dependencies are ported; existing Rust ordering is retained).
`Print.cpp` and `GCode.cpp` add planning and integration dependencies. This is
substantially larger than the historical same-tool WIP. The latter's
single-object/single-filament/no-raft/rib rejection block cannot be removed
and its same-tool algorithm then used as a general planner.

All source paths below are under `OrcaSlicer/src/libslic3r`, at the ARD-0105
pin. Destination paths are under `crates/ares-core/src/project_slice`.
Existing preparation/emission remain temporary compatibility shells around
these upstream owners, not a new Ares-owned tower pipeline.

### A. Activation, ordered tools and tower layer partitions

- Source: `Print.hpp::wipe_tower_type`; `Print.cpp:3290` `has_wipe_tower`,
  `3385` `enable_timelapse_print`, `3390` `_make_wipe_tower`;
  `GCode/ToolOrdering.cpp::{insert_wipe_tower_extruder,sort_and_build_data,
  fill_wipe_tower_partitions,collect_extruder_statistics}` (partitions at
  816–919); header `LayerTools`.
- Destination: existing `gcode_emit/tool_ordering/` and `layers/schedule`,
  `raft_schedule`, plus private `wipe_tower/plan` input assembly.
- Include source activation (not merely smooth mode), logical-tool identity,
  selected tower filament, downward partition propagation, tower-layer height,
  empty/support-only layers and insertion of the raft separation-gap layer.
  Do not build a second ordering from the first object's records or tool zero.
- E2E witnesses: enabled/disabled and traditional/smooth controls; two used
  tools, unused configured tools, multiple objects, support-only/raft-gap
  layers. Verify exported settings and actual reference activation.

### B. Purge allocation and layer-depth planning

- Source: `Print.cpp:3572–3690` Type2 branch; `ToolOrdering.cpp`
  `WipingExtrusions::{mark_wiping_extrusions,ensure_perimeters_infills_order}`
  (1620, 1765); `WipeTower2.cpp::{extract_wipe_volumes,plan_toolchange,
  plan_tower,save_on_last_wipe,first_toolchange_to_nonsoluble}` (2178–2363);
  `WipeTower2.hpp::WipeTowerInfo::{ToolChange,toolchanges_depth}`.
- Destination: `wipe_tower/plan/{volumes,layers,depth}`; wiping assignment
  in existing tool-ordering/tool-bucket and ordered-extrusion ownership.
- Include matrix/prime-volume selection, minimal purge, flush multiplier,
  infill/object purge allocation, soluble ordering, ramming depth and last-wipe
  volume savings. Preserve per-region entity ownership; never route through
  `regions[0]` or reinterpret all entities as one material.
- E2E witnesses: unequal tool-to-tool volumes, purge-in-tower on/off,
  purge-into-infill/object, soluble transition and configured tower filament.
  Unit checks supplement, not replace, full-output proof.

### C. Writer, geometry and material parameters

- Source: `WipeTower2.cpp` geometry helpers (37–559), `WipeTowerWriter2`
  (561–1218), constructor / `set_extruder` (1248–1416),
  `generate_{rib_polygon,support_rib_wall,support_cone_wall}` (2462–2647),
  `get_wipe_tower_cone_base`; shared `WipeTower.hpp` result/box data.
- Destination: private `wipe_tower/{writer,geometry,filament}` with focused
  files below 400 LOC; reuse existing geometry/arc primitives where they
  implement the cited operation without changing precision or ordering.
- Include f32/f64 boundaries, volumetric limiting, rotated movement, arcs,
  never-skip markers, rectangle/cone/rib and fillets/gaps, explicit accounting
  and source vector lookup semantics. No blanket-index-zero material access;
  retain an upstream first-element rule only where the source specifies it.
- E2E witnesses: three shapes, fillet on/off, rotation and nonzero offsets,
  brim/auto-brim, thin layers, volumetric limiting and arc fitting.

### D. Priming, per-layer generation and material lifecycle

- Source: `WipeTower2::{prime,tool_change,toolchange_Unload,
  toolchange_Change,toolchange_Load,toolchange_Wipe,finish_layer,generate,
  construct_tcr}`, cpp 1417–2160 / 2364–2447 / 1219;
  header `set_layer`, `FilamentParameters` and used-filament data.
- Destination: `wipe_tower/{generate,priming,toolchange,finish_layer}` and
  subordinate unloading/cooling/ramming modules.
- Include initial and later tower layers, sparse/no-sparse behavior, interface
  features, soluble materials, SEMM and multi-tool loading/ramming/cooling,
  final unload and per-tool statistics. A matching first tower block does not
  satisfy this boundary. Historical unload arithmetic is reusable only after
  comparison against current source state and logical-tool parameters.
- E2E witnesses: priming on/off, SEMM and toolchanger transitions, enabled
  ramming/cooling/parking, sparse layers, interface controls, one-layer final
  purge and tower ending before/at the final object layer.

### E. Occupancy, skirt and placeholders

- Source: `Print::{first_layer_wipe_tower_corners,finalize_first_layer_convex_hull}`,
  `Print.cpp:3020–3162`, `_make_skirt` using tower corners;
  `WipeTower2::{get_bbx,get_rib_offset,get_brim_width,get_z_and_depth_pairs}`.
- Destination: existing `gcode_emit/{footprint,skirt,placeholders}` consuming
  the generated plan, not nominal first-object or rectangle-only guesses.
- Include first-layer bounds with/without a skirt, object-center-without-tower,
  plate placement/rotation and actual brim/rib/cone envelope.
- E2E witnesses: no skirt, combined/per-object skirt, bounds placeholders,
  multiple objects, rotation and placement. Preserve current support/raft
  footprint behavior and object-local versus plate coordinates.

### F. Live writer integration, PA and final accounting

- Source: `GCode.cpp::WipeTowerIntegration::{append_tcr2,
  post_process_wipe_tower_moves,prime,tool_change,finalize}` (1118–1596),
  its layer-export call sites, `Print::_make_wipe_tower` final-purge selection,
  and current upstream pressure-advance/statistics owners.
- Destination: existing `gcode_emit/{layers,tool_transition,finish,
  pressure_advance,adaptive_pa}`, private `wipe_tower/emit` and `motion/tower`.
- Include source ordering of startup/skirt/tool/object actions, template and
  tool-change substitution, offsets/never-skip handling, source travel/wipe
  endpoint handoff, live Z/retraction state, final purge after cooling and
  tower PA resets. Preserve per-tool extrusion ledgers and generation-time
  PA reset events; do not parse arbitrary user PA commands to infer resets.
- E2E witnesses: complete output including repeated commands/comments/M73,
  mixed-tool lifecycle, nonzero offsets, PA on/off/models, final purge,
  per-tool filament totals and footer. Rerun the 29 BBL/PA regression cases.

### G. Integrated gate and delivery

Dependency order: A → B; C supplies D; B + C → D; D supplies E and F;
A–F together → G. Arithmetic reuse is not permission to copy old adapters.
Several implementation commits are allowed, but unconnected writer/geometry
is neither an E2E result nor a reviewable completed slice. No added public API
or dead-code/lint suppression is permitted just to publish an intermediate.

Before implementation, enumerate fixtures against A–F. For each case retain
exported settings, model identity, reference identity, artifacts and the
observed source branch. Capture RED then GREEN using the original full-output
comparator, and independently repeat against seeded Orca. If a fixture exposes
an activated dependency, complete its owning source slice; do not switch it
off, ignore it or add an unsupported-feature guard. Type1 runtime/preview mesh
and UI are deferred; shared Type2 dependencies and all activated Type2 branches
above are included. Exhaustive printer/options coverage remains unclaimed.

Only after integrated complete-output GREEN: workspace Nextest, Clippy with
zero new warnings, fmt and core/adapter WASM; final review-candidate push;
`REVIEW-REQUEST`, then stop for matching approval. Fetch/rebase, repeat gates
and ff-only integration afterwards. Coordinator additionally authorizes
intermediate feature-branch commit/push checkpoints (including the initial RED)
every one or two sub-boundaries. These are not review requests, main candidates
or claims that the active path is complete. Snapmaker bridge-direction residuals
remain in the later bridge slice, not this tower change.

## Provenance and execution

Adaptive PA was approved at `91cae8cf`; rebase onto `0d6dfe1f` preserved all
three patches (`git range-diff` equality). Workspace 7592 PASS, both live
references PASS, Clippy 249 baseline warnings, fmt and WASM passed again.
Fast-forward integration: `0d6dfe1f -> 56e4da53`.
Evidence: `/tmp/ares-pi-tower-pa-integration-*.log` and
`/tmp/pi-unified-exec-69-55e90c70.log`.

Coordinator's BBL handoff: expected two `G1 Z.1` lines before unretraction,
Ares emitted one after the first timelapse / object travel. The retained path
was `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-AHpvpi`, byte 44919 / line
1090. That directory is no longer available in this worktree environment;
generate a fresh original-reference pair rather than claiming its inspection.

The first tracer uses stock `Bambu Lab P1P 0.2 nozzle` with `cube10.stl`, no
smoke overrides, no geometry rewrites and no disabled timelapse dependency.
## Fresh RED and source diagnosis

Original CPU0 run `b3c18bd2-66ae-4e4c-9324-2bf7799c59e4` confirms the
handoff: all three completed oracle attempts are divergent. The inspected
final pair differs only by one missing `G1 Z.1` (7 bytes):
`/tmp/ares-pi-tower-bbl-z-red-official/case-pQ3q43`; log
`/tmp/ares-pi-tower-bbl-z-red-official.log`, first difference line 966.

An additional full-output fixture enables the upstream `gcode_comments`
option without altering the stock baseline. Its original comments identify
**two distinct source operations**: travel's `move to first perimeter point`
Z line followed by `_extrude`'s `ensure Z matches planned layer height`.
Ares emitted only the latter and dropped the travel XY comment. Evidence:
`/tmp/ares-pi-tower-bbl-z-comments-red.log`, run
`ef3f0fcd-c365-4437-b4d1-fcf661f0893a`, pair `case-ILF3QY`.

The quantization hypothesis is disproven: upstream's post-contour reset uses
`quantize_xyzf` too (`GCode.cpp:6396-6400`). The owning difference is
`GCode::travel_to` passing `m_need_change_layer_lift_z` as `force_z`
(`GCode.cpp:7480-7482`) into `GCodeWriter::travel_to_xyz`: its lifted dead-band
shortcut requires **both** `!force_z` and `!will_move_z` (line 771). The Rust
lift-cancellation predicate tested only `will_move_z`, missing forced travel
after an unclear-position timelapse. Preserve the force flag independently
of the approach-shape flag; let the existing XY/Z split and first-extrusion
synchronization emit their own commands. Do not add a literal duplicate Z.

## GREEN prerequisite slice and gates

The source predicate repair passes both P1P 0.2 cases (stock + source comments):
`/tmp/ares-pi-tower-bbl-z-force-fix.log`, run
`59cf49a5-100b-4bc0-9683-c813e5859e75`, 6.868s. Extend the unchanged-stock test
to H2S, P1P and P1S at 0.2/0.4/0.6/0.8 mm. All 12 presets pass, including
0.4-mm controls, as does the additional comment-enabled P1P 0.2 case.

| Gate | Result | Log |
| --- | --- | --- |
| Original CPU0 | 5 tests / 29 complete-output cases PASS, 33.752s | `/tmp/ares-pi-tower-bbl-z-matrix-official.log` |
| Seeded, seed 0 / CPU0 | 5 tests / 29 complete-output cases PASS, 40.550s | `/tmp/ares-pi-tower-bbl-z-final-seeded.log` |
| Workspace Nextest | 7594 PASS, 9 existing skips, 373.293s | `/tmp/ares-pi-tower-bbl-z-final-workspace.log` |
| Clippy | exit 0, same 249 baseline warnings, no added warnings | `/tmp/ares-pi-tower-bbl-z-final-clippy.log` |
| rustfmt | exit 0 | `/tmp/ares-pi-tower-bbl-z-final-fmt.log` |
| Core + adapter WASM build | exit 0, 16.14s | `/tmp/ares-pi-tower-bbl-z-final-wasm.log` |

The 29 cases consist of 13 BBL cases and 16 Adaptive PA regressions, not 29
new printer presets. Original run: `e239d0a8-bb20-4230-b773-d0cbf1eae4a9`;
seeded run: `9eae1e45-62ef-4927-883e-4cde2651bf99`. Separate artifact roots:
`/tmp/ares-pi-tower-bbl-z-matrix-official/` and
`/tmp/ares-pi-tower-bbl-z-final-seeded/`. Gate chain session 78 exits 0.
Only documentation changes after these code checks to record their results.

## Approved prerequisite integration

Coordinator approved `b1a5a84d40b3bdea5d4f4442fbd03698633b9b3c` after checking
`GCodeWriter.cpp:771`. Fetch/rebase found main still at `56e4da53`; repeated
all gates: workspace 7594 PASS / 9 existing skips (267.783s), original 29
cases PASS (31.626s), seeded 29 cases PASS (42.046s), Clippy 249 unchanged
warnings, fmt and WASM PASS. Logs: `/tmp/ares-pi-tower-bbl-z-integration-*.log`.
FF push `56e4da53 -> b1a5a84d` succeeded; evidence
`/tmp/pi-unified-exec-86-f065366e.log`. Type2 work resumes only after this
approved integration.

## Type2 complete-output RED

The public fixture `orca_parity_smooth_type2_tower_complete_output` exports
Creality Ender-3 0.4 with an explicitly verified Type2 rib/filleted tower at
(15, 150), enabled prime tower and traditional/smooth modes. It compares
complete output, including M73, without disabling tower dependencies.
Original CPU0 run `5a6af1f7-5bde-4d23-b18d-7024d25bd929`: traditional control
PASS; smooth RED, expected 127844 bytes vs actual 100712 bytes, first
difference line 18 (`M73 P0 R10` vs `M73 P0 R8`). The reference also proves
the prime-tower role is present only in smooth mode. Evidence:
`/tmp/ares-pi-tower-type2-red-official.log`, reference pair `case-LjY9Xj`.
Production Type2 behavior remains unimplemented at this RED checkpoint;
this is not a green slice or review request.

## Authorized branch checkpoints and A progress

Coordinator approved A–G and requested the plan/RED checkpoint followed by
incremental branch pushes. `dc7a848f` contains that initial checkpoint and is
published on `pi/smooth-type2-timelapse-tower`; no main push or review request.
Latest coordinator main notice: `931691eb` adds classic extra overhang perimeters
and moves skirt's convex hull to `geometry/convex_hull`; preserve both on the
approved integration rebase, along with `16ac907b` support-interface fan request
lifecycle in `gcode_emit/cooling/role_fans` and `motion/fan`.
`e865eb2a` changes Grid paths to `Oriented`
while keeping their collections sortable. Preserve non-reversal and sortable
semantics, `b64b1d90` behavior-preserving geometry lint refactors, and
`7cfa0b33` (zero sparse-anchoring reference center and removal of
`fill_entities::object_center`) when rebasing before integration.
The coordinator explicitly released the main freeze. Type2 remains unapproved:
only after final approval may it rebase, repeat gates and ff-integrate. If a
concurrent push wins, rebase and repeat validation; never force-push.

A's partition kernel is now connected to the existing `ToolOrdering::build`:
partition counts propagate downward, tower flags retain wrapping/smooth rules,
raft gaps insert or reuse a midpoint with the following tool, support-only
changes retain their tower, and selected tower filament insertion reruns
reordering/partition planning. `calc_max_layer_height` is shared with the
existing skirt scheduling consumer. The upstream per-object bottom-Z update
is retained, not replaced with an independently designed minimum-Z rule.

Six supplementary source-example unit tests were first run against missing
behavior: 6 FAIL (`/tmp/ares-pi-tower-type2-partitions-red.log`), then 6 PASS
(`/tmp/ares-pi-tower-type2-partitions-green.log`). Both live references retain
29 passing BBL/PA cases and the traditional Type2 control; smooth still fails
at the original M73 mismatch. Each aggregate is **5 tests PASS / 1 FAIL**, not
green: `/tmp/ares-pi-tower-type2-partitions-{official,seeded}.log`, 26.115s and
42.965s. These live runs precede an equivalent nesting-only Clippy adjustment
inside the already non-tower sparse-layer loop; final checks are recorded in
`/tmp/ares-pi-tower-type2-partitions-{clippy,green}-final.log`.

The subsequent A implementation connects source activation, priming statistics
and first-tool selection, and materializes tower-only gap layers through the
existing support/emission schedule. `GCodeWriter.cpp:63–79::set_extruders`
requires sorting the writer's own tool list independently of priming order.
The new activated fixture exposed an out-of-bounds registry access before this
sort was ported; its regression verifies sorted IDs and inactive-tool access.
The combined ordering/activation/registry selection passes 19 tests; Clippy
retains 249 warnings. Logs: `/tmp/ares-pi-tower-type2-A-final-{unit,clippy}.log`.

The raft fixture uses a valid 0.1-mm contact gap and verifies the reference's
`Z=0.7` gap-layer marker. The earlier 0.5-mm input was rejected by Orca and is
not parity evidence. Multi-tool witnesses use smooth mode: upstream
`DynamicPrintConfig::normalize_fdm_2` counts model-used filaments before regional
wall overrides and otherwise disables the tower for this single-volume model.
IDEX placement uses X=70, inside the shared printable area; X=15 was rejected
by the reference. SEMM fixtures explicitly set the source-default
`filament_is_support` for both materials, as CLI flush-matrix construction
requires its vector to match the color vector.

The original-reference regression run retains all 29 BBL/PA cases and the
traditional control. Activated smooth and raft outputs remain RED at M73;
B–G are pending. The initial multi-tool run additionally exposed a reference
preset-vector rejection, so it does not establish all multi-tool witnesses.
Evidence: `/tmp/ares-pi-tower-type2-A-final-official.log`. After correcting
the fixture vector, all three official multi-tool references activate the
tower, verify priming on/off, and reach full-output RED (not reference errors):
`/tmp/ares-pi-tower-type2-A-tools-supported.log`. Expected/actual byte counts:
IDEX 809473/162228; SEMM 895321/123617; SEMM priming 902374/123887.
The separate seeded run reproduces all five activated divergences and keeps
the 29 regressions plus traditional control passing:
`/tmp/ares-pi-tower-type2-A-final-seeded.log`. fmt/diff checks pass.
No Type2 review request or integration is authorized by this checkpoint.
The A checkpoint is published as `b308fa2b`.

## B: Purge witnesses and collection ownership

The asymmetric SEMM witness retains matrix `[0, 200, 110, 0]` and multiplier
0.75 in exported bytes. Its paired infill-purge variant additionally verifies
`flush_into_infill=1` and the actual reference `; PURGING FINISHED` marker
(`GCode.cpp:5328`), not just a requested flag. Both reach full-output RED:
748725/123618 and 752940/123618 expected/actual bytes respectively. Evidence:
`/tmp/ares-pi-tower-type2-B-purge-red.log`, run
`c7ffcbac-aceb-4766-ad35-d63ae4a8fbd0`.

Source inspection confirms that `WipingExtrusions` assigns complete original
collections, whereas the existing Rust island stage flattens sortable fill
collections. Preserve layer-local collection IDs through this flattening,
ordering and tool partitioning before allocating purge volume. Existing
perimeter `source_order` already provides its corresponding identity;
region IDs are not replacements for collection IDs. The ownership decision
is recorded in ARD-0105. Volume allocation, depth simulation and generation
remain pending; this ownership work is not completion of B or Type2.

Ownership validation: 9 focused tests PASS (31.290s), including shared identity
for flattened siblings and preservation through multi-region tool buckets.
Clippy remains at 249 warnings; fmt and diff checks PASS. Original and seeded
runs each retain the 29 BBL/PA cases and traditional control; all 7 activated
Type2 cases remain full-output RED. Logs:
`/tmp/ares-pi-tower-type2-B-identity-{unit,clippy-final,official,seeded}.log`.
The five multi-tool Ares outputs are byte-identical before and after adding
identity metadata (`/tmp/pi-unified-exec-178-37569fa2.log`); this is regression
evidence, not parity GREEN. No full-workspace/WASM gate or review request is
claimed for this partial B checkpoint. It is published as `f31df169`.

## Continuing B–G execution evidence

See [the Type2 progress record](2026-10-04-smooth-type2-timelapse-progress.md)
for private kernel checks, copy ownership, official/seeded RED witnesses, and
remaining integration obligations. This is not a completed integration boundary.
