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
