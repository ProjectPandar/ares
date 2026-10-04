# Type2 execution evidence (WIP)

Current status: production activation is connected since `1985672b`; the earlier
RED/disconnected entries below are historical checkpoints, not the current tree.
Final completion still requires the remaining source-consumer audit, witnesses,
and removal of introduced lint diagnostics. Main integration is frozen until
explicit coordinator release; branch checkpoints are not final approval.

Continuation of [the complete-activation plan](2026-10-04-smooth-type2-timelapse-port.md).
The source boundary remains pinned libslic3r `WipeTower2.cpp`,
`ToolOrdering.cpp::WipingExtrusions`, `GCode.cpp::WipeTowerIntegration`, and
`PrintApply.cpp::print_objects_from_model_object`, with private Rust destinations
in `project_slice::gcode_emit` and `project::effective_config`.

## C/D work in progress (not a completed integration boundary)

Private `wipe_tower/` code now contains source-owned material projection,
writer, rectangle/cone/rib walls, rounding, depth planning, the five last-wipe
simulations, per-layer finish, priming, unloading/loading/ramming/cooling and
wiping. These were saved in checkpoint `3519d10d` but remain **not connected to production emission**.
At that checkpoint B's whole-collection purge assignment and E–G were pending.
The production-dispatch continuation below does not change the full-output RED
status. Do not publish these modules as a completed
Type2 implementation or infer parity from their unit tests.

Precision regression: `scaled(Vec2f)` divides in float precision, whereas
`Point::new_scale` uses the double `scale_` macro (`Point.hpp:204–206,647–704`).
The new 0.7-mm test first failed with 699999 instead of 700000, then passed with
the separate source conversions. Evidence:
`/tmp/ares-pi-tower-type2-C-precision-{red,green}.log` (4/5 then 5/5 PASS).
Depth/activation/writer/geometry checks: 8 PASS in
`/tmp/ares-pi-tower-type2-C-plan-tests.log`. The additional generation test
covers all three shapes and two-tool transitions; it passed in
`/tmp/ares-pi-tower-type2-D-generator-tests.log`. These are supplementary kernel
checks, not the official/seeded comparisons or full gates.

Source reachability matters: `WipeTowerWriter2` comments out its arc-setting
initializer (cpp 578), leaving its private flag false even when the print
setting requests arcs. Its reachable linear path is ported; do not silently
activate the dormant arc branch. The wall gap caller supplies an empty skip
vector (cpp 2526–2528); linear path discontinuities still retract and travel.

Open source defect for final review: the last priming tool reads
`tools[idx_tool-1]` even for a singleton (cpp 1480). Rust must not reproduce the
out-of-bounds read. Current WIP uses the same-tool matrix entry in that case;
this needs explicit single-used-tool/unused-configured-tool reference evidence
before the final parity claim. No guard rejects single-tool activation.
Combined B/C/D kernel checks now pass 22 tests (including partial-axis SEMM reset):
`/tmp/ares-pi-tower-type2-BCD-placement-tests.log`. Clippy completed with 354
warnings (`...-placement-clippy.log`), failing zero-new-warning (baseline 249).
No suppressions were added. Consumers must be connected and warnings resolved
before review. These checks do not establish complete-output parity.

B's original collection-volume snapshot is now captured before island
flattening and `path_simplification::apply`, following `Print.cpp:2425,2571`.
It retains original 3D polyline length, not fitted-move lengths, and thin-fill
IDs follow the ordinary fill volumes. Capture is limited to objects requesting
infill/object flushing. Assignment and production consumers remain pending;
`/tmp/ares-pi-tower-type2-B-raw-volume-check.log` records the compile check.
The strict LargeBed volume fixture was corrected to `100.00000000000001`,
confirmed by a native C++ scalar probe; no tolerance was introduced.
A private source `WipingExtrusions` allocator now has four passing checks for
whole collections/per-copy overrides, dedicated-object priority, support overrides
with double subtraction, and the source boolean-as-tool ordering expression.
`/tmp/ares-pi-tower-type2-B-dedicated-tests.log` is kernel evidence only. Grouped
instance indices now flow into `ResolvedPrintObjectConfig`, following
`PrintApply.cpp:129–167`; actual emission/integration remain unfinished. A two-copy CLI witness is being added.

## Authorized compilable checkpoint

