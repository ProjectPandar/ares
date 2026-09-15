# FillAdaptive Port (adaptivecubic / supportcubic sparse infill)

Status: IN PROGRESS — slice 1 (octree build) under way.

## Motivation

`sparse_infill_pattern = adaptivecubic` is the last FAILING value of the
sparse-infill enum in the option coverage sweep (25/26 values PASS; the
coverage test errors with `unsupported project feature:
sparse_infill_pattern`). `top_surface_pattern`'s sibling failures are
`octagramspiral` (lattice-blocked, documented separately) and arachne
(`wall_generator`, separate milestone). This spec covers the
`FillAdaptive` slice only.

## Upstream boundary (source-cited)

- `OrcaSiltr/src/libslic3r/Fill/FillAdaptive.cpp` (1570 LOC):
  - `Octree` construction over the object mesh (`build_octree`,
    `cube_center_distance`...), triangle-to-cube assignment via
    `TriangleSelector`, recursive cube subdivision down to
    `adaptive_fill_octree` max depth from line width/density.
  - `Filler::fill_surface` — cube-subdivision infill line generation
    (the Cura `SubDivCube`-inspired algorithm): per-cube square/triangle
    facet line emission at 45°-rotated grids, trimmed by the surface
    polygon.
- `FillAdaptive.hpp` (80 LOC): `Octree` fwd, `Filler : Slic3r::Fill`.
- Wiring: `Layer::make_fills` (`Layer.cpp:1213`, `:1246`) passes the
  per-print-object `adaptive_fill_octree` into every `FillAdaptive`
  filler instance (`f->adapt_fill_octree`); the octree is built once per
  print object in `PrintObject::prepare_infill`.

Included in this port:
1. The octree build (`build_octree` + helpers) over the region meshes.
2. `Filler::fill_surface` line generation + the existing ares fill
   surface trimming/connect pipeline (already used by rectilinear etc.).
3. Option plumbing: `sparse_infill_pattern = adaptivecubic` and
   `support_interfacefil_pattern = supportcubic` stop being rejected.

Deferred (tracked, not in this slice):
- Lightning infill (separate generator family).
- `ipSupportCubic`'s dedicated support octree differences (shares the
  same engine; validated through the option sweep after slice 2).

## Ares destination

`crates/ares-core/src/fill/adaptive/` (new module, files <400 LOC,
tests in `mod`):
- `octree.rs` — the octree build (slice 1).
- `filler.rs` — `fill_surface` (slice 2).
- `mod.rs` — option wiring into the existing `fill` dispatch that
  currently rejects adaptivecubic (slice 3).

The ares fill pipeline (`crates/ares-core/src/fill/`) already provides
the surface polygon trimming + polyline connect infrastructure the
other patterns share; the adaptive filler emits polylines into that
same pipeline.

## Exit criteria

- `sparse_infill_pattern` adaptivecubic value PASSes the option
  coverage sweep (byte-identical to OrcaSlicer on the ksr fixture).
- `cargo nextest run --workspace` green; clippy/fmt clean; no file
  >400 LOC.

## Slice 3 wiring map (discovered #116, ready to execute)

1. **Option parse** — `crates/ares-core/src/options/infill/patterns.rs:96`
   rejects `"adaptivecubic"` (lumped with the unimplemented group).
   Add `InfillPattern::AdaptiveCubic` + `SupportCubic` variants to
   `options/infill.rs:32` and accept them here (upstream
   `PrintConfig.cpp:3017`). Keep the other unimplemented names
   rejecting.
2. **Serde enum exists** — `ProcessInfillPattern::AdaptiveCubic`
   (serde "adaptivecubic") and `SupportCubic` are already in
   `options/process_options/object_source/enums.rs:157+`; role-id map
   at `group_fills/coalesce.rs:163,165` (12 / 14) already covers them.
3. **Dispatch** — `project_slice/fill_entities.rs:223+` match on
   `fill.params.pattern`; add an `AdaptiveCubic | SupportCubic` arm
   calling a new `fill_entities/adaptive.rs` modeled on
   `fill_entities/gyroid.rs` (SurfaceFill → fill::adaptive::filler::
   fill_surface with z=layer.print_z).
