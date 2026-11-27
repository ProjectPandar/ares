# Tree styles and model landings on a tilted belt

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
Upstream paths are relative to `src/libslic3r/`; Rust paths are relative to
`crates/ares-core/src/project_slice/` unless noted otherwise. This follows
[Organic belt support](belt-tree-support.md) but ports a different generator.

## Style dispatch and scope

`PrintObject.cpp:5088–5094` dispatches tree support to `TreeSupport`.
`Support/SupportParameters.hpp:208–221` resolves tree Default to Organic.
`Support/TreeSupport.cpp:1901–1955` sends Organic to `generate_tree_support_3D`;
Slim, Strong and Hybrid share the contact-point, height planning, node descent,
area drawing and toolpath stages in `TreeSupport`. Native does not substitute
one generator for the other on belts.

Implement in this order:

1. Admit explicit Slim, Strong and Hybrid for automatic, build-plate-only
   belt trees. Port the shared floor and first-layer rules to the existing
   non-Organic generator, including Hybrid polygon nodes.
2. Admit automatic Slim with `support_on_build_plate_only = false` using the
   separately witnessed model-contact behavior. Keep the belt floor active
   simultaneously; model landing is permission, not a different build plane.
3. Keep Organic/Default and Strong/Hybrid model landing deferred until each
   generator/style has a paired Native witness that isolates actual model
   landing. The Organic and non-Organic contact policies are not interchangeable.

The admission boundary is `belt.rs::automatic_support`, called by `admit`.
Keep the existing restrictions on rotation, nondegenerate floor,
raft, brim, painted or forced support, number of objects, volumes and
filaments, and layer scheduling. Do not admit manual support, add a fallback
for mismatched type/style, or alter normal support admission. Native rejects
raft on belts (`Print.cpp:2145–2149`).

## Build-plate-only non-Organic trees

The flat-bed port already resolves the style (`support_material/style.rs`),
plans heights (`support_material/tree/non_organic/heights.rs`), samples tree
contacts (`non_organic/contacts.rs`), and builds and fills node areas
(`non_organic/drawing.rs` and `non_organic/toolpaths.rs`). Below, paths beginning
`non_organic/` are under `support_material/tree/`. Preserve the style policy:

| Style | Native behavior already represented in Ares |
| --- | --- |
| Slim | `PrintConfig.hpp:381–389` and `TreeSupport.cpp:2872–2876,3253–3285` select the Slim movement policy without the non-Slim 5 mm near-tip hold. The active drop path computes radius from millimetres to the tip (`:1977–2004`); do not substitute the adjacent layer-count radius overload. Preserve `non_organic/drop_settings.rs` and `neighbor_movement.rs`. |
| Strong | `TreeSupport.cpp:687–697,3261–3285,3315–3328` sets the Strong flag and bypasses selected neighbour-centre and tip-directed adjustments. Preserve the Strong cases in `non_organic/neighbor_movement.rs` rather than changing floor math or imposing a minimum radius. |
| Hybrid | `TreeSupport.cpp:3782–3805` separates a sufficiently large non-sharp-tail overhang into circle and polygon roots; both the original and residual area must exceed `sqr(scale_(10))` (`SupportParameters.hpp:331`). Polygon nodes do not merge or steer during descent (`TreeSupport.cpp:3191–3218`). Polygon-derived areas request normal infill when the base pattern is Default (`:686–697,2563–2569`). Keep `non_organic/contacts.rs`, `dropping.rs`, `drawing/{areas,groups}.rs` and `toolpaths/base.rs` as the owners. |

Port these belt-specific differences:

1. **Pair global heights with the global floor.**
   `PrintObjectSlice.cpp:1034–1041` applies global Z to both object layers and
   the floor shift. `TreeSupport::plan_layer_heights`
   (`TreeSupport.cpp:3523–3610`) consumes those object heights;
   `PrintObject.cpp:5130–5144` applies no additional shift to the installed
   non-Organic layers. In `support_material/project.rs` and
   `support_material/project/{frame,tree_input}.rs`, pass `plan.layers` and
   the global `belt/floor_context.rs::BeltFloorContext` to this path. Apply the
   configured floor offset once; do not use the local context or `restore`
   used by normal and Organic support. Reuse `non_organic/generation.rs` for
   dispatch and `heights.rs` for synchronized or independent support heights.

