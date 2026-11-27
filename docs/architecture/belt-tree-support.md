# Organic tree support on a tilted belt

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
Upstream paths are relative to `src/libslic3r/`; Rust paths below are relative
to `crates/ares-core/src/project_slice/`.

## Public behavior and style dispatch

When an automatic organic tree is requested on a belt, slicing should print
branches under real overhangs and terminate those branches on the inclined belt.
The result is observable as positive-extrusion support motions: shifting the
support floor changes their paths, not just a setting in the G-code header.

`PrintObject.cpp:998–1002,5088–5094` dispatches tree support to `TreeSupport`.
`Support/SupportParameters.hpp:208–221` resolves explicit `organic` and tree
`default` to Organic. Grid and Snug on a tree request also fall back to default;
that mismatched style/type combination is not part of this slice.
`Support/TreeSupport.cpp:1901–1954` dispatches Organic to `generate_tree_support_3D`
(`Support/TreeSupport3D.cpp:4315–4329`) and runs other tree styles through its
own node, circle and toolpath pipeline. It has no belt style switch. Native
therefore has two belt-aware tree paths, not one floor algorithm for all styles:

| Native style | Belt-specific behavior |
| --- | --- |
| Organic, including default | Shared smart-overhang detection accounts for the shifted lower layer and belt surface. Separate local-Z floor polygons clip branch slices and end root descent without becoming collision/avoidance obstacles. The final generator layers are clipped again before toolpaths. Organic `print_z` receives global belt Z only after installation (`TreeSupport.cpp:777–786,925–1005`; `TreeModelVolumes.cpp:124–129,148–171,576–577`; `TreeSupport3D.cpp:3984–4015,4038–4163,3674–3703`; `PrintObject.cpp:5130–5144`). |
| Slim, Strong, Hybrid | The same smart-overhang detection feeds the non-organic path. Slim changes node sizing; Strong selects stronger branches; Hybrid can use polygon nodes for large overhangs. The plane is not added to the collision/avoidance outlines. Contacts at or below the floor are rejected; a descending node stops when the entire circle or polygon is in the belt. Areas are clipped in global Z before grouping, and layer-zero flat-bed adhesion is suppressed. Node heights already derive from shifted object layers (`TreeSupport.cpp:687–695,711–736,1661–1668,1700–1708,1775–1804,2304–2312,2377–2397,3109–3118,3198–3205,3351–3360,3523–3656,3754–3759,3784–3790,3922–3940`; `PrintObject.cpp:5130–5144`). |

Admit **one generator**: `enable_support = true`, `support_type = tree(auto)`,
`support_style = organic` or `default`, and `support_on_build_plate_only = true`.
Require an active nondegenerate X/Y belt floor, no raft, no active belt brim,
no painted support facets, and no forced first support layers. Keep existing
one-instance, one-model-volume, one-filament, by-layer and other belt restrictions.
Native rejects belt raft (`Print.cpp:2145–2149`). Keep the normal support admission
from [normal belt support](belt-normal-support.md) independent. In `belt.rs::admit`,
reject tree requests outside this subset; do not map them to normal grid or change
flat-bed style resolution. Use the existing `support_material/style.rs` resolution
for the admitted alias and explicit style.

## Rewrite boundary

1. **Detect organic overhangs against the signed belt reference.**
   `TreeSupport3D.cpp:3526–3540` consumes `TreeSupport::detect_overhangs`.
   Give `support_material/tree/overhangs/first_pass.rs` the belt inputs from
   `support_material/project.rs` through the organic generator. Translate a
   copy of the lower extrudable polygons on the floor axis by truncated scaled
   `-lower_height * shear`, using the signed slice shear, not the magnitude of
   `build_plate_tilt_x/y` or the decoupled machine-frame angle. Sample the belt
   polygon at current layer **bottom** (`print_z - height`), restrict it to the
   union bounds of the current and original lower polygons expanded by 10 mm,
   then union it into the translated lower copy (`TreeSupport.cpp:925–986`).
   Feed this effective lower geometry to the normal offset/difference and
   cantilever calculation, and to the floating sharp-tail check
   (`:983–1021` and its existing continuation). Do not globally translate object
   contours, replace the original lower slices used by the later bridge pass,
   or skip a whole layer because one island touches the belt. Keep the existing
   flat-bed smart-overhang behavior.

2. **Keep organic layer generation local and the floor out of avoidance.**
   Reuse `belt/floor_context.rs` from normal belt support; do not duplicate its
   signed half-plane or apply the support floor offset twice
   (`Support/BeltFloorContext.cpp:16–38,53–123`). Use `BeltSlice::print_z_offset`
   and the existing belt floor from `belt.rs`: the organic context subtracts
   the global offset from the floor shift, and per-object-layer cache entries
   use `planned.print_z - print_z_offset`. In overhang detection, pair global
   object heights with the global floor, or subtract the same offset from
   both; the half-plane must not mix frames. Preserve the organic generator's
   own height grid for generated layer and mesh slice Z. In
   `support_material/tree/model_volumes.rs`, cache the per-layer belt-side
   polygons separately in the same indices as the no-raft object outlines.
   An active cache is distinct from an empty polygon on one layer. Do not add
   it to blockers, model collision, placeable areas, avoidance or smoothing:
   the belt is a branch landing, not an obstacle to slide down
   (`TreeModelVolumes.cpp:124–171,576–577`; `TreeSupport3D.cpp:3966–3975`).
   Continue using the existing machine-border projection
   (`TreeSupport.cpp:698–703`; `TreeSupport3D.cpp:4324–4329`;
   `support_material/project/bed.rs`).

