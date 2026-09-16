# Raft Layers Milestone (raft_layers option domain)

## Status: IN PROGRESS — slice 2 (z-grid) committed (`8aa31fd3`)

Additional upstream anchors confirmed while porting the grid:
- Contact layer z: `new_contact_layer` raft branch
  (`SupportMaterial.cpp:1738-1746`): `print_z = raft_contact_top_z`,
  `bottom_z = raft_interface_top_z`, `height = contact_raft_layer_height`.
- Contact silhouette: `detect_contacts` layer_id==0 branch
  (`SupportMaterial.cpp:1573-1577`): `expand(overhang_polygons,
  raft_expansion)` — the overhangs for layer 0 (empty lower layer) are
  the whole first-object-layer lslices.
- Object layer ids shift by `raft_layers()` (`SupportMaterial.cpp:2130`
  comment: "layer->id incorporates the raft layers") — ares'
  `apply_raft_expansion` (print_paths/support_interface.rs:72) already
  assumes this id layout.
- `raft_layers()==1` degenerate case: no generate_raft_base layer
  push at all; the single contact layer comes from top_contacts
  (`Slicing.cpp:208-211`).

Goal: un-gate `raft_layers` (capabilities.rs rejects nonzero today) with
byte-parity raft emission on the KSR sweep cases
(`option/raft_layers/max` = 100, `option/raft_layers/seeded`),
verified against the live 2.4.2 oracle artifact
`/tmp/ares-parity-artifacts/case-u7sdch/orca.gcode`.

## Upstream boundary (source-cited)

| Concept | Upstream anchor | Ares destination |
|---|---|---|
| raft z-grid math (base/interface/contact split, heights, gap) | `src/libslic3r/Slicing.cpp:194-236` (`SlicingParameters`) | `project_slice/parameters.rs` — ALREADY PORTED and exact |
| raft layer stack + polygon expansion | `src/libslic3r/Support/SupportCommon.cpp:244-390` (`generate_raft_base`): brim trim → `inflate_factor_fine` (0.5mm when raft_layers>1), `inflate_factor_1st_layer` = max(0, raft_first_layer_expansion − fine), contact = first object layer silhouette w/o holes, base/interface/contact layer push loop | `project_slice/raft/` (new mod: `grid.rs`, `polygons.rs`) |
| raft toolpaths (base/interface/contact fills) | `SupportMaterial.cpp` support-layer fill (`fill_support` paths; densities `raft_first_layer_density` 90%, `raft_base_density`, `raft_interface_density`; spacing = width*(100/density − 1)) | reuse existing rectilinear fill machinery |
| raft emission (`;TYPE:Support`, `;TYPE:Support interface`) | `GCode.cpp` support-layer print (`;TYPE:` from ExtrusionRole) | existing role→TYPE mapping (`motion/features.rs:219`) |
| first-layer interaction | skirt on raft layer 1-2 (skirt_height semantics vs raft), brim auto-brim off with raft | existing skirt/brim modules |

## Verified oracle ground truth (case-u7sdch, raft_layers=100, KSR cube)

- z grid: 0.2 (first), 0.5…14.9 step 0.3 (49 base), 15.2…29.6 step 0.3
  (49 interface), contact 29.9, object z-min = 29.9 + gap 0.2 → 30.1,
  then object layers 0.2 step to 40.1. Total 150 ;LAYER_CHANGE.
  Matches `Slicing.cpp` math exactly (base=interface=50 split).
- Layer 1 emits skirt (TYPE:Skirt, WIDTH:0.42) then raft first layer
  (TYPE:Support, F2100, density 90%).
- Object sits above raft; wipe/retract between raft and object regions
  (E-2.8 F3600 wipe pattern — existing machinery).

## Included / deferred

- INCLUDED: raft-only path (enable_support=0) — the sweep case scope;
  base/interface/contact z grid; silhouette expansion; base/interface
  fills; emission; skirt-on-raft interaction.
- DEFERRED: raft + support-columns interaction (SupportMaterial.cpp
  column base trimming `columns_base` branches), tree supports with
  raft, `first_object_layer_bridging` (forced false upstream
  `Slicing.cpp:203`), painted layer-height profiles with raft.

## Plan slices (each commit+push)

1. ✅ Spec (this doc).
2. `project_slice/raft/grid.rs`: raft z-grid (first/base/interface/
   contact/object-shift) derived from SlicingParameters, unit-tested
   against the oracle z sequence above. Gate STAYS.
