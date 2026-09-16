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