3. **End branches on the floor during drawing.**
   In `support_material/tree/organic/drawing/mesh_slices.rs`, subtract the
   cached belt polygon at `begin + slice_index` after model and bed clipping,
   before removing tiny fragments. Drop an all-empty branch; otherwise remove
   leading empty slices, advance `begin`, and retain the trimmed first slice
   as the contact reference. Pass the information that slices were removed to
   `drawing/contacts.rs`: skip non-gracious extension when leading slices were
   removed on a belt. Otherwise retain the existing bounded root descent, but
   do not extend a `verylost` branch to layer zero on a belt. After collision
   trimming at each candidate descent layer, subtract that layer's floor.
   Preserve the minimum-area stop; keep the surviving slice and stop descent
   when the floor reduces its area by more than the tiny-area threshold. A
   nonempty half-plane that misses the branch is not a contact. Suppress the
   flat-bed bottom-contact pad and model fallback in the corresponding belt
   branches; do not introduce a special belt interface pad
   (`TreeSupport3D.cpp:3984–4015,4038–4120,4144–4163`).

4. **Clip derived layers and emit in the installed frame.**
   Organic branch clipping is not enough: base and interface derivation can
   produce additional polygons. In `support_material/tree/organic/generation.rs`
   and its sibling `output.rs`, preserve interface derivation and layer
   ownership, then subtract the local belt half-plane from
   every nonempty generated layer after the bed intersection, computing the
   plane from that layer's actual local `print_z`, not an object-layer index.
   Do this before toolpath fill (`TreeSupport3D.cpp:3640–3642,3670–3703`). Reuse
   the belt conditions in the normal slice's shared `adhesion.rs`,
   `first_layer.rs` and support toolpath fill: no flat-bed first-layer base
   inflation or dense flange on an active belt, even if a local base layer
   starts at Z zero (`Support/SupportCommon.cpp:448–456,1921–1955`).
   In `support_material/project.rs`, restore the global Z offset exactly once
   to installed organic support `print_z` after local toolpaths are built
   (`PrintObject.cpp:5130–5144`). Reuse the normal slice's belt layer scheduling,
   support-only group handling, raw printed Z count and belt writer
   (`GCode.cpp:2296–2388,2475–2484,3532–3555,7665–7705`). Do not move XY or
   apply `belt_support_floor_offset` to the emission band or printer transform.

## Deferred

- Slim, Strong and Hybrid (global-Z node landing and `draw_circles` path);
  tree-on-model support and its placeable/model-contact policy; manual or painted
  support, support volumes, forced first layers, raft, and belt brim with tree.
- New interface algorithms, tree radius/merge policy, wall/roof sampling,
  bed collision, G-code transforms, and multi-object or multi-material belt
  behavior. Existing organic roof generation is reused, including clipping its
  output when present; this is not a new roof-generation slice.

## Synthetic fixture and observable verification

Use a watertight asymmetric T cantilever based on the single-filament
`tests/parity/mesh-edge-pairing/cube-control.3mf` container. The stem spans
X = −4 to 4 mm and Z = 0 to 8 mm; the joined cap spans X = −16 to 17 mm and
Z = 8 to 13 mm. Extrude the whole cross section from Y = −6 to 6 mm and use
one instance translated to (110, 146, 0) mm. Use coupled 30° X rotation,
`rev_x / pos_z / pos_y` remaps, automatic explicit Organic, build-plate-only,
30° support threshold, two configured top interface layers, and no skirt,
arc fitting, raft or brim. Compare the same fixture with support floor offset
0 mm and +1 mm, changing only that setting. The translated instance exercises
the global floor/layer correction.

Pinned Native Official and Seeded (replay seed 0) both produced **38,349**
positive-relative-E support moves with an XYZ word at 0 mm, and **36,374** at
+1 mm. The support move bytes change within each reference mode, independent
of the config block. Both Native lanes completed and neither fixture needs a
printed `Support interface` role to demonstrate belt contact. These counts are
observations, not committed golden outputs or fitted expected values.

Compare whole G-code from the public project-slice and CLI paths against the
pinned Native references in both modes, using the existing parity normalization.
Assert real support extrusion on the reference and confirm the raised floor
changes support motions, not just the serialized setting. Cover a translated
Y-tilt case with the opposite floor side and the default-to-Organic alias;
keep support-disabled belt, normal belt support, and the deferred tree styles
as controls. Preserve exact-tip static checks, paired Official/Seeded parity,
workspace nonparity, and byte-identical G-code across core counts.