3. `project_slice/raft/polygons.rs`: contact silhouette (holes
   filled) + expansion factors (fine 0.5mm, 1st-layer expansion),
   unit-tested against oracle layer-1/2 perimeters.
4. Raft fills (base/interface/first densities → spacing → rectilinean
   paths, roles SupportMaterial/SupportMaterialInterface).
5. Emission wiring: raft layers prepend to layer stream, object
   print_z shift, skirt interaction. Run oracle case; iterate to
   byte parity; THEN remove the capabilities gate.
6. Sweep verification (ARES_PARITY_SWEEP=1) + docs update.

## Verification per slice

- Unit tests source-cite upstream lines; no golden weakening.
- Final: `option/raft_layers/max|seeded` PASS in the sweep; ksr
  golden untouched; workspace nextest green.

## Slice progress + oracle calibration (#204)

- `e93270ff`: EdgeGrid SDF + `contours_simplified`
  (`EdgeGrid.cpp:672-886`, `:1283-1400`) — geometry primitive 1.
- `e64bbd4a`: `SupportGridPattern` smsGrid port
  (`SupportMaterial.cpp:637-836`) — geometry primitive 2 (grid
  stretching + island sample filter).

Oracle calibration (case-u7sdch, cube lslices ≈ 105..115mm):
- raft 1st layer fill span 101.059..118.941 (17.882) —
  grid-cell-snapped `expand(lslices, raft_expansion=1.5)` (13mm
  input) grown to full grid cells; implies grid_resolution ≈ 2.4mm+
  (support_base_pattern_spacing + flow spacing — exact value to be
  wired from SupportParameters in the emission slice).
- interface layers fill 102.518..117.482+ — = contact + 0.5
  (`inflate_factor_fine`, jtSquare).
