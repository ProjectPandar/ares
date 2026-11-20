# Object layer ID for sparse infill anchoring

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) assigns `Layer::generate_sparse_infill_polylines_for_anchoring` an object-relative layer ID (`Fill/Fill.cpp:1561,1611`). Actual sparse infill emission uses the absolute layer ID (`Fill/Fill.cpp:1378`). Both paths use physical `print_z` (`:1383,1616`). Object layer IDs begin after the raft (`PrintObjectSlice.cpp:89`); Ares builds the same IDs in `project_slice/layers.rs:240`. In `project_slice/prepare_infill/bridge_over_infill/sparse_anchoring`, `layer_index` already equals the Native object-relative ID. Use that index for Triangles and Stars, and remove the redundant absolute ID from the shared dispatch arguments. The adaptive arm's separate absolute-ID lookup is deferred. Leave the emission path in `project_slice/fill_entities` unchanged.

## Pattern audit

Source paths in the Native column are relative to `src/libslic3r/Fill`. Ares arm locations refer to the anchoring dispatcher before normalization.

| Pattern | Native behavior | Ares anchoring before normalization |
| --- | --- | --- |
| Triangles | `FillRectilinear.cpp:3567–3584`: fixed sweeps at one line, trapezoidal type 1 at multiple lines; `:3134–3144,3422–3426` rotates the frame using `(id / thickness_layers) % 3`. | `arms.rs:118–130`: passes absolute ID to multiline geometry (`fill/triangles_trapezoidal.rs:26–32`). |
| TriHexagon (Stars) | `FillRectilinear.cpp:3586–3601`: shifted fixed sweeps at one line, trapezoidal type 2 at multiple lines; same rotation. | `arms_tail.rs:41–69`: passes absolute ID to multiline geometry (`fill/stars_trapezoidal.rs:28–34`). |
| Cubic | `FillRectilinear.cpp:3379–3386,3603–3618`: Z shift in both paths; trapezoidal type 3 also uses the same rotation. | `arms.rs:135–181`: already uses object index and physical Z. |
| Adaptive Cubic / Support Cubic | `FillAdaptive.cpp:1844–1859,1902`: sections the octree at physical Z; multiline selects `(id / max(thickness_layers, 1)) % 3`. | `arms.rs:297–319` passes absolute ID through `fill_entities/adaptive.rs:314–349` to `fill/adaptive/filler.rs:194`. Deferred: existing printed sparse differences on both one- and three-layer rafts prevent isolating the anchoring phase. |
| Grid | `FillRectilinear.cpp:3134,3176,3230,3440,3525–3545`: trapezoidal transposition and cap axis follow effective ID parity; ordinary Grid reverses odd-layer polylines. | `arms.rs:82–99`: already uses object index for both. |
| Quarter Cubic | `FillRectilinear.cpp:3620–3648`: Z-dependent shifts, no ID-dependent phase (`FillRectilinear.hpp:150` disables base alternation). | `arms_tail.rs:72–89`: physical Z only. |
| Lateral Lattice | `FillRectilinear.cpp:3550–3564`: Z-dependent offsets; reverses each polyline on odd IDs. | `arms_tail.rs:91–107`: physical Z, missing reversal. |
| Lateral Honeycomb | `FillRectilinear.cpp:3652–3709`: Z-dependent sweeps; reverses each polyline on odd IDs. | `arms_tail.rs:109–127`: physical Z, missing reversal. |
| TPMS-D | `FillTpmsD.cpp:121,128,139,152`: physical Z and its own angle, not the base alternating direction. | `arms_tail.rs:128–151`: physical Z only. |
| TPMS-FK | `FillTpmsFK.cpp:136,143–145,157–159`: physical Z and its own angle; the forwarded base direction does not set the field phase. | `arms_tail.rs:152–179`: physical Z only. |
| Lightning | `FillLightning.cpp:18–32`: tree lookup by object layer ID. | `arms_tail.rs:180–190` and `arms_tail/lightning.rs:31`: already uses object index. |
| Honeycomb | `FillBase.cpp:454–458`, `FillHoneycomb.hpp:60`, `FillHoneycomb.cpp:54,81`: effective ID modulo 3 sets the direction when alternation is enabled. | `arms.rs:323–325` and `honeycomb.rs:30–31`: already uses object index. |
| 3D Honeycomb | `Fill3DHoneycomb.cpp:336–345,389–394`: ignores base direction; own angle, physical Z, layer height and thickness. | `arms.rs:200–228`: Z and layer height, no ID phase. |
| Gyroid | `FillGyroid.cpp:319–328,349,370–373`: physical Z and its own angle, ignores base alternating direction. | `arms.rs:182–199`: physical Z only. |
| Cross Hatch | `FillCrossHatch.cpp:186–211`: physical Z and its own angle, ignores base alternating direction. | `arms.rs:49–68`: physical Z only. |
| Line and linear family | `FillBase.cpp:454–458`, `FillBase.hpp:244`, `FillLine.cpp:30`, `FillRectilinear.cpp:2764`: base direction can alternate by effective ID; some patterns disable alternation. | `arms.rs:230–294`: already uses object index. |
| Concentric and plane paths | `FillConcentric.cpp:27–115` does not use base direction; `FillPlanePath.hpp:56` disables ID alternation. | `arms.rs:326–357`: no ID phase. |
| Other tail arms | Not sparse anchoring pattern generators. | `arms_tail.rs:191–194`: invariant checks only. |

## Observable behavior

Native constructs bridge anchors before combining sparse infill (`PrintObject.cpp:789–793,3381,5075`; `Surface.hpp:54`). Ares uses the same ordering (`project_slice.rs:212–214`, `region_slices.rs:103`). A three-layer raft therefore leaves the modulo-three phase unchanged for these unit-thickness anchoring surfaces; use it as a control, not as a phase discriminator. A one- or two-layer raft changes that phase. The missing Lateral reversal is a separate parity dependency; change it only if an observable bridge difference isolates the reversal.

Compare pinned Native and the public slice output for a synthetic closed plateau, with normal raft and arc fitting disabled, on X1C and K1. Observe internal bridge motion separately from printed sparse infill so unrelated motion differences do not establish an anchoring regression. The correction must not change the printed pattern's layer-ID contract. No changes to fill geometry algorithms, raft placement, infill combination, or unrelated pattern dispatch are included.