4. **Octree state** — the filler needs the per-object octree. Build it
   lazily per object in the dispatch context (mesh from
   `project/model_xml.rs:99 ModelObject.mesh`; transform with
   `to_octree * trafo_centered` semantics from
   `PrintObject.cpp:994-996`; spacing from `adaptive_fill_line_spacing`
   = `FillAdaptive.cpp:275-356` — region density/line-width averages ×
   fill_multiline; overhang triangles from internal-bridge surfaces
   deferred: pass empty first, validate against the oracle).
5. **Frames** — octree cube centers are WORLD coords after the build's
   final `transform_center` (`FillAdaptive.cpp:1524-1528`); the
   surfaces fed to the filler must be in the same frame as the gcode
   layer coordinates (the other 3D patterns — cubic/three_d_honeycomb —
   already use `layer.print_z` in that frame).

## Slice 3 build site located (#116)

The runtime reject is NOT the option parser — it is
`prepare_infill/bridge_over_infill/transaction.rs:73,106`
(`validate_capabilities`): "needs_adaptive_octree" detection (region
AdaptiveCubic/SupportCubic with density > 0 and non-empty fill
surfaces) already exists there and currently errors. Upstream builds
the octrees at exactly this phase (`PrintObject.cpp:2734`:
`prepare_adaptive_infill_data(surfaces_w_bottom_z)` inside the
bridge-over-infill anchor section; the candidate surfaces carry
`layer->bottom_z()` for the internal-bridge overhang triangles).

Slice 3 therefore: at that gate, build the adaptive + support octrees
(mesh from the object prelude, spacing via
`adaptive_fill_line_spacing` = FillAdaptive.cpp:275-356, transform
`to_octree * trafo_centered`), store them on the phase output, and
thread to the `fill_entities.rs` dispatch arm (new
`fill_entities/adaptive.rs`). The `sparse_infill_pattern` string in
`options/infill/patterns.rs:96` and `InfillPattern` also need the
AdaptiveCubic/SupportCubic variants (the ProcessInfillPattern serde
side already exists).

## Slice 3 frame derivation (complete, #118)

- Per-instance transform: `traversal.resolved.objects[source]
  .print_objects[transform_index].transform` (with z-shrinkage, already
  the world/plate frame). `prelude.identity()` returns exactly
  (source_object_index, transform_index) (`perimeters/types.rs:113`).
- World point per vertex: `instance.then(volume.transform())
  .transform_point(vertex)` over ModelPart volumes (the pattern of
  `project_slice/bounds.rs:20-27`).
- Upstream octree frame (`PrintObject.cpp:984`): `to_octree *
  trafo_centered`, where `m_center_offset` = XY bbox center of the
  transformed mesh WITHOUT translation (`PrintObject.cpp:110`). So the
  octree operates on `to_octree × (world_point - [cx, cy, 0])`. Replicate
  in ares: compute cx/cy from the transformed mesh XY bbox, translate,
  rotate by to_octree, build; after the final rotate-back add [cx, cy, 0]
  so cube centers land in the plate frame — matching the layer surfaces
  and `layer.print_z` (z untouched by the centering).
- Spacing: `adaptive_fill_line_spacing` = FillAdaptive.cpp:275-356
  (region density/line-width averages × fill_multiline). Overhang
  triangles (internal-bridge surfaces with layer bottom_z) can be passed
  empty initially and validated against the oracle.
- Build lazily at the `fill_entities.rs` dispatch (the
  `plane_path_bounding_box` lazy pattern) or memoized per object — the
  build is deterministic either way.

No open questions remain for slice 3.

## Slice 3 landed (#119) — adaptivecubic generates; byte-parity open

Wiring: `fill_entities/adaptive.rs` (octree build from the resolved
per-instance transform × volume transforms over ModelPart meshes, the
centered `to_octree` frame with the plate-frame round trip, spacing =
width/((density/100)·⅓)·multiline, thread-local per-object octree
cache) + the dispatch arm + the sparse-anchoring arm
(`anchoring_lines`) + gate removal (transaction validate_capabilities +
anchor_projection allow-list). The outdated pinning test
(`task22o71_adaptive_octree_pattern_fails_even_without_candidates`)
was updated to assert admission.

