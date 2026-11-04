# Spec: parity of the curled-perimeter (malformation) slowdown — C1

Status: draft
Provenance: the case was first attributed to CoolingBuffer layer-time
slowdown; that direction was verified formula-exact and ruled out, and
the divergence re-attributed to the curled-perimeter (malformation)
slowdown in this spec. The cooling-buffer spec was superseded and not
kept; its durable conclusions (feedrate table, wipe-speed echo) are
folded in below.

## Verified facts

Case: stock matrix `case-VpfOo2` (Creality Sermoon V1, cube10.stl,
10×10×10 cube whose first layer is inset ~0.3 mm by a bottom chamfer).
Only layer **Z = 0.6** diverges; Z0.4 and Z0.8+ emitted streams are
byte-identical between orca and ares (including F values).

With `slow_down_for_layer_cooling` disabled in a repacked 3mf
(seeded oracle, `ORCA_REPLAY_SEED=0`, single CPU):

| role | orca `G1 F` (Z0.6) | ares `G1 F` (Z0.6) |
|---|---|---|
| inner wall | F5940 ×3 edges, F6060 ×1 | F6477.879, F6060, F5940, F6060 |
| outer wall | F4800, F5580, F5400, F5520 | F6360, F5520, F5400, F5520, F6300 |

So orca slows Z0.6 walls MORE than ares (by ~0.4-0.7 mm of effective
"distance" in the band lookup).

## Upstream anchors (this is a source-cited rewrite slice)

1. `src/libslic3r/Support/SupportSpotsGenerator.cpp`:
   `estimate_malformations` (≈141-196) — per layer, for every
   `erExternalPerimeter` entity: `estimate_points_properties
   <true, true, false, false>` over slice points → `ExtrusionLine`s;
   per segment `middle_distance` = 2D distance to previous layer's
   *external* `ExtrusionLine`s; sign via
   `prev_layer_boundary.distance_from_lines<true>(middle) + 0.5·flow_width`;
   `curled_up_height = estimate_curled_up_height(dist·sign·expansion,
   0.5·(a.curv+b.curv), l->height, flow_width, bottom_line.height, params)`
   → `Layer::curled_lines` when > `curling_tolerance_limit` (0.1).
2. `src/libslic3r/GCode/ExtrusionProcessor.hpp` (~366-420):
   per emitted path: `estimate_points_properties<true,true,true,true>`
   (signed distance to `prev_layer_boundaries[obj]` = previous
   `lslices` 2D lines + `0.5·width` offset), then
   `artificial_distance_to_curled_lines` — max over
   `prev_curled_extrusions[obj].all_lines_in_radius(middle, 10·width)` of
   `width·(1-d/limit)²·(curled_height/(height·10))`, with the len>2mm
   projected-length filter (0.4·len inside the influence box).
3. `calculate_speed` band lookup (`overhang_overlap_levels` =
   90/75/50/25/13/0 with `dynamic_overhang_speeds`) over both
   `curr.distance` and the artificial distance; min wins.
4. Called from `PrintObject.cpp:909`
   (`PrintObject::estimate_curled_extrusions` via
   `SupportSpotsGenerator::estimate_malformations`) and GCode.cpp:6692.
   Ares port: `crates/ares-core/src/project_slice/gcode_emit/motion/
   overhang/{mod,curl,curl/geometry,sections}.rs` — `estimate()`,
   `CurlTracker::{collect,previous_curled}`, `artificial_distance`,
   `curvatures`, `estimate_curled_up_height`, `speed_sections`,
   `speed_for_distance`.

## What is verified equal already

- Ares `collect` + `estimate_curled_up_height` reproduce upstream values
  on the chamfer layer (dist = 0.3, heights 0.025–0.313, corner-max).
- Ares `curvatures` independently re-derived via upstream
  `estimate_points_properties` walk → identical per-point values
  (mid-top ≈ 0.033).
- Ares `artificial_distance` computes formula-correct values on the
  CurledLines it has (in_range, projected-length filter present).
- The `previous_curled`/`collect` roll semantics are lazy but
  order-equivalent to upstream's per-layer set.

## The open delta (what the implementation must resolve)

Orca's per-segment effective slowdown distance on Z0.6 is roughly
0.078–0.15 mm where ares computes 0.0125–0.095. For orca's inner-wall
top edge (midpoint (0,4.056) in centered coords) this needs a curled
line within ~0.55 mm, i.e. **a mid-top-edge L1 segment with
curled_height > 0.1** — ares assigns it h = 0.025 and excludes it.
Sub-hypotheses to test, in order:

