# Draft SPEC: AutoBrim consumes the post-elephant-foot bottom outline

Status: **NATIVE-FIRST DISCOVERY / SPEC-REVIEW-REQUEST**, no code authorized.
Branch/worktree: `worker2/brim-next-native-discovery` /
`~/ares.worker2-brim-next-native-discovery`.
Docs base: merged `e81afae89a4874af3c5556a592c3d5a1d121055c`.
This ports a `libslic3r::Brim` outline consumer, not filling, ironing, seam,
scarf, Arachne, Voronoi, or a new Ares pipeline feature.
Queued exact-group SPEC/RED/GREEN `55b38f744→9774c0c1a→fb70752b9` is untouched.

## Observed real setting combination

Take the already-merged single-object tall-cube AutoBrim input, retain its mesh,
placement, global speeds and material. Enable elephant-foot compensation0.3 mm
and `brim_use_efc_outline=1`. Native moves the AutoBrim footprint inward with
the compensated bottom outline; current Ares continues using its raw outline.
Disabling either the amount or the brim-outline flag yields matching Native
and Ares output. This extends the same real configuration option already
supported by explicit brim types, rather than inventing a geometry mode.

The source input is
`tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf`, SHA256
`e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5`,
a retained exported-project cube (10×10×2 mesh, Zscale50 gives height100).
One object, one model volume, one instance, one PLA/extruder1, ByLayer,
combine_brims0, gap0, AutoBrim configured width10, no supports/raft/tower/skirt.
No geometry, role speeds, object keys, material settings or scene membership
change between cases. Four combinations alter only the2 settings and explicitly
select `elefant_foot_compensation_layers=1`.

## Native-first evidence before implementation

Final root:
`/home/indexyz/ares-coord/evidence/worker2-auto-efc-discovery-20261020T204057/`.

All8 Native invocations (4×official/seeded) and4 existing CLI invocations exit0,
serial CPU13. Both Native modes have identical Brim bodies/hashes. Full-output
discovery comparison replaces only the generated-by line, not M73/motion or
geometry. API/nextest was not run and no public test/comparator was edited.

| EFC amount / brim-outline flag | Input SHA256 | Native / Ares Brim XY extrusion lines | Full output, both modes |
|---|---|---|---|
| 0 /0 | `1faebd4419d3310ad0d43c73a10bc7d85cda391b5c8b2faa128ec7e250833fd0` | 448 /448 | equal |
| 0 /1 | `895dfab6a6438a91d475b1fb2fd8cecfbc64ac5dc672dad9b0bd88d12b2f07a0` | 448 /448 | equal |
| 0.3 /0 | `0e4066dedce3940dde14653f8c081b6c0cdd771b6f3b8e5f2ee851e12691ed61` | 448 /448 | equal |
| 0.3 /1 | `bfe65cf39ea2c485c016aa6563e788679826ff17ae4fdfa68cd69b3d17617b93` | 453 /448 | **different** |

First normalized byte difference1321 is a real Brim-entry XY travel:
Native `G1 X107.108 Y107.365` vs Ares `G1 X106.808 Y107.065`.
The0.3 mm inward shift is observable before the Brim block; first extruded
coordinates also differ (Native107.578/106.925 vs Ares107.278/106.625).
It is not an M73-only discrepancy. Ares target Brim-body hash equals its
amount0.3/flag0 control, while Native target differs from that control; thus
changing the flag has actual Native effect but is ignored for Ares AutoBrim.
One nonempty Brim TYPE block remains in every case. Counts are not loop counts.

Evidence includes the exact input manifest/archives, Native logs/result/G-code,
CLI output/errors/exits, `generate-inputs.js`, `run.sh`, `summarize.js`,
`summary.json`, `verification.txt`, source/binary hashes and pre-doc clean status.
The generator copies the merged input's entries and changes settings only;
original mesh entry bytes are verified identical across all4 inputs. A later
RED must freeze input-only archives and their referenced hashes, not probes or
Native output. Scripts stay outside Git; no builds/FULL occurred in discovery.

### Read-only CLI provenance and prerequisite

The shared existing CLI build target currently contains the queued exact-group
candidate `fb70752b954851de99a7629312846dae045cccc7`, binary SHA256
`856dc6cf4b15b60b0444f96e97213d1e52109c3bcd092f1ba517bb5e00a3ab65`.
This is **not** falsely labeled as an exact merged-main CLI. It is used read-only;
no queued source, branch, tracked file or build target was modified here.
The selected inputs have a single group, whereas that queued change repairs
assignment across groups;3 inactive controls match completely. Still, before
committing RED reconfirm on the then-current merged main or explicitly stack
on the approved exact-group dependency. Source evidence below also shows the
same AutoBrim EFC exclusion on merged `e81afae89`. No claim of current-main
API RED is made from this exploratory binary observation alone.

## Upstream owner and source boundary