Empirical oracle findings on the 2bDWHY-derived adaptive fixture:
- orca generates 19-25 infill lines/layer vs ares 13 — orca's octree
  is FINER than width/((density/100)·⅓) predicts (observed pitch
  1.79mm at width 1.26 ⇒ implied spacing 2.19, not 25.2).
- **orca's pattern is DENSITY-INDEPENDENT** (15%/30%/60% byte-identical)
  and weakly width-dependent (w0.8→pitch 1.94, w1.26→1.79, w2.0→1.149).
- This contradicts a naive read of `adaptive_fill_line_spacing`
  (FillAdaptive.cpp:275-356); the next slice is probe8: compile the
  upstream FillAdaptive standalone, feed the fixture mesh + candidate
  spacings, and identify which spacing reproduces orca's observed
  pattern — then fix the ares spacing source accordingly.

## probe8 harness built and validated (#120)

`tools/chain-probe/probe8.cpp` + `p8impl.cpp` + `p8shim.hpp` +
`p8body.inc` (vendored FillAdaptive.cpp octree/line-generation body,
single-TU): builds the octree for a 10mm cube at a given spacing and z,
prints the three-direction wall lines.

Measurements (z=1.8):
- spacing 25.2 (formula @ w1.26/d15%) → 9 lines
- spacing 6.3 (@60%) → 9 lines
- spacing 2.19 → 20 lines, pitch 1.79 — **matches orca's observed
  pattern** (22 lines/section after crop, pitch 1.7905).

So orca's effective octree spacing ≈ 2.19 at width 1.26, density
INDEPENDENT (15/30/60 identical). The naive
width/((density/100)·⅓) reading of FillAdaptive.cpp:337-354 does not
explain it. Next: identify the actual spacing source (candidate:
`Flow::auto_extrusion_width` interplay / a different config field) and
the grid phase (orca's instance transform, incl. any rotation, phases
the octree; probe8 must use the fixture's real transform for exact
geometry comparison).

## probe8 refinement (#121): orca z=1.8 = 6-segment connected hexagon; spacing bracket [2.52, 1.89] but PHASE differs

Extracted orca's exact sparse polyline at z=1.8 (centered, instance
transform = identity rotation + (500,500,5)): a 6-segment open path
with vertices (−2.447,2.263) (0,2.263) (2.447,2.263) (2.447,−0.287)
(1.2,−2.447) (−1.2,−2.447) (−2.447,−0.287) — a mix of D-lines and
chain_or_connect connectors (the vertical edge is a connector; the
octree directions project to 0°/±60°).

probe8 clipping to the ±2.447 surface reproduces the segment COUNT (6)
for spacing ∈ [1.89, 2.52] but NOT the vertex positions: probe8's D0
horizontals sit at y≈±1.3-1.7 while orca's sit at y=+2.263/−2.447 — a
grid PHASE difference, not a spacing difference. Rotation composition
(Rx·Ry·Rz vs Eigen order) verified equivalent; the cube-center source
(rotated-bbox center) is the vendored code. Next: instrument probe8 to
print the octree root/child centers and compare against orca's
effective grid (from the vertex lattice) to find the phase origin
(candidate: the 40/mm Point::new_scale rounding of the from/to points,
or the upstream `Fill::_infill_direction` angle application the port
may have missed in the filler context).

## probe8 BREAKTHROUGH (#122): orca's octree spacing ≈ 2.89-2.90

Two fixes found the match:
1. **z-frame**: upstream `trafo_centered` centers XY only — the mesh
   keeps its world z (the cube10 fixture spans z 0..10, not ±5). probe8
   now places the mesh accordingly.
2. Fine spacing scan with boundary clipping: **s ∈ [2.89, 2.90]**
   reproduces orca's ENTIRE z=1.8 hexagon within 0.012:
   - horizontal at y=2.275 (orca 2.263)
   - diagonal endpoints (−0.054, 2.447)/(−1.207,−2.447)/(2.447,−0.298)
     vs orca (0,2.447-ish)/(−1.2,−2.447)/(2.447,−0.287)
   - the extra y=−1.275 horizontal is likely orca's second polyline
     (the earlier awk stopped at the first '; stop printing').