1. `middle_distance` for mid-edge segments: ares reports exactly
   0.3000 for every segment — orca may measure a different value
   (e.g. distance to the L1 *external-line set* vs to the inset L0
   slice boundary; different line contents in `prev_layer_lines`).
2. `boundary_sign`: ares gets sign=+1 for all L1 mid-edge points;
   orca may get -1 for some (different `distance_from_lines<true>`
   semantics on the chamfer boundary) flipping the band inclusion.
3. `flow_width`: upstream uses `get_flow_width(region,
   erExternalPerimeter)` = `frExternalPerimeter` flow width — ares uses
   `path.width` (0.61). If upstream's value differs (e.g. the region
   flow is a different width on the chamfer layer), `low/high` band
   edges `malformation_distance_factors·flow_width` shift and the
   curvature term `b = curling_t·flow_width` changes → different `c`.
4. `curled_distance_expansion` / `params` fields (aggregate-init —
   verify all Params fields actually used by the port: filament_type,
   max_acceleration, raft_layers_count, brim_type, brim_width are
   passed in; the port ignores them — confirm they're unused upstream).
5. The CurledLine set itself: if orca keeps L0's chamfer lines in the
   previous layer's *boundary* for Z0.6 (two-layer lag), its
   `curr.distance` differs — testable by checking whether orca's
   needed distance fits `signed+offset` better than `artificial`.

## Method (cheap, established)

- Repack `input.3mf` with `slow_down_for_layer_cooling = 0` via the
  node zip+zlib rewriter, slice with the seeded oracle → `plate_1.gcode`
  = orca's exact pre-cooling emitted F per segment.
- Slice the same modified 3mf with ares → ares' pre-cooling F per
  segment. Direct per-segment diff, no inference needed.
- Optional instrumentation (as used, reverted): env-gated dumps of
  `curled` inputs (dist/sign/curv/h) in `curl.rs::collect` and of
  `artificial_distance` intermediates in `curl/geometry.rs`.

## Scope

In: the malformation/curl slowdown path named above — align
per-segment distances & curled heights to upstream on this case.
Out: cooling buffer (verified), new options, other presets.

## RED test (E2E, required by project rules)

`orca_parity` harness: slice `case-VpfOo2`'s input.3mf with
`slow_down_for_layer_cooling` forced to 0 through the user-override
path (M854) in both slicers, compare emitted `G1 F` sequence
per segment on Z0.6 — first divergence pinpoints the segment.
Then the original full case (cooling on) must flip to PASS.

## Acceptance

- Stock-matrix `case-VpfOo2` PASS under seeded oracle.
- Full gates: `cargo nextest run --workspace`, clippy clean, fmt,
  wasm32 check, official + seeded v2 controls on submission hash
  (official: 8 known env fails + split_sides flake).

## Further narrowing (same-day analysis)

Two-pass estimate calls observed: `estimate()` is invoked twice per
path; `collect()` inside it appends each layer's external lines to
`CurlTracker.current` on every outer-wall estimate (each layer shows
26 CURL entries = 13 segments × 2). Between the two calls of the same
path, `previous_curled` still returns the correct N−1 set — roll is
guarded by `self.layer != layer_index`. No order corruption found:
ares' inputs are formula-correct.

Verified-equal-now: curvature port reproduces upstream walk values
exactly (mid-top point (0,4.645): 0.033 both implementations);
estimate heights on the chamfer layer (0.025–0.313) match the
formula; boundary signed distance and +0.5·width offset present;
`curr.distance` for Z0.6 inner wall = −0.589 (inside material → no
boundary slowdown) — meaning orca's slowdown on these segments must
come from the artificial/curled distance or from `original_speed`
already differing.

Therefore the residual delta must be ONE of:
- a `CurledLine` set that orca builds bigger than ares' (more
  segments above tolerance 0.1 — likely more mid-edge segments,
  which for ares sit at h=0.025), or
- orca's `path`/`line` entities used by estimate_malformations are
  the sliced-perimeter entities (pre-variable-speed), while ares
  feeds emitted-path points — possibly different segmentation on
  the chamfer layer, or
- orca marks additional roles as `erExternalPerimeter` on the
  overhang layer (`detect_overhang_wall`), adding its inner wall's
  lines to the curled set and to `prev_layer_lines`.

The no-cooling diff (F-values per segment) plus a one-off dump of the
CurledLine sets is enough to discriminate these three.

## Review appendices