The coordinator explicitly requested saving the current compilable WIP to the
feature branch before continuing production integration. This checkpoint is
not a completed B–G boundary or a REVIEW-REQUEST. Main's current Clippy baseline
is approximately 219 warnings, not the older 249 count; the branch's 354 warnings
must lose all introduced diagnostics before review, without suppressions.
The checkpoint was pushed as `3519d10d03283e71862ea015fa8a4df7a35e5faf`;
workspace `cargo check` and formatting passed before publication.

## Object-island instance dispatch

`layers/instances.rs` now resolves real printable members, caches their integer
shifts once per print object, and supplies tool-bucket/copy dispatch. This ports
`PrintApply.cpp:129–167`, `ShortestPath.cpp:2015–2036`, and `GCode.cpp:5106–5138,5380`.
Tool-specific ordering chains only participating instances, in source object/copy
order, using the configured raw-coordinate tower seed. No millimetre conversion
is used for the integer chaining input. The emitter installs each actual shift
and tracks `(print object, local copy ordinal)` for external-planner handoff.
The consuming island adapter clones additional copies and moves the last one;
single-instance emission does not clone geometry. Original region/collection
identities survive those replays. Purge allocation and tower emission remain
pending; copy-aware brim/labels/external-planner geometry still need end-to-end audit.

Validation for this dispatch slice:
- 18 focused tests PASS: `/tmp/ares-pi-tower-copy-dispatch-final-unit.log`.
- Clippy remains 354 warnings (no increase): `...-final-clippy.log`.
- Official and seeded runs now assert positive Ares extrusion at both contours,
  in addition to the existing native assertions and unchanged full comparator.
  The two-copy Ares stream grows from 121630 to 200157 bytes, M73 R9 to R14;
  native remains 831091 bytes, R69. Both full-output comparisons remain RED.
  Evidence: `/tmp/ares-pi-tower-copy-dispatch-assert-{official,seeded}.log`.
- All five BBL/PA control test functions pass in each reference run (official:
  `...-final-official.log`; seeded: `...-assert-seeded.log`).
- Core/adapter WASM build, formatting and diff checks PASS:
  `/tmp/ares-pi-tower-copy-dispatch-wasm.log`.
No generator/purge integration completion or REVIEW-REQUEST is implied.

## Auto-lift instance ownership

`Print.cpp:594–614` overhang queries now visit every real member shift rather
than treating a transform-group index as a ModelInstance index. Detected layer
geometry remains owned once per print object; only shifts are stored per copy.
A travel through the second copy's overhang must hit, while a travel through
the gap must not. RED E0560 and GREEN 14 tests are recorded in
`/tmp/ares-pi-tower-copy-auto-lift-{red,green}.log`. Clippy remains 354 warnings.
The existing native overhang-spiral witness passes against official and seeded
binaries (`...-{official,seeded}.log`); this supplemental motion check is not a full Type2 parity
claim. Copy-aware external boundaries, brim and labels remain to be audited.


## Whole-collection purge production dispatch

`Print.cpp:3572–3654` now supplies the real layer allocator and depth-plan input.
Original collection snapshots retain region, role, infill/solid classification
and local collection identity before flattening/simplification. Support volume
is read from original 3D polylines before support simplification. Metadata is
built only for eligible flush targets, in linear collection/region work.
`GCode.cpp:4870–4967,5328–5421` consumers now route whole collections per local
copy ordinal, apply object-scoped support/interface overrides, and emit wiping
before normal passes with the native `; PURGING FINISHED` boundary. Same-tool
assignments are still wiping assignments. The source signed -1 normal sentinel
is retained even when the unsigned lookup returns its maximum value; it must
not become a nonexistent tool bucket that loses object paths.

Evidence for this connected sub-boundary:
- Routing API RED: `/tmp/ares-pi-tower-wiping-routing-red.log`.
- Sentinel RED observed `Some(18446744073709551615)` instead of `None`:
  `/tmp/ares-pi-tower-wiping-sentinel-red.log`; regression now passes.
- 32 focused tests PASS: `/tmp/ares-pi-tower-wiping-routing-all-unit.log`.
  Whole-collection routing also preserves the fixture's original `[2,2,7]`
  order/region ownership and leaves the second copy's nominal ownership intact.