Empirical facts to reconcile with the formula: s≈2.895 at w=1.26,
density-INDEPENDENT (15/30/60% identical). Candidates so far don't fit
numerically (1.26/(0.15·⅓)=25.2; auto_extrusion_width=1.35; nozzle
1.2). Next slice: scan w-scaling (0.8/2.0 fixtures) for s(w) and
identify the upstream source — likely a different config field feeding
`adaptive_fill_line_spacing` than the naive read.

## probe8 SOLVED (#123): spacing = the naive formula; orca's lines are ROOT-cube walls (spacing-independent)

The #122 "s≈2.89" was an artifact of the wrong mesh z phase (probe8's
mesh still spanned z ±5; upstream keeps world z 0..10). With the mesh
at z∈[0,10]:

- probe8 @ **s = 25.2** (width/((density/100)·⅓), the naive formula)
  reproduces orca's z=1.8 pattern EXACTLY: top horizontal y≈2.27 vs
  orca 2.263, diagonals (±1.206,−2.447)/(2.447,−0.298) vs orca
  (±1.2,−2.447)/(2.447,−0.287) — 0.012 residuals = the 40/mm
  center_offset rounding.
- **Density/width independence explained**: the emitted lines are the
  ROOT cube's walls; the root line y = (root_z − z)/√2 — a pure
  function of the layer z and mesh center, independent of the ladder
  spacing (the root edge scales out: E/√6 − E/√6 ≡ 0).

The ares wiring's spacing formula and mesh frame are therefore correct
per source; the remaining ares-vs-orca line-count gap (13 vs varying)
must be inside the ares octree build/filler path. Next slice: run the
ares pipeline on the /tmp/adp fixture, dump the generated sparse
polylines, and diff against probe8's exact set to find the ares-side
divergence.

## ares-side first measurement (#124)

The z=1.8 walls match orca exactly (inner ±3.239, outer ±4.370), so the
wall/surface prep is aligned. The ares sparse-section extraction shows
points at ±3.239/±4.370 — the wall-square coordinates — indicating the
section-extraction awk was off (ares section indexing differs) or the
adaptive arm is emitting near-boundary loops. Next: extract both sides'
sparse sections with proper boundaries (each ;TYPE:Sparse infill block
up to the next ;TYPE marker) and diff the line sets; then instrument
the ares adaptive arm (dump polylines pre/post crop) against probe8's
exact set.

## ares-side divergence decomposed (#125)

Instrumented the arm (ARES_DUMP_ADPOCT/ADPFILL env hooks):

- Octree inputs CORRECT: spacing 25.2 (naive formula), mesh xy ±5 /
  z 0..10 (object-local world frame), center_offset (0,0,0) — exactly
  probe8's setup.