- flow_width hypothesis DISPROVEN: upstream `get_flow_width(region,
  erExternalPerimeter)` = `PrintRegion::flow(frExternalPerimeter).width()`
  = `outer_wall_line_width` = 0.61 = ares' `path.width`.
- estimate() being called twice per path is benign for the emitted
  result (roll guard works; previous_curled=N-1 both passes).
- Verified equal: curvatures (0.033 mid-top, matches upstream walk
  re-derived independently), estimate_curled_up_height inputs/outputs,
  signed boundary distance + offset, formula params
  (malformation_distance_factors .2/1.1, tolerance .1, max 10·h,
  expansion 1.0).
- Residual candidates, ranked: (i) CurledLine set membership near
  tolerance — orca's Z0.4 set plausibly contains mid-edge segments ares
  excludes; (ii) `detect_overhang_wall` relabeling giving orca extra
  erExternalPerimeter entities in the malformation pass; (iii) the
  `path.speed`/`original_speed` input for those segments already
  differing before `estimate_extrusion_quality` (path-attribute-level
  slowdown, not the quality estimator).

## Third narrowing pass (final state of the analysis)

Quantified the residual: inverting the band lookup on orca's emitted
speeds shows its effective slowdown distance is ~1.8–2.7× ares' on the
affected runs (orca art ≈ 0.075–0.118 vs ares ≈ 0.04–0.05). Ruled out
in this pass: flow_width (0.61 both), curvature (independent
re-derivation identical), curr.distance (boundary identical, inner
wall −0.589, outer wall ~0 → band0 both), role relabeling (no
erOverhangPerimeter emitted; TYPE labels identical), resonance
avoidance / small_perimeter (disabled), chained heights (L0 all zero).

Implication that survives elimination: for the artificial-distance
term to deliver ~2× ares' value, orca's CurledLine set on Z0.4 must
contain ~2× higher `curled_height` values OR extra mid-edge lines.
That cannot arise from the public inputs already verified equal
(distance 0.3, curv ≤ 0.55, heights ≤ ~0.35), so the port likely
differs in *which entities/lines feed* the malformation pass (the
`perimeters.flatten().entities` set — e.g. whether additional
entities are classified `erExternalPerimeter` upstream, or whether
the annotated points/segmentation differ upstream on the chamfer
layer), OR in a term of `estimate_curled_up_height` still unmirrored
(e.g. `curled_distance_expansion` or `params` fields the aggregate
init leaves non-default in orca's build path — e.g.
`bridge_distance` 16 passed as `max_line_length` changes
segmentation if edges were longer).

Suggested order of attack for implementation:
1. Dump ares' CurledLine set per layer (already-instrumented dump
   exists in git history); construct the needed set that yields
   orca's emitted speeds and compare cardinality/extent/height.
2. Read `perimeters.flatten()` role assignments upstream for this
   case — check if any inner/bridge entities have
   `role() == erExternalPerimeter` (e.g. via overhang detection) —
   would add inner-wall lines to `prev_layer_lines` AND to
   `prev_curled_extrusions`, explaining extra slowdown near inner
   wall.
3. If neither: take upstream `object_full.obj` debug output from a
   DEBUG_FILES build of orca to get the true CurledLine set.

## Final note: strongest surviving hypothesis

New inference that survived the uniform-slowdown test: orca slows
**uniformly ~99 mm/s across all four inner-wall edges** — artificial
distance from a corner-only curled set cannot produce a uniform value
(top edge is far from every curled corner). Uniform slowdown points
to `curr.distance` ≈ +0.078 on the inner wall (signed −0.227, i.e. a
previous-layer boundary edge ~0.23 mm inside the wall centerline —
e.g. a boundary set that includes the inner wall's own inner-edge
ring, the layer-region boundary rather than the raw slice, or
`lslices` containing the inner hole of the wall ring), or to
`original_speed` ≈ 99 for that path (path-level speed attribute,
upstream of the quality estimator). These are the two remaining
hypotheses; both are falsifiable by dumping the
`prev_layer_boundaries` line set and the `speed` argument upstream
passes to `estimate_extrusion_quality`.

## Resolution (implemented)

Root cause confirmed and fixed: `CurlTracker::collect` was invoked from
`overhang::estimate` on each **emitted path fragment** — open polylines
split at the seam/speed boundaries — so `curvatures()` saw
`looped = front == back` = false and the 8 mm `walk` truncated at the
fragment ends. Upstream `estimate_malformations`
(`SupportSpotsGenerator.cpp:141-196`) instead collects each
`erExternalPerimeter` **entity** via `extrusion->collect_points`: for an
`ExtrusionLoop` that is the concatenation of all sub-path polylines —
the full closed loop (`ExtrusionEntity.hpp:507-512`; `flatten()` yields
loops un-flattened since `ExtrusionLoop::is_collection()` is false).