- Official and seeded tests assert equal positive purge-boundary counts in
  both single-copy and two-copy infill cases; both translated Ares contours
  remain asserted. All five BBL/PA control functions PASS per reference.
  Evidence: `/tmp/ares-pi-tower-wiping-routing-final-official.log` and
  `/tmp/ares-pi-tower-wiping-routing-seeded.log`.
- Full comparisons remain RED: infill 752940/126156 bytes, two-copy
  831091/203384 bytes (reference/Ares). The comparator is unchanged. Generator
  results, placement, labels/brim/support-copy lifecycle audit and E–G remain
  unfinished; retaining a production plan does not emit a tower.
- Clippy completes with 341 warnings, down from 354; four new nesting
  diagnostics were removed, without suppression. This still fails the final
  zero-introduced-warning gate against main's approximately 219 baseline.
  `/tmp/ares-pi-tower-wiping-routing-final-clippy.log`.
- Core/adapter WASM build PASS:
  `/tmp/ares-pi-tower-wiping-routing-final-wasm.log`; fmt/diff checks clean.

No Type2 approval or final review readiness is claimed.

## Copy ownership and full-output witness

- Source-instance members now survive transform grouping into
  `ResolvedPrintObjectConfig::instance_indices`; local copy ordinals are positions
  in that list, not the original ModelInstance indices themselves.
- RED: missing member lookup (E0599), `/tmp/ares-pi-tower-copy-members-red.log`.
  GREEN: 104 related tests, `/tmp/ares-pi-tower-copy-members-green.log`.
- The CLI SEMM/infill fixture now has two translated instances of one object.
  It uses fresh native project/output paths, avoiding stale exported-case cache
  reuse. Native comments label both copies `copy 0`; assertions instead verify
  real extrusion at both cube contours (X90.31 and X120.31, Y114.69).
- Official and seeded references separately pass fixture validation but remain
  full-output RED: 831091 reference bytes versus 121630 Ares bytes; first
  difference at byte465, M73 R69 versus R9. Evidence:
  `/tmp/ares-pi-tower-type2-copies-official-final.log` and
  `/tmp/ares-pi-tower-type2-copies-seeded-final.log`. Earlier cache-reusing and
  copy-label assertion runs are diagnostics, not valid parity evidence.
- Current focused regression run: 126 PASS,
  `/tmp/ares-pi-tower-copy-final-unit.log`; fmt/diff checks clean.
  Clippy completed with 354 warnings,
  `/tmp/ares-pi-tower-copy-final-clippy.log`. The three new grouping nesting
  warnings were removed; the disconnected-kernel warning debt still fails
  the zero-new-warning gate.
- `ares-core` and `ares-wasm` build for `wasm32-unknown-unknown` passed:
  `/tmp/ares-pi-tower-copy-wasm.log`.
- Existing BBL/Adaptive PA controls pass all five test functions separately
  against official and seeded references:
  `/tmp/ares-pi-tower-copy-controls-{official,seeded}.log`.
- No production copy emission, generator integration, final review request or
  main integration is claimed by these data-model and fixture changes.

## Connected generation and emission — validation in progress

- `Print::_make_wipe_tower`, `WipeTowerIntegration::{prime,tool_change,finalize}`
  and `append_tcr2` now drive production generation and live output. Results
  enter the layer schedule before skirt/brim construction; priming generation
  remains unconditional, while its output follows the source setting. Final
  purge follows the cooling flush rather than re-entering layer fan filtering.
- The integration retains the writer/tool/template/PA lifecycle, transformed
  endpoints and wipe paths, interface temperatures, first-layer occupancy,
  priming collision pause and tower tool-change statistics. Tower travel shares
  the existing motion writer without ordinary extrusion Z restatement.
- `GCode.cpp:5348–5372,5478–5494` labels empty wiping-pass instances too. The
  full infill/copy differences identified missing start/stop pairs; those
  labels are now retained rather than removing them from the comparison.
- Raft skirt occupancy uses the materialized `RaftSchedule` paths. Synthetic
  tower support layers must not hide raft footprint reconstruction. Returning
  from a tower restates native WIDTH/HEIGHT; lifted travel retains feedrate for
  the downstream cooling parser to deduplicate.