2. **Detect overhangs with the signed slice tilt.** Use the existing
   `support_material/tree/overhangs/first_pass.rs::detect_layer` floor input
   in `non_organic/generation.rs`. `TreeSupport::detect_overhangs`
   (`TreeSupport.cpp:777–786,925–1021`) shifts the lower extrudable slice by
   `-lower_height * shear` and adds the belt half-plane sampled at current
   layer bottom, restricted to the existing local window. Keep the unshifted
   model outlines for later bridge finalization, collision and avoidance.
   `TreeSupportData` explicitly excludes the floor from these outlines
   (`TreeSupport.cpp:3915–3943`); adding it would push branches along the belt
   instead of letting them stop on it. Keep the sampling box tied to Native's
   centered `PrintObject::bounding_box()` (`TreeSupport.cpp:3668–3691`,
   `PrintObject.cpp:143–162`, `Print.hpp:403`), not the floor or bed bounds.

3. **End node descent at the belt.** In `non_organic/contacts.rs`, reject
   an automatic candidate at or below `floor_z(candidate)` before creating
   its virtual node (`TreeSupport.cpp:3754–3775`). Use `belt/floor_context.rs`
   for the point height and signed shear. `TreeSupport::drop_nodes`
   (`TreeSupport.cpp:3105–3129,3191–3218,3351–3369`) stops a single or merged
   circle once the proposed support `print_z` is at or below
   `floor_z(center) - abs(shear) * max(0, radius)`; use the larger current
   radius for a merged pair. For Hybrid, test each polygon remaining after
   the next model collision subtraction against the **minimum** floor height
   on its outer contour. Do not create a child for a landed footprint.
   Preserve the parent `to_buildplate = false` transition and merge validity
   ordering without introducing a model-collision prune for a belt landing.
   The leaf predicates are `TreeSupport.cpp:711–736`; the Rust descent owners
   are `non_organic/{drop_nodes,merging,dropping}.rs`.

4. **Clip the drawn areas before model contact bands and area grouping.**
   At each planned support `print_z`, subtract the global belt half-plane
   from base, roof, first roof, floor and roof-gap areas after normal bed,
   object and roof clipping (`TreeSupport::draw_circles`,
   `TreeSupport.cpp:2353–2397`). Apply at the corresponding boundary in
   `non_organic/drawing.rs`, before base simplification and
   `drawing/bottom_interfaces.rs`. Retain the polygon and wall source areas
   used to classify each surviving group; do not infer Hybrid classification
   from the clipped shape or clip only after toolpath generation.

5. **Do not build a flat-bed flange on the belt.** Skip layer-zero circle
   brim expansion and Hybrid polygon-only expansion in
   `non_organic/drawing/{node_shapes,areas}.rs` and `drawing.rs`
   (`TreeSupport.cpp:2304–2312,2526–2548`). Keep the separate first object
   layer XY collision clearance (`:2241–2252`). Use ordinary tree base flow,
   density and fill and ordinary interface filling at support layer zero
   (`TreeSupport.cpp:1705–1718,1775–1819`), not the first-layer brim walls
   or dense sheath. Rust destinations are
   `non_organic/toolpaths/{base,interfaces}.rs`. The equivalent common
   Organic/normal first-layer checks are in `Support/SupportCommon.cpp:448–456,
   1921–1955`; do not route non-Organic toolpaths through that shared filler.
   Reuse the existing belt export and support-only layer scheduling.

## Model landings