Fix: collect once per loop entity in `perimeter.rs` before seam
placement (upstream runs malformation estimation during perimeter
generation, long before `split_at` can insert a seam vertex), once per
`ExtrusionMultiPath` in `multi_path::emit`, and keep the per-path
collect in `estimate` only for standalone `ExtrusionPath` entities
(`inside_loop`/`inside multipath` flag plumbed through `Emission` /
`PathProperties`).

Result on the stock C1 case (Sermoon V1 + cube10, seeded oracle,
no-cooling variant + original): the catastrophic Z0.6 wall-speed
divergence is eliminated — emitted F values on the overhang layer now
match the oracle's slowdown band-for-band.

Residual (documented for follow-up): ~190 diff lines remain out of
~2600, dominated by (a) one seam-adjacent outer-wall segment per layer
slowed ~10% more in ares (F6120 vs F6818.82, artificial distance 0.081
vs 0 in oracle — traced to the degenerate i==0 CurledLine sitting at
ares' loop-start corner; if upstream's natural loop start vertex is a
different corner, its degenerate line covers a different region), (b)
E-last-digit rounding drift on a handful of moves, (c) M73 progress
line placement (oracle ordering flake class) and knock-on
cooling-buffer layer-time F729/F750 differences. The byte-compare RED
test exists at
`crates/ares-cli/tests/orca_parity/curled_perimeter_slowdown.rs` and
currently still fails on those residuals; the loop-entity collect fix
is nevertheless independently correct against the cited upstream
boundary.

## Residual slice spec: loop vertex order at `collect_points` time

**Scope**: the ~30-line residual on layers Z0.8+ — two seam-adjacent
outer-wall segments per layer slowed ~10% too much (F6120 vs oracle
F6818.82; artificial distance 0.081 vs ~0). Traced to the degenerate
`i == 0` CurledLine: `estimate_points_properties` emits a zero-length
line at point 0 with `curled_height` from `curvature[0]` — in ares that
point is the loop's natural start vertex, the square's corner, whose
windowed curvature is π/3 (upstream-faithful per the
`square_corners_carry_the_windowed_curvature` test: the seam-start
corner gets a zero-length backward tangent and `atan2(+0,−0) → π`,
matching the MK4S oracle observation). If upstream's `paths[0]` start
vertex is a different point (non-corner, or a different corner whose
degenerate line chains to < 0.1 height), the whole residual follows.

**Upstream vertex-order chain** (Arachne, since stock
`wall_generator` default = `PerimeterGeneratorType::Arachne`,
`PrintConfig.cpp:7155-7167`; classic `traverse_loops` is NOT on this
path):

1. `process_arachne` (`PerimeterGenerator.cpp:2093+`) offsets
   `surface.expolygon` via `offset_ex(..., -ext_perimeter_width/2 +
   ext_perimeter_spacing/2)` — vertex 0 of the offset contour is
   ClipperLib-v1's `OutRec` ring head (`clipper.cpp` append/hole
   semantics), not a normalized lowest-vertex.
2. Arachne `WallToolPaths` builds the outer wall
   (`Arachne/WallToolPaths.cpp:68+`) from the inset outline; the closed
   `extrusion`'s junction 0 follows the outline vertex order.
3. `extrudePathsToExtrusionEntities`
   (`PerimeterGenerator.cpp:~528-566`) splits the closed extrusion at
   `ExtrusionJunction` boundaries into `paths`; overhang splitting
   (`append_overhang_clipped_paths` equivalent, :440-520) then calls
   `chain_and_reorder_extrusion_paths(paths, &start_point)` where
   `start_point` = the first `occurrence==1` junction in
   `std::unordered_map` iteration order, preferring non-overhang
   (:491-506) — **hash-order-dependent** when multiple
   occurrence-once candidates exist.
4. `estimate_malformations` (`PrintObject.cpp:900-914`, runs during
   perimeter generation) sees this chained order; `paths[0]`'s first
   point becomes the degenerate `i == 0` CurledLine position.

**Ares equivalence points to compare**:
- `ares-core/.../mesh_slicer/chaining.rs` (loop seed = first spare
  line, mirrors `chain_lines_by_triangle_connectivity`
  `TriangleMeshSlicer.cpp:1082-1146`).