- Broad emission tests: 289 PASS in
  `/tmp/ares-pi-tower-emission-all-unit.log`. Clippy completed with 297 warnings
  and core/adapter WASM built successfully in
  `/tmp/ares-pi-tower-emission-{clippy,wasm}.log`. These precede the subsequent
  numerical/lift regressions and are not final gates for the current tree.
- Official and seeded full comparisons each report 6/7 functions PASS in
  `/tmp/ares-pi-tower-emission-corrections-{official,seeded}.log`. This includes
  the complete-output single-tool Type2 activation/raft matrix and all five
  BBL/PA controls. The tools/priming function still fails: symmetric SEMM and
  priming have matching complete lengths (895321 and 902374 bytes), while
  asymmetric SEMM differs in two retraction numbers. IDEX geometry/lift and
  infill/copy label differences also remain. No full Type2 GREEN claim.
- Numerical RED: `Point(double,double)` rounds in
  `wipe_tower_point_to_object_point`, unlike `Point::new_scale` truncation.
  `/tmp/ares-pi-tower-tower-point-rounding-red.log` observes 998456 instead of
  998457; the corrected Normal/LargeBed regression passes in
  `/tmp/ares-pi-tower-rib-rounding-red.log`.
- Numerical RED: `libslic3r.h` includes `<math.h>`, exposing `sin(float)` in
  the global namespace. `rounding_polygon` divides in float before assigning
  double. The C++/Eigen/native-Clipper probe verifies that overload with a
  static assertion; the Rust regression observes X34047251 instead of
  X34047252 in `/tmp/ares-pi-tower-rib-rounding-red.log`. After correction, all
  336 isolated rib-rounding points match the native scalar probe. Fresh Rust
  and complete-output gates are still required.
- Bridge direction remains assigned to pi-tip. This slice does not modify
  `BridgeDetector` or `detect_bridging_direction`. No final review request,
  approval, rebase or main integration is claimed.

### Existing activation corpus GREEN

- After the point-constructor and float-sine fixes, the default tower retract
  uses the native fixed Normal lift, while `change_layer` preserves its
  forced Spiral lift for Auto. The preceding lift RED observes Slope rather
  than Normal in `/tmp/ares-pi-tower-tower-lift-mode-red.log`.
- 302 emission/raft tests PASS:
  `/tmp/ares-pi-tower-numeric-lift-green.log`. Clippy completes with 296 warnings
  in `/tmp/ares-pi-tower-numeric-lift-clippy.log`; the introduced-warning debt
  still fails final review readiness. Core/adapter WASM PASS:
  `/tmp/ares-pi-tower-numeric-lift-wasm.log`.
- Official **7/7 PASS**:
  `/tmp/ares-pi-tower-numeric-lift-official.log`, separate artifacts under
  `/tmp/ares-pi-tower-numeric-lift-official/`.
- Seeded **7/7 PASS**:
  `/tmp/ares-pi-tower-numeric-lift-seeded.log`, separate artifacts under
  `/tmp/ares-pi-tower-numeric-lift-seeded/`.
- The unchanged complete-output comparisons cover disabled/smooth/raft
  activation, IDEX, SEMM, asymmetric purge, infill allocation, two positively
  extruding copies and front-edge priming, alongside all five BBL/PA control
  functions. This is the first GREEN live Type2 emission checkpoint, not the
  final A–G completion: extend the remaining shape/occupancy/option witnesses,
  complete the source-consumer audit, remove introduced diagnostics and run
  the full workspace gates before requesting integrated review.

### Shape/sparse and convex-hull placeholder continuation

- The emission checkpoint workspace passed **7636 tests, 9 skipped** in
  `/tmp/ares-pi-tower-emission-checkpoint-workspace.log`; this predates the
  additional variant and placeholder tests below.
- Ten shape/sparse/option cases first passed separately against official and
  seeded references in `/tmp/ares-pi-tower-variants-red-official.log` and
  `/tmp/ares-pi-tower-variants-seeded.log`: rectangle, cone, sharp rib,
  automatic tower brim, thin layers, arc option, no-sparse layers, zero skirt
  loops, rotated/offset tower, and singleton SEMM front-edge priming. The last
  case supplies the previously missing native singleton-priming witness.
- Adding bounds with/without skirt and a nearby explicit brim exposed missing
  `first_layer_print_convex_hull` in
  `/tmp/ares-pi-tower-occupancy-red-official.log`. The nearby-brim case already
  passed; it is not evidence that the fake-tower brim mask has been ported.