- contact layer (z=29.9) fill 103.018..117.482 — the grid-extracted
  contact silhouette (NOT +0.5'd).
- object first layer outer-wall centerline 105.225..114.775 (10mm
  cube at center 110).
- Skirt on raft layer 1: 99.245..120.755.

Next: `raft/polygons.rs` chain (first ⊇ base = union(interface) ⊇
interface = expand(contact, 0.5, jtSquare) ⊇ contact), wired to
SupportParameters in emission; then fills + emission + gate removal.

## Slice 3 done (`5bf73c40`) + fill map for slice 4 (#205)

Polygon chain ported and oracle-consistent: contact 102249..117319,
interface = contact ± 500 (jtSquare), base = interface, first =
base ± (first_exp − fine). Holes grid-filled. ares-core 6965/6965.

Raft fill map (`SupportCommon.cpp:1440-1523`, per raft layer):
- Layer 0 (base flange): pattern `raft_interface_fill_pattern`,
  angle `raft_angle_1st_layer`, spacing `first_layer_flow.spacing()`,
  density `raft_first_layer_density` (90%), role erSupportMaterial,
  flow `first_layer_flow`.
- Layers 1..base_raft_layers: pattern `base_fill_pattern`
  (support_base_pattern), angle `raft_angle_base`, spacing
  `support_material_flow.spacing()`, density `support_density`,
  role erSupportMaterial, with_sheath.
- Interface layers (>= base_raft_layers incl. contact): pattern
  `raft_interface_fill_pattern`, angle `raft_interface_angle(
  interface_id)` (alternating per layer), spacing
  `support_material_flow.spacing()`, density
  `raft_interface_density`, flow `raft_interface_flow`, role
  erSupportMaterialInterface.
- link_max_length = spacing * link_max_length_factor / density.
- Interface-layer fills use base polys (`raft_layer.polygons`) but
  base-layer loop infills base polys via `to_infill_polygons`;
  tree cut-through deferred (no trees in KSR raft-only).

Next: port SupportParameters raft fields (SupportParameters.hpp),
fill emission via existing rectilinear machinery, then wire layers
into the emission stream (ids shift by raft_layers) + gate removal.

## Slice 4a done (`0f100e5c`) + remaining inventory refined (#207)

`raft/fill_params.rs`: densities/patterns/angles derivation with
per-layer ±45° interface alternation. One port bug fixed in-review:
the even-`interface_raft_layers` +90° rotation applies ONLY to
`raft_angle_interface`, never to the flange (`:153-158`). KSR values:
flange 90°, base 0°, interface π ± π/4, densities 0.407/0.607.

Key structural finding: `FillSupportBase::fill_surface`
(`FillRectilinear.cpp:3610-3634`) does NOT use the plain
rectilinear connection — it uses `connect_base_support`
(`FillBase.cpp:2247` + `emit_loops_in_band` `:1952`), a distinct
algorithm (contour-band arch emission) from `connect_infill`
(plain rectilinear, `:3038-3041`). ares' rectilinear port implements
the connect_infill family; the support-base variant must be ported.

Remaining slices:
4b. `fill/base_support.rs`: connect_base_support + emit_loops_in_band
    port (FillBase.cpp:1952-2300, ~350 LOC upstream).
4c. `raft/fills.rs`: per-layer fill assembly — vertical lines at
    spacing/density, angle per layer kind, roles/flows/widths, via
    4b; flange uses first_layer_flow, interface uses
    raft_interface_flow (SupportCommon.cpp:1440-1523).
5.  Emission wiring: raft layers prepend to the layer stream
    (object layer ids shift by raft_layers, print_z absolute),
    skirt on raft layer 1, wipe/retract transitions.
6.  Gate removal + oracle byte-parity loop on case-u7sdch +
    `option/raft_layers/seeded`; full sweep; docs close-out.

## Slice 4b in progress — structure mapping for connect_base_support (#208)

Upstream `connect_base_support` (`FillBase.cpp:2247-2480`) decomposes
into (all citations `FillBase.cpp`):
1. `create_boundary_infill_graph` — ares: `connect::graph::
   build_working_graph` ✓ EXISTS.
2. `mark_boundary_segments_overlapping_infill` — MISSING in ares
   (only the touching pass exists); uses
   `rounded_thick_segment_collision` — ares HAS it
   (`connect/collision.rs:174`).
3. Empty-contour perimeter loops (`:2281-2296`): contours with zero
   infill endpoints and perimeter > trim_length + 0.5·line_spacing
   emit a clipped perimeter loop.
4. Excess arches via `emit_loops_in_band` (`:2298-2324`) — PORTED
   (`6d8d4e88`), band = [x + half_width, x + line_spacing −
   half_width] oriented by `graph.first(cp)`.
5. `base_support_extend_infill_lines` (`:1834-1948`): walk the
   contour next/prev while |Δx| ≤ 0.33·line_spacing stopping at the
   neighbor's point_idx; extend when Δy (sign-flipped for `first`)
   > 0.5·line_spacing and the arc fits in contour_not_taken_length;
   prefer the longer Δy side; `take_cw_full`/`take_ccw_full` = ares
   `connect::contour::append_full`; trims the taken side, and the
   non-trimmed side re-derives via closed_contour_distance_ccw.
6. Main connection loop (`:2326-2480`): merged_with union-find,
   take_next(take_first) with trimmed/T-joint/self-loop/closing-loop
   handling (`path_length_along_contour_ccw`), vertical-arch
   preference (`take_vertical_prev`: prefer untrimmed, else longer).

ares adaptations needed: `first(cp)` = endpoint parity (idx & 1 ==
0, exists as path_index_for_intersection); `next_vertical`/
`prev_vertical` need the Up/Down direction classification added to
the graph build (upstream BoundaryInfillGraph::Direction from
infill line orientation); `append_full` replaces take_cw/ccw_full.

Next: `fill/base_support/connect.rs` implementing 2+3+5+6 on top of
the existing connect module + emit_loops_in_band, then 4c fills and
the emission wiring.

## 4b COMPLETE (`7785e12b` + `196d9565`); fills recipe locked (#211)

connect_base_support fully ported (fill suite 1289/1289), split into
connect.rs (358) + arches.rs (230).

`raft/fills.rs` recipe (FillSupportBase::fill_surface
`FillRectilinear.cpp:3610-3634` + make_fill_lines `:2920-2961` +
_infill_direction `FillBase.cpp:275-291`):
1. Per layer: angle per kind (fill_params.rs), spacing per layer
   (flange: first_layer_flow.spacing; others: support flow spacing),
   density per layer (flange: raft_first_layer_density·0.01; base:
   support_density; interface: raft_interface_density).
2. union(layer polygons) → expolygons; offset each by
   `overlap(0) − 0.5·spacing` (inner inset only; ares
   prepare_rectilinear_contours(expolygon, −angle, 0.0, −0.5·spacing)).
3. refpt = OBJECT bbox center (`set_bounding_box(bbox_object)` —
   SupportCommon.cpp:1454) rotated by −angle → grid anchor:
   align bbox.min to spacing via refpt; n_vlines = ceil(w/spacing);
   lines at bbox.min.x + i·spacing.
4. Emit ONLY OuterLow→OuterHigh pairs as 2-point vertical polylines
   (inner-hole intersections skipped, `:2948-2960`); x ∈ [bbox±0
   margin]; rotate back by angle.
5. connect_base_support(polylines, boundary=layer polygons as
   polygons_outer (the OFFSET outer contours!), bbox, spacing,
   density) — note: boundary_src = poly_with_offset.polygons_outer
   (the inset outer contours), NOT the raw layer polygons.
6. link_max_length = spacing·link_max_length_factor/density
   (factor 3? upstream FillParams::link_max_length default).

Next: implement fills.rs + emission wiring (5), then gate removal.

## #215 architectural finding: TWO pipelines, raft must land in the project one

ares has two slicing entry paths:
- `pipeline.rs` (model-based, `LayerPrintPaths` stream, hosts the
  `print_paths` support machinery incl.
  `apply_raft_expansion`) — NOT exercised by the KSR sweep.
- `project_slice.rs` (3mf project path — what the sweep/ksr tests
  use): closing → slicing → perimeters/infills as project entities →
  `gcode_emit/layers.rs` emission over islands (`Layer::lslices`).

Therefore: `raft/emit.rs`'s LayerPrintPaths transform targets the
wrong stream for sweep parity. The raft integration point is the
PROJECT pipeline's `gcode_emit` layer loop:
1. `bridge::build_raft_stream(options, first-layer lslices, scale)`
   is pipeline-agnostic ✓ (works as-is; contours adapter added).
2. gcode_emit needs: raft layers prepended to its layer schedule
   (absolute print_z from the grid; object layers shift z by
   `object_print_z_min` — check where gcode_emit derives print_z:
   likely `gcode_emit/layers/schedule.rs` + `entry.rs`), the raft
   fill polylines emitted as support extrusions (SupportMaterial /
   SupportMaterialInterface roles → `;TYPE:Support` /
   `;TYPE:Support interface` via features.rs:219 ✓), skirt extension
   for raft layers 0..skirt_height-1 around raft polygons.
3. The object layer z shift must also flow into the layer PLANNING
   (`project_slice/layers.rs` generate_layer_pairs start) — or be
   applied at the gcode_emit layer-entry boundary.

Remaining inventory (unchanged semantics, corrected target):
- 5a. gcode_emit raft layer schedule + z shift.
- 5b. raft fill emission as support entities (bridge plans →
  polylines → extrusion moves).
- 5c. skirt on raft layers.
- 5d. gate flip + oracle byte-parity loop (case-u7sdch).

## 5a integration surface mapped (#215 cont.)

`gcode_emit/layers/schedule.rs build()`: `per_object_z` accumulates
layer heights from 0 (`scan(0.0)`); `merged` = sorted
`(z, object, layer)` triples; the layer CONTENT comes from
`objects[object][layer]` (OrderedExtrusionLayer). Integration:
- `Schedule` gains `raft: Option<RaftSchedule>` (per-object raft grid
  z's + fill extrusions + object_print_z_min).
- `merged` gains raft entries `(raft_z, object, RAFT_LAYER_SENTINEL)`
  and object z's shift by `object_print_z_min`.
- The emit loop (layers.rs) must handle raft entries: emit the raft
  fills as extrusions (SupportMaterial roles → TYPE comments),
  first-layer speed semantics at raft flange (z == first raft z),
  skirt on raft layers 0..skirt_height, wipe/retract between raft
  regions (existing machinery driven by the layer-change seam).
- `per_object_z` shift: the accumulation `scan(0.0)` starts at
  `object_print_z_min` when raft is active (upstream
  `PrintObject::layers()` print_z carries the raft offset).

Next turn: implement RaftSchedule + merged-entry extension (5a
scope), then the emit-loop raft branch (5b).

## #216 remaining question for 5a implementation

`ObjectOptions` carries all raft fields (raft_layers/raft_expansion/
raft_first_layer_density/raft_first_layer_expansion/
raft_contact_distance/support_base_pattern_spacing/
support_interface_spacing — object_fields.rs:22-51) ✓. The project
side should call `parameters::slicing_parameters(settings,
&resolved.object, height, extruders)` directly (correct types, no
SliceOptions adapter).

OPEN: support/interface flow WIDTH sourcing in the project path
(upstream support_material_flow: explicit support width → else
line_width default). KSR needs 0.45 (line_width) — verify which
ProjectSettings field carries process line_width
(raw_settings/effective_config exploration pending). The width feeds
Flow::spacing (width − h·(1−π/4)) for both grid resolution and fill
spacing.

5a order: (1) resolve the width source, (2) build_project_raft in
bridge.rs (project types), (3) raft_schedule.rs (written once, was
ahead of the API — dropped), (4) layers.rs emit branch.

## 5b construction recipe locked (#219)

Raft fills integrate as ordinary fill entities (fills output scaled-µ
Polylines ✓ matching the fill machinery):
- `FillExtrusionPath { polyline, fitting: vec![] (regular fills are
  empty — fill_entities/grid.rs:50), role:
  ExtrusionRole::SupportMaterial/SupportMaterialInterface,
  mm3_per_mm: ordinary_volume(support_width, height), width:
  support_width (0.45), height }` — `Flow`/`ordinary_volume` from
  `perimeters/flow.rs:244+`.
- Wrap per raft layer: `OrderedExtrusionLayer { islands:
  [OrderedExtrusionIsland { entities: [Fill(FillExtrusionEntity::
  Path(...))] }] }` → `motion::emit_layer` handles travel/Z/TYPE/
  wipe/retract exactly like object layers (IslandPrintEntity enum,
  island_print_order.rs:14-29).
- layers.rs entry loop: `raft_schedule::classify(layer_index)` →
  Some(raft_index) → emit the pre-built raft OrderedExtrusionLayer
  with a raft EntryGeometry variant (chunk_slices = raft polygons,
  lower boundary empty, spacing = support flow spacing).
- first_group/boundary: for raft entries `previous_layer_z` comes
  from the raft grid (raft_index−1, 0 for index 0) — the
  `per_object_z[first_object].get(first_layer - 1)` lookup only
  covers object entries.

Next: implement the raft OrderedExtrusionLayer construction in
raft_schedule.rs + the layers.rs emit branch.

## #226 oracle convergence status (live diff running)

Fill pairing fixed (InnerLow→InnerHigh — ares' inner = upstream's
fill-boundary "outer"). Live: 19397 lines, raft fills on all layers.

Diff inventory vs case-u7sdch oracle:
1. SKIRT on raft layers missing in ares (oracle: 2 skirt loops
   98.494..121.506 on z=0.2, then again z=0.5) — 5c.
2. Flange pitch 0.374 (spacing from width 0.38 at first-layer
   height) ✓ plausible; oracle pitch TBD after skirt separation.
3. ares zig-zags directly line-to-line; oracle shows wipe/retract
   between fill groups (entity ordering + wipe behavior at the
   motion layer).
4. M73 timing (layer count change) + M106 first-layer fan gate.
5. Speeds: ares F600 flange vs oracle F2100 — support first-layer
   speed chain (initial_layer speed for SupportMaterial role).
6. flange = `fill_expolygons_with_sheath_generate_paths` with
   with_sheath = tree_support_wall_count > 0 = FALSE for KSR → plain
   path; the concentric rectangles in my earlier dump were the SKIRT
   loops, not a sheath. No sheath needed for KSR.

Next: 5c raft skirts (oracle loops at 98.494 = raft polys + skirt
distance), then speeds/M73/wipe convergence.

## #228 flange structure finding (oracle)

Oracle z=0.2 TYPE:Support opens with ONE FULL BOUNDARY LOOP of the
raft first-layer polygon (101.059..118.941, 45° corner cuts at
1.049), then the inner zig-zag. ares emits zig-zag only. Candidate
sources: connect_base_support's closing-loop / single-endpoint case
(take_next trimmed branch with cp1==cp2 → full contour loop), or the
flange taking the boundary arch between the outermost verticals.
Next session: trace which upstream branch emits the full-loop-first
structure (SupportCommon flange call chain + take_next), then mirror
in the ares port.

Skirt hull now uses fill points (was polygons); residual ~0.2mm
offset under investigation (hull span arithmetic: oracle hull ≈
19.43 vs fills 18.2 — unaccounted +1.2mm source TBD).

## #231 speed convergence analysis

Oracle flange F2100 = initial_layer_infill_speed 35 (KSR setting
confirmed; slow_down_layers=0 so the ramp is inert — the plain
layer-0 rule applies). features.rs `speed()` already returns
initial_layer_infill_speed at layer_index 0 ✓. ares emitted F600
(10mm/s) — the raft fill emission path is not reaching features::
speed with layer_index 0 (or hits a different default). Next:
trace motion.rs fill emission → features::for_fill(SupportMaterial)
→ speed(options, state.layer_index, ..) for the raft entities; the
boundary sets state.layer_index = raft_index ✓ so the suspect is
the fill emission's layer index source.

Grid-phase residual: contact span 14.30 vs oracle 14.371-0.07 band
— sub-cell; defer until speeds/line positions re-measured after the
speed fix (F affects nothing geometric).

## #232 speed CONVERGED; estimator/layer-count next

Flange F2100 ✓ (initial_layer_infill_speed 35 — the F600 was a stale
pre-sheath-fix reading). Sheath loop + fills now carry the correct
layer-0 speed chain.

Remaining oracle diffs (live):
1. `; total layer number: 50` vs 150 — the header layer count counts
   only object layers; the 100 raft layers missing (header source
   reads object records — needs the raft count added).
2. Estimator: 2m25s vs 31m47s — the M73 chain (R2 vs R31). Root
   suspects: (a) the header/estimator layer model missing raft
   layers (same source as #1), (b) raft fill E/time not reaching the
   estimator blocks.
3. M106 S255 vs S0 on the flange — first-layer fan-off rule not
   applying at the raft flange layer.
4. Extrusion segment count 5916 vs 9051 (~1.5×) — fill line count /
  line-position µ-residuals (contact span 14.30 vs 14.37 band).

## #235 estimator = downstream of fill density; flange spacing root cause

M73 频率(1437 vs 131)与时长(2m25 vs 31m47)都是估计器对行数
的正确响应：挤出段 5916 vs 9051（1.53×）× ~0.4s/段 ≈ 29min
差。根因是填充密度。

法兰 zigzag 间距: ares 0.375 vs oracle 0.419。
0.419 = spacing/0.9 → spacing = 0.377 = first-layer support flow
spacing with width **0.42** = KSR `initial_layer_line_width` ("0.42")
— not 0.38 (`support_line_width`). The flange flow follows the
FIRST-LAYER width chain (upstream first_layer_flow uses
first_layer_line_width fallback), same for the skirt WIDTH:0.42.

Fix: bridge `first_layer_flow_spacing` (and flange width_mm for the
sheath WIDTH comment) = initial_layer_line_width (0.42) — need the
first-layer width derivation: initial_layer_line_width > 0 else
line_width.

## #237 estimator root cause FOUND (block-level proof)

GT oracle processes the SAME ares gcode → 2726s (1.4× oracle 1907s
— consistent with the line-count gap). ares own estimate: 136s.
Block dump (ARES_DUMP_BLOCKS): the cumulative reaches **1891s
(≈oracle 1907s ✓ the main body is CORRECT)**, then **66 blocks with
NEGATIVE speed (−0.066)** drag it down to 136s. First bad block id
13612 starts at a modal `G1 F2400` line with dist 0.4879/speed
−0.066 — the modal-F cache is poisoned (−0.066 smells like an
E-value leak into the F slot). Downstream: M73 P1149 R0 artifacts
(percent>100 from the shrunken total).

Fix next: the modal-F/word parsing in estimate.rs (or the block
builder consuming a stale/negative feedrate) — then the estimator
converges 136s→1891s in one shot, collapsing M73 frequency, R
values, and the P>100 artifacts together.

## #238 negative-F SOURCE FOUND (emission, not estimator)

`grep F-` → exactly 2 lines: `G1 F-3.986` on the OBJECT first layer's
Skirt wipe sequence (z=30.3, `;TYPE:Skirt` → `G1 F-3.986` then
`G1 X.. E-36.67986 / E-75.169` huge negative wipe E). −3.986/60 =
−0.066 = the poisoning feedrate; the modal F stays negative until
the next F word → 66 negative-speed blocks ✓ fully consistent.

Fix: the skirt-wipe feedrate computation in the emission path
(cooling/wipe F derives from a ratio over the negative wipe E —
upstream uses the absolute value). NOT an estimator bug; the
estimator correctly mirrors whatever F the gcode carries.

## #244 chain-merge state (live, raw-diff 23,046 lines)

Connection merges pairs (line+arch) but breaks every OTHER arch:
right-boundary arches connect, left-boundary arches leave an F9000
travel (13/layer). Oracle = one continuous serpent per layer.
Suspect: the vertical-consumption loop takes one direction per pair
and the opposite-arch's endpoints are already consumed when reached —
likely the take_next trimmed branch appends (1e10 limited) instead of
merging polylines, or the loop ordering skips every second arc.
Next: probe the consumption loop on the 27-line square (CHAIN fixture
merges 26→14×4-pt: same every-other signature ✓ reproducible).

## #245 take_next walk-through (root cause narrowed to the same-chain arch)

Fixture indices: idx 2k = line-k BOTTOM, 2k+1 = line-k TOP.
- idx=0: next=2 (bottom arch) → take_next(0,true) merges line0+line1
  via take_full_arc; consumed {0,2}; chain lives in slot 0.
- idx=1 (line0 top): prev=3 (top arch) → take_next(3,false):
  cp1=1, cp2=3; resolve_merged(1)=resolve_merged(3)=**slot 0** (both
  ends already on the SAME chain) → the `polyline_idx1==polyline_idx2`
  trimmed branch fires → take_limited appends the top arch to the
  chain. This is EXACTLY upstream's same-chain/self-loop case
  (`:2405-2414`). If the append is a no-op (empty/direction guard),
  the chain misses every top arch → the observed every-other break.
Next: verify append_limited actually extends the chain in this branch
(assert the output len grows per arch; suspect the second take() on
the same slot returns default-empty and the arc lands on a lost vec).

## #248 verdict: connection comes from the cost-selection phase

ORDR probe closed the vertical-consumption question: the top arches
are prev_trim=true/len=0 (inside the tube) and upstream's SAME gate
skips them. The oracle's full serpent must come from the COST
selection phase (`:2462-2502`): for each unconsumed cp it evaluates
prev/next arch costs (`evaluate_support_arches` — the trimmed arch
walk uses take_cw/ccw_limited with not_taken), filters
cost_max >= cost_low && cost_min <= cost_high && rel_diff >= 0.25,
sorts by cost desc, and take_next()s them. On the KSR base layer the
arch cost = max deviation from the chord — a straight square edge
arch has cost 0 → cost_max(0) < cost_low → NOT selected either.

Remaining suspect: `base_support_extend_infill_lines` — the
contour-walk extension (0.33·spacing x-tolerance, 0.5·spacing
y-threshold) extends each line END along the boundary; on the square
the whole edge qualifies (dist_max_x=0 along the edge, dist_y=0.537
per arch step > threshold only across one arch) → line ends WALK the
entire edge, chaining the serpent via point_idx updates. THAT is the
likely serpent engine. Next: verify extend on the fixture (print
extend_next/extend_prev decisions).

## #249 oracle structure read: SINGLE continuous serpent CONFIRMED

The oracle z=0.5 Support section is ONE continuous polyline: entry
corner clip (117.482,102.712)→(117.288,102.518), bottom edge sweep,
corner, then pure horizontal zigzag X117.482↔102.518 with Y stepping
0.537, arch moves at each turn (E.02072 = arch, E.57729 = long line),
closing with the reverse corner clip at the top. **ZERO F9000
travels in the whole section** (only one M73). So the serpent is a
single polyline built by take()/take_limited during the consumption
phases — the take() helper (FillBase.cpp:531-576) chains pl1+arc+pl2
and the arcs at BOTH ends of each line pair up: the vertical
consumption loop's BOTTOM arch merges pair (n, n+1); when the loop
reaches the TOP endpoint of line n, the arc to line n+1's top is
UNTRIMMED (not_taken=16.6mm full edge) → take() path fires and the
chain grows. The ares break is that the loop's gate for the top
endpoints evaluates prev_trimmed (true) BEFORE the not_taken check —
but upstream's gate is `!cp.prev_trimmed || not_taken > min_arch`,
not_taken at the TOP endpoints = full edge 16.6mm >> 0.79 → passes!
=> ares' top endpoints report not_taken=0: the mark/extend phases
over-trimmed them. NEXT: why ares idx=1 prev_len=0 while upstream's
top-arch not_taken is the full edge — check mark's inside-loop
len_out test for the square (the corner vertices lie INSIDE the
tube, so the loop keeps 'inside' until the neighbor endpoint, and
the not_taken stays the full edge; ares' walk stops early or the
trim threshold differs).

## #249b root cause FOUND: prev-index swap in the take_next(prev, false) call

ORDR idx=1: prev_trim=true prev_len=0, but the gate is
`!prev_trimmed || not_taken_prev > min_arch` — prev_len=0 fails BOTH
→ skip. Upstream idx=1: cp.prev_on_contour=idx3, the top arch — BUT
upstream evaluates cp=idx1 whose prev_trimmed should ALSO be true...

Wait: idx=1 prev=Some("3") per the ARCH probe. Upstream calls
`take_next(*cp.prev_on_contour /* = idx3 */, false)` — the GATE is on
**cp** (idx1: prev_trim/len) but take_next receives **idx3** as its
`cp` → inside, cp1 = next_of(idx3) = idx1, cp2 = idx3. ares passes
`prev` (=idx3) ✓ same. So the gate values must match upstream —
unless upstream idx1's prev_trimmed is FALSE. ares' idx1 prev_trim
came from mark: the top arch (0.537mm) lies fully inside the tube of
line 0 (endpoints on the line's tube) → inside stays true → trim(0).
Upstream mark: same walk, same trim. BUT upstream's touching pass
(run BEFORE overlapping in build_working_graph) may have already set
not_taken differently, and crucially upstream's **arch order on the
contour** differs: the top edge runs X+ direction, endpoints sorted
ccw; ares' split_boundary_working_copy may order next/prev along the
OPPOSITE orientation, so what ares calls the 0.537 "prev" arch of
idx1 is upstream's 16.6mm "next" arch. Evidence: ares idx=1
next_len=16.6m (the FULL edge) — upstream idx1's NEXT should be the
0.537 arch and PREV the 16.6m. The prev/next orientation is
INVERTED in ares' graph for this contour. Fix: verify the split
orientation (ccw vs cw point insertion) in graph.rs vs upstream's
split_boundary_working_copy; likely flip.

## #250 orientation NOT inverted — trim lengths themselves match upstream

ORIENT probe: idx=1 prev=3 (x=0 top, ADJACENT), next=0 (own bottom) —
per the square's ccw order (top edge runs X+), idx1's contour
neighbors are idx3 (X=0) and... wait idx1=(-537000,top): next_on_contour
walking ccw = idx0 (-537000,bottom)?? No — the ccw point ORDER is
[(-7650601,-7650601) bottom-left → (-537000,bottom) → (0,bottom) →
(537000,bottom) → (7650601,-7650601) → right edge up → top edge X-
→ left edge down]. So the bottom hits come BEFORE the top hits; the
LAST bottom hit's ccw-next runs up the right edge, across the whole
top edge (all top hits in REVERSE X), down the left edge back to the
FIRST bottom hit. idx1 (first top hit in X) therefore has prev=3
(the X=0 top, 0.537 away) and next=0 (wrapping down the left edge,
16.6m away) — **ares' graph matches upstream exactly**. Not
inverted. Then upstream idx1's prev arc (0.537 to idx3) is the SAME
short top arch — upstream's gate `!prev_trimmed || 0.537*1e6 >
0.79*1e6` — 537000 < 789100 → upstream ALSO skips. UNLESS upstream's
prev_trimmed for idx1 is FALSE: upstream's mark_boundary_segments_
touching_infill (the FIRST trim pass, with clip 1.7·spacing /
colliding 0.8·spacing) trims boundary arcs COLLIDING with infill
tubes of OTHER lines. The 0.537 top arch lies in the tube of BOTH
end lines?? The arch is BETWEEN two adjacent lines' endpoints —
distance to either line = 0 (endpoints on the lines) but the arch
MIDPOINT is 0.5·0.537=0.27 from either line > tube radius 0.204 →
not fully inside either tube → NOT trimmed by the touching pass;
the overlapping pass (radius 0.5·(spacing+eps)=0.204, checks SEGMENT
ENDPOINTS inside the tube): endpoint = the OTHER intersection
point (on the neighbor line) → distance 0 → inside → walk
continues... loop until `closed_contour_distance(...) >=
not_taken_next` — the walk covers the whole 0.537 arch → inside
stays true → trim(0). Upstream same math → also trims. So how does
upstream connect? => the serpent must come from the LAST bottom
hit's NEXT direction: idx=52/54 (last bottom) next arc = up-right-
edge + across whole top + down-left = 16.6+16.6+16.6 ≈ 50m arch,
not trimmed, length >> min_arch → take_next(last_bottom, true) with
take_first=true → cp1=last bottom, cp2=first top — take() FULL arc
SWALLOWS the whole top edge including all top endpoints → one huge
polyline serpentine... then the bottom pairs chain into it. The
ares loop never reaches this because it takes SHORT arches first
(idx=0's 0.537 next) consuming endpoints so the long arch's cp2 is
already consumed. Upstream iterates in the same order... but
upstream's idx=0 gate: !next_trimmed(0.537 SHORT arch, next_trim
likely TRUE after touching pass) || 537000 > 789100 → skip → the
long arch survives. THE DELTA: ares idx0 next_trim=FALSE. Root
cause: the touching pass should trim the short bottom arches too.