- `geometry/clipper` offset output ring-head order vs
  `clipper.cpp`'s `OutRec::pts` head for a simple square offset.
- `arachne/traverse/overhang.rs::select_open_start` uses
  `PointHashMap::into_ordered()` — deterministic — where upstream's
  `unordered_map` order may pick a different occurrence-once point.

**Verification plan** (choose cheapest first):
1. Dump ares' loop vertex list at `collect_entity` (env probe) for
   layers 1-3 of the C1 case → already known: starts at corner
   (4.645,4.645), 5 lines/loop (4 whole-edge + 1 degenerate).
2. Build orca with `-DSLIC3R_DEBUG_FILES` (or `OrcaSlicer_DEP` debug
   env `debug feature`) to emit `object_full.obj`/`layer` debug
   polygons, recovering upstream's pre-seam loop vertex order for the
   same layers — establishes whether upstream's paths[0] also starts
   at a corner (then residual is elsewhere) or not.
3. If DEBUG_FILES build is unavailable, infer from oracle output: with
   the seeded oracle the only extra-slowed segment should mark the
   degenerate line's position; orca's Z0.8/Z1.0 walls show NO
   extra-slowed seam-adjacent segment, suggesting upstream's i==0 line
   either sits at a non-corner (curvature 0 → chain-only h ≈ 0) or its
   h decays below the 0.1 threshold.

**Exit criterion**: C1 case `orca_parity` full byte-compare PASS
(modulo documented M73 placement + E-ulp flake classes), or a
byte-exact demonstration that upstream's collect-time vertex order is
reproduced.

## Residual investigation — second pass (2026-10-06)

**Mechanism exclusions verified**:
- `point_occurrence`/`std::unordered_map` start-point selection runs ONLY
  for `!extrusion->is_closed` (`PerimeterGenerator.cpp:463-497`). The C1
  cube wall is a closed extrusion → `start_point =
  paths.front().first_point()`; no map. The libstdc++ unordered_map
  simulation is therefore NOT needed for this residual (the
  `bridge_direction.rs` extraction stays unimplemented for now).
- Ares' collected input (`collect_entity`, all layers): a single
  `ExternalPerimeter` path of 5 points (4 corners + closing duplicate),
  vertex 0 = corner (4.645,4.645); no overhang `clip_extrusion` split on
  this wall → `clip_extrusion` Clipper2Z ordering is also excluded.
  Remaining vertex-order candidate: the upstream offset contour's ring
  head / Arachne wall junction 0 — cannot be observed without an
  upstream DEBUG_FILES build, but see below.

**NaN-semantics experiment (reverted)**: ares' `walk` interpolation
leaves `position` unchanged when the delta is zero-length, producing a
(0,0) back/front vector and `atan2(+0,-0) = π` → curvature[0] = π/3 for
the loop-start corner (documented in
`square_corners_carry_the_windowed_curvature` as oracle-verified).
Upstream `normalized()` on the same zero vector yields NaN, so the
curvature update is discarded (`abs(cur) < abs(NaN)` is false). Making
ares propagate NaN identically gives curvature[0]=curvature[1]=0 on the
square — and the whole CurledLine set shifts: the seam-adjacent
extra-slowdown disappeared, but the divergence shape changed (151 diff
lines vs 42 before) — orca's inner wall is slowed MORE (F5940 on 3
edges) than either ares variant. Conclusion: the π/3 seam-start quirk is
upstream-faithful for at least some cases; the NaN model cannot be right
unconditionally, or another compensation exists upstream (e.g. upstream
feeds a slightly different point list where pt0 is not at a dup corner —
loop vertex order again).

**Orca band shape (no-cooling)**: outer wall Z0.8 emits F4800/F5580 on
seam-adjacent scarf groups, F5400/F5520/F4800+F5580 mid-loop; inner wall
F5940×3 edges + F6060. Ares (pre-NaN) emits F6360/F5520/F5400/F5520/F6300
outer, F6477.88 inner uniform — both directions differ from oracle, so
the residual is NOT a single localized fix: upstream's CurledLine h
values differ per position in a way that requires the actual line set.
Decision point reached within the time box: next step requires upstream
`layer->curled_lines` — either a `-DSLIC3R_DEBUG_FILES` orca build
(`SupportSpotsGenerator.cpp:144-147` writes
`object_malformations.obj`/`object_full.obj`) or an upstream instrument
that dumps `curled_height` per segment.