Read-only upstream checkout commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`; files under
`OrcaSlicer/src/libslic3r`:

1. `Brim.cpp:56-62::use_brim_efc_outline` is **brim-type agnostic**: flag set,
   compensation positive, compensation_layers positive, raft_layers0. It does
   not exclude btAutoBrim. These are existing config conditions, not a new policy.
2. `Brim.cpp:112-118::get_print_object_bottom_layer_expolygons` builds the
   post-EFC bottom footprint from first-layer region surfaces with the cited
   SCALED_EPSILON closing. This is an existing consumer/source, not a rewrite
   of EFC geometry itself.
3. `Brim.cpp:468-472` computes/selects the object's adjusted bottom outline.
   `:477-501` retains original volume groups for mesh-height/moment Auto width
   calculation. Do not feed compensated geometry into the raw width formula.
4. `Brim.cpp:504-510` intersects the selected object's bottom outline with each
   group's original slices when EFC outline is active. `:511-527` consumes those
   clipped polygons for per-polygon small-area bump, clamp and flow quantization;
   `:532+` offsets those polygons into actual brim areas. Scope is this consumer
   swap/intersection, not a different compensation or grouping algorithm.
5. `PrintObjectSlice.cpp` owns production of compensated bottom-region slices;
   it remains unchanged. This task does not alter main slices, model extrusion,
   perimeters, fill, toolhead routing or the brim group assignment rewrite.

Current Rust under `crates/ares-core/src/project_slice/gcode_emit`:

- `brim/outline.rs::efc_enabled` deliberately restricts accepted brim types to
  explicit outer/inner/both/painted. Its comment leaves AutoBrim/btEar deferred.
  The approved active Native witness now supplies AutoBrim admission evidence.
- `brim.rs::BrimPlan::generate` already obtains `efc_bottom` and calls
  `outline::select`, but AutoBrim ignores that selected `source` and iterates
  `volume_groups` via `brim/auto.rs::auto_group_area` on original `group.slices`.
  Merely admitting AutoBrim in the gate cannot reproduce the required polygon
  consumer because the auto branch does not use the selected per-group outline.
- `brim/auto.rs::auto_group_area`: keep raw group bbox and `width_mm` operands;
  apply source-cited bottom/group intersection to the polygons iterated by
  `polygon_width_mm`/outer-area generation only. The existing BrimPlan/context
  is a compatibility shell for upstream objects, not a separate Ares pipeline.

## Non-negotiable decisions / included and deferred

Included: AutoBrim-specific admission of the existing EFC outline gate and the
per-group intersection/consumer semantics of Brim.cpp504-527. Reuse existing
`outline::bottom_regions`/`select` and geometry intersection infrastructure;
no duplicate post-EFC producer or generic new policy abstraction.
Raw moment-based Auto width, volume-group height, object maximum speed,
material reference and adhesion calculations must not change. Keep pre-EFC
width-vs-post-EFC-per-polygon operands distinct exactly as upstream does.

Deferred: BrimEars/NoBrim gate expansion, painted-ear behavior, support/raft/tower,
object-wide material eligibility, combined-brim ownership, new grouping or EFC
geometry algorithms, and all other workers' fill/ironing/seam/Arachne/Voronoi
boundaries. Do not add Unsupported guards, fallback to raw slices on error,
magic coordinate offsets or a flag beyond the actual upstream setting.

No production edits before approval/public RED. GREEN only production code,
RED/helper/fixtures/docs frozen. Every file<=400LOC and core remains platform
neutral (WASM/Windows/macOS/Linux); no I/O/UI behavior in core. Stop on a first
strict residual outside this consumer boundary; do not weaken or normalize it.

## Proposed post-approval SPEC → RED → GREEN

1. Rebase docs to the required dependency/current main; reconfirm all4 Native
   observations and exact CLI source hash. Resolve the binary-provenance caveat
   above, not by assuming single-group equivalence constitutes final evidence.
2. Commit public4case input-only RED reusing merged `runner/native_api_cli.rs`.
   Native must produce active Brim XY moves before API/realCLI comparisons.
   Require strict complete ordered parity (only existing generator metadata
   treatment) and API/CLI raw equality; 1active target FAIL +3controls PASS in
   both modes. Assert flag-dependent Native Brim geometry rather than comments
   alone. No M73 mask or rejected-Native-as-success.
3. GREEN implements only cited gate/per-group consumption. If test output still
   differs, stop and report first raw residual instead of changing raw Auto width,
   compensation producer or unrelated routing. Retain explicit/painted EFC,
   Auto small-diagonal, exact-group and object-speed controls unchanged.
4. Directed dual-mode evidence, fmt, wasm32 and
   `cargo clippy --workspace --all-targets -- -D warnings`; inherit the W3 lint
   fix during rebase rather than duplicate it. Final exact-tip dual canonical
   FULL only in coordinator-allocated W2 window. No FULL during discovery/queue.
   Merge/push coordinator-only; docs-only RR now.

## CPU/replay provenance

All Native/CLI/analysis commands serial `taskset -c13`; seeded explicit seed0,
`ORCA_REPLAY_CPU=13`, existing seed-only wrapper/binary. Official explicit2.4.2
extracted appdir/wrapper. Only CPU13, no stress or competing workerCPU use.
Original `Brim.cpp` was read from the original AutoBrim worktree's ignored
upstream checkout; this worktree has no local checkout/resources link.
Replay into a new external root: copy3scripts, verify retained source/binary
hashes, run `taskset -c13 node <root>/generate-inputs.js <root>/inputs` and
`taskset -c13 bash <root>/run.sh <root>`; inspect manifest/summary. Only this
SPEC document enters Git; queued exact-group chain is unchanged.