- `Print.cpp::finalize_first_layer_convex_hull` (3154–3164) supplies the new
  hull boundary in `footprint/first_layer.rs`; cached brim/skirt outlines take
  precedence over object contours before adding tower corners. The placeholder
  follows `GCode.cpp:2925–2938`, `ConfigOptionPoints`, and `Point.hpp::to_string`.
  A second RED exposed missing `PlaceholderParser.cpp:1687,2333::vector_size`
  in `/tmp/ares-pi-tower-hull-official.log`. `size` now requires an unindexed
  vector variable and returns an integer; numeric functions were moved to
  `expression/functions.rs` to keep the parser under 400 lines.
- All 16/18 hull vertices, not merely their bounding boxes or first vertices,
  are now exposed by the two bounds fixtures. The unchanged full comparator
  passes **8/8 functions per reference** in
  `/tmp/ares-pi-tower-hull-full-{official,seeded}.log`, with separate artifact
  roots. The new function contains 13 variants; the other seven functions
  retain the original Type2 and BBL/PA coverage.
- An additional scalar RED shows point formatting must use native ostream
  general precision: `[-0.000001, 1]` versus `[-1e-06, 1e+06]` in
  `/tmp/ares-pi-tower-hull-format-red.log`. The correction reuses the existing
  `motion::format::ostream`.
- Current checkpoint gates: **7639/7639 workspace PASS, 9 skipped** in
  `/tmp/ares-pi-tower-hull-workspace.log`; official and seeded each **8/8 PASS**
  in `/tmp/ares-pi-tower-hull-final-{official,seeded}.log`; core/adapter WASM
  PASS in `/tmp/ares-pi-tower-hull-final-wasm.log`. Formatting and diff checks
  pass. Clippy completes with the unchanged **296 warnings** in
  `/tmp/ares-pi-tower-hull-final-clippy.log`; this remains above baseline and
  does not satisfy the final zero-introduced-diagnostic gate.
- Still pending: first-layer fallback support/raft geometry and actual
  print-object membership, the source first-instance/brim semantics of the
  object-only center, fake-tower brim exclusion and mesh-bottom consumers,
  remaining A–G witnesses, and introduced lint diagnostics. Existing bounds
  plumbing is still a temporary compatibility shell, not completed geometry
  parity. No final review request or main integration is authorized here.

### Object-only center and independent copy fixtures

- Checkpoint `513a6fd2` published the hull continuation above. The next
  activated-tower center slice follows `PrintObject::get_first_layer_bbox`
  (`Print.cpp:2126–2148`), `PrintObject::has_brim` (`Print.hpp:362`),
  `Brim.cpp:908–934`, and `GCode.cpp:2999–3006` in
  `footprint/first_layer.rs::first_layer_object_center`.
- Preserve the source distinction: without brim, the center uses the first
  actual PrintInstance of each print object. Auto brim qualifies even with
  width zero and its object box includes all copies. Thus the two-copy
  fixture returns X95 for explicit no-brim but X110 for auto brim. Cached
  instance shifts retain original membership; both copies still positively
  extrude and their full output remains compared.
- Adding the second copy variant exposed shared filenames/output directories
  in the test helper. These now retain each exported case's unique path so
  the no-brim case cannot reuse the auto-brim native result. The earlier
  `/tmp/ares-pi-tower-center-{red,no-brim-red}-official.log` failures were
  fixture assumptions/isolation errors, not production regression evidence.
- The valid isolated RED is
  `/tmp/ares-pi-tower-center-isolated-red-official.log`: native
  `; COPY_CENTER 95,110`, Ares `; COPY_CENTER 110,110`. After correction,
  official and seeded full corpora each pass **8/8**, including both copy
  variants, in `/tmp/ares-pi-tower-center-green-{official,seeded}.log`.
- **323 emission/raft tests PASS**, WASM build, formatting and diff checks
  PASS: `/tmp/ares-pi-tower-center-{unit,wasm}.log`. Clippy remains **296
  warnings** (`...-center-clippy.log`). The 7639-test workspace result belongs
  to `513a6fd2`; the center continuation has focused gates, not a new workspace
  result. Support/raft footprint fallback, support contribution to brim boxes,
  fake-tower brim/mesh consumers and lint debt still prevent final completion.