**Slim.** With build-plate-only disabled, `TreeSupport::drop_nodes` retains
model-reachable node groups that the build-plate-only case prunes. Preserve
its existing model collision and current-layer contact behavior
(`TreeSupport.cpp:2980–3029,3220–3239,3359–3369`;
`non_organic/{grouping,dropping,pruning}.rs`). After belt clipping, keep the
flat-bed `draw_circles` model policy: it can merge circle roofs with the
original overhang and derive a bottom gap and floor interface from actual
intersecting typed model surfaces (`TreeSupport.cpp:2315–2326,2408–2524`).
`non_organic/drawing/{areas,bottom_interfaces,trimming}.rs` already implements
these steps. Search the typed top and bottom surfaces without a gap-sized
window, then use the real contact `print_z` in the same global frame as the
support layer to clear only overlapping gap areas and select the support
layers in the configured bottom interface band. A tilted floor crossing is
not a typed model surface; do not use its height or the flat first-layer gap
as a bottom-interface anchor. Keep final model clearance and fill-role
selection (`TreeSupport.cpp:2551–2561,1766–1776`).

**Organic/Default (deferred).** `TreeSupport3D` selects a model-reaching branch
through model avoidance and placeable areas, then searches its influence
area for a suitable model root (`Support/TreeSupport3D.cpp:403–419,1687–1768,
2638–2670,2774–2784`; `Support/TreeModelVolumes.cpp:176–180,673–765,785–811`).
The Rust owners are `support_material/tree/model_volumes/{avoidance,placeable}.rs`,
`tree/organic/{single_area,nodes/model_contact}.rs` and
`tree/organic/drawing/contacts.rs`. These already implement the flat-bed
policy. On belts the floor remains a separate local-Z cache. A gracious
model root can still emit bottom contacts from the trimmed front slice;
non-gracious belt descent does not get the flat-bed bottom pad or retention
fallback (`TreeSupport3D.cpp:3984–4038,4116–4163`). A future slice must preserve
that distinction; simply enabling the option or restoring every bottom pad
would not implement model landings.

## Public-API witnesses

- **Circle styles.** Use the translated 30° X cantilever from the Organic
  slice, changing only `support_style`. Both Native references exited
  successfully and agreed on ordered positive-extrusion support motion.
  Default and Organic matched (38,349 base moves); Slim and Strong each
  differed from Default and from each other (9,222 and 8,653). Assert nonempty
  base support and distinct motion, not those counts.
- **Hybrid.** The ordinary cantilever produces identical Hybrid and Slim
  support motions. Use a cantilever with the underside parallel to the
  slicing planes to give one layer enough uninterrupted overhang for a
  polygon node. Keep the base pattern identical so it cannot explain the
  style difference. Both references emitted base moves for aligned Slim and
  Hybrid, and Hybrid changed when only `belt_support_floor_offset` changed
  from 0 to +1 mm. Assert both differences and nonempty support; do not
  require a difference in roof motion when only the floor is raised.
- **Slim model landing.** Add a connected 3 mm shelf under the left wing of
  the translated X cantilever and leave the right wing clear. Hold the bottom
  Z gap at 0.2 mm and request two bottom interface layers; change only
  `support_on_build_plate_only`. Both references emitted interface moves at
  shelf-side X and just above shelf height only with model landing enabled
  (587 moves versus zero in this window). Test positive-E interface moves in
  that region, not just a global interface count: roof interfaces exist in
  the plate-only control too. Evaluate model height in the configured belt
  coordinates (`GCode/MachineFrameTransform.cpp:20–57`). Require the right
  wing to retain positive-E base support in both cases. An ordinary and a
  slice-aligned Organic shelf probe changed support motion but added only
  near-belt interfaces, so neither qualifies as an Organic landing witness.

Run paired Official and Seeded comparisons against both the CLI and public
Rust slice API. Compare complete ordered G-code bytes, removing only the
volatile generator line. Retain existing normal, Organic, Y-tilt and flat-bed
tree controls, including unsupported combinations still outside the slice.
Require no selected skips or new failures in the affected support, overhang,
perimeter, bridge and belt cases. No change to the printer transform,
raft/support annotations, or the public support option vocabulary is part of
these slices.