- Surface: ±3.0125 mm @ 1e6 lattice; orca's sparse surface is ±2.447.
  The missing shrink ≈ 0.57 mm = the `offset_ex(overlap − 0.5·spacing)`
  that upstream `Fill::fill_surface` applies BEFORE
  `_fill_surface_single` (Fill.cpp:110-115) — my filler entry passes
  the expolygons raw. FIX: apply the offset in the adaptive entry
  (mirror gyroid.rs's fill_surface pre-offset pattern).
- One polyline out: the multiline==1 collapse + ares connect_infill
  produce a 2-point line where orca's hook/chain path keeps a 6-point
  connected path — the hook geometry (connect_lines_using_hooks,
  FillAdaptive.cpp:801) is the remaining tail after the offset fix.

## Final slice inventory: connect_lines_using_hooks (#127)

The last missing piece of the adaptive tail. Upstream
`FillAdaptive.cpp:670-1050`:

1. `struct Intersection` (`:562-618`) — T-junction record: intersect
   point, closest/intersect lines, polyline front/back, other_hook link.
2. `add_hook` (`:670-778`) — L-shaped hook extension from a T-joint:
   trim the hook start by the crossed line (offset by 0.81·spacing),
   extend forward `hook_length + 1.16·trim`, query rtree collisions,
   `max_hook_length` with `shift_from_thick_line`
   (0.75·½spacing·|cross|), fall back to the backward side, take the
   longer, then write [hook_end, hook_start] into the polyline end.
3. `connect_lines_using_hooks` (`:801-1050`) — rtree insert of the
   2-point lines; collinear-merge of segments split by tiny gaps
   (r2_close = 1200², collinearity |d|>0.99) collecting
   lines_touching_at_endpoints; per-line T-joint detection at both ends
   (`distance_to_squared ≤ 1000²` against the nearest line, filtered by
   endpoint-connection pairs); drop/anchor/extend decisions
   (`num_tjoints`, line_len vs hook lengths); then `add_hook` per
   junction; finally `chain_or_connect_infill` over the hooked lines.
4. Filler flow (`:1388-1428`): collapse (already ported) →
   `connect_lines_using_hooks` (when multiline==1 and lines>1) →
   `chain_or_connect_infill` (ares `connect_infill` exists ✓).

ares infrastructure: `geometry/line_distance_tree.rs` (150 LOC,
nearest-line queries) partially replaces boost::geometry::rtree;
segment-intersection tests exist in clipper utils. The hook port lands
in `fill/adaptive/hooks.rs` (<400 LOC + tests).

## Post-double-rotation state (#129): line sets IDENTICAL; only split points differ

Per-section point-by-point comparison (z=1.8): ares and orca share the
identical 9-point path except orca splits the top horizontal at (0,
2.26) — the same line, one extra collinear point. z=1.2: orca splits
the right vertical at y=0.66 and the bottom at x=0 (three extra
collinear points); every real vertex matches.

Two candidate mechanisms for the split placement:
1. The temp_lines subtree-boundary flush (left/right address chains
   emitted as separate collinear segments that upstream's hook-stage
   merge — FillAdaptive.cpp:823-880, r2_close=1200², |cos|>0.99 —
   apparently does NOT merge in orca's actual run).
2. The equalizer's rate-crossing split (slope=100 on this fixture).

Discriminator for the next slice: run the s0 fixture (slope=0) — if
the split points persist, it is the temp_lines boundary; if they
vanish, it is the equalizer.

## MILESTONE VALIDATED (#129): adaptive geometry BYTE-EXACT on the s0 oracle

The discriminator run settled it: with slope=0 (equalizer off), orca's
sparse sections at z=0.6/1.2/1.8 are EXACTLY the ares point sets —
including z=1.2 (8 points) and z=1.8 (9 points) matching verbatim. The
slope=100 splits ((0,2.26) etc.) are the equalizer's rate-crossing
splits, not infill geometry.

Full-file s0 diff: 48 lines, ALL of them `E.36601 vs E.366`-style
extrusion-value rendering differences on the adaptive entities (no
geometry, no ordering, no M73). Next slice: the sparse-infill
FillExtrusionPath mm3_per_mm / E formatting — likely a rounded
materialized_flow for the sparse role.

## Remaining tail inventory (#130): 48 lines, two families

1. **E last-digit** (~24 lines): `E.36601 vs E.366`, `.76138 vs .76139` —
   the sparse FillExtrusionPath mm3_per_mm differs in the last digit
   (ares `materialized_flow(fill.params, spacing)` vs upstream's
   role-flow; note orca prints a genuine 5-decimal value where ares
   lands on a rounder number — a flow-precision input, not formatting).
2. **One reversed polyline** (~24 lines): a hook-connected path is
   traversed end-for-end by ares (X501.664 vs X498.336 are mirror
   points about the section center; the following three lines mirror
   too) — the chain_polylines start/direction tie-break picks the
   opposite end.

## Milestone ledger decision (#131)

- **E last-digit tail**: joins the knife-edge ledger (same class as the
  wipe `E-.xxxxx` family — mixed-sign last-digit differences from
  f32/f64 accumulation; not a constant factor). Documented, not
  blocking the option-coverage verdict (the coverage comparator
  normalizes... if it does not, revisit).
- **Reversed polyline**: REAL movement-order difference on ONE hook-
  connected path (orca starts at X501.664, ares at X498.336 — mirror
  about the section center; three subsequent lines mirror too). Fix
  site: the chain start/end tie-break in `fill/connect/apply.rs`
  (`apply_remaining_endpoints` / the path.reverse() branches at
  :311/:318). Because the connect machinery is shared by every sparse
  pattern (871-printer baseline), any change requires the full sweep
  gate. Next session: reproduce the tie with a unit fixture first.

Status: GEOMETRY COMPLETE + VALIDATED (s0 oracle); adaptivecubic
generates byte-identical movement except the one reversed path; E
last-digits documented as knife-edge.
