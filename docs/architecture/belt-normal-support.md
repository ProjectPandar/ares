# Normal support on a tilted belt

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
Upstream paths are relative to `src/libslic3r/`. Rust paths below are relative
to `crates/ares-core/src/project_slice/` unless stated otherwise.

## Public behavior and scope

`belt.rs::admit` currently rejects a belt project requesting support before
normal support generation runs. For an admitted automatic normal-grid case,
public project slicing and CLI export must instead produce G-code with the
support and interface moves Orca prints under the overhang. In particular the
inclined floor terminates support columns at different layers across the
belt, changes the extent and Z of the bottom contacts, and can leave layers
that print only support before the first object extrusion. These are emitted
motions and layer changes, not just an admission change or a config echo.

The initial slice admits `enable_support = true` with `normal(auto)`,
`support_style = grid` or `default`, a nondegenerate X/Y belt floor, no raft,
no active belt brim, no painted support facets, and no forced first support
layers (`enforce_support_layers = 0`). The current one-instance,
one-model-volume, one-filament, by-layer and other belt restrictions remain.
Keep rejecting requests outside this support subset; support-disabled projects
retain their current behavior. `Print.hpp:505–511` identifies support/raft
requests; `PrintObject.cpp:998–1002,5088–5117` dispatches normal support to
`PrintObjectSupportMaterial`; `Support/SupportMaterial.cpp:723–734` selects
Grid by default. Native rejects belt raft in `Print.cpp:2145–2149`.

## Rewrite boundary

1. **Run normal support in the local object slicing frame.**
   `PrintObjectSlice.cpp:912–921,1032–1041` records the floor lift and then
   applies the global belt Z correction to both the floor and object
   `print_z`. `PrintObject::_generate_support_material`
   (`PrintObject.cpp:5095–5127`) removes that correction before scheduling
   support and puts it back on installed support `print_z` after the
   generator returns. The normal generator anchors its height grid at
   the local first-layer height; a globally shifted negative top contact
   is not a valid input to that height grid.

   In `support_material/project.rs::generate`, use `plan.belt_slice`'s
   `print_z_offset` and `floor`, already derived in `belt.rs::BeltSlice::new`
   and applied to planned print heights in `raw_intersections.rs:200–204`.
   Give the normal support generator local planned print heights and a local
   floor (`floor.shift - print_z_offset`), with the same epsilon rule as the
   source. Preserve XY, `slice_z`, layer IDs and thicknesses. Do not mutate
   shared object layers. Fill and modulate in that local frame, then restore
   the offset once on the installed support layers' `print_z` before scheduling.
   The existing normal generator remains the owner of support geometry.

2. **Detect overhangs with the tilted lower-layer reference.**
   `PrintApply.cpp:1377–1386` derives effective `build_plate_tilt_x/y` from the
   absolute slice rotation (`BeltTransform.hpp:68–82`), already done by
   `crates/ares-core/src/project/effective_config/phases.rs:201–217`. Use those effective values,
   not the possibly decoupled G-code gantry angle. `Support/SupportCommon.cpp:2195–2199`
   uses `tan(deg2rad(clamp(tilt, -89, 89)))` and maps Y tilt to X slope and
   X tilt to Y slope. In `support_material/overhangs.rs`, through
   `grid_generation.rs`, translate the lower polygons by this slope times
   lower layer height once before region detection. Use the translated copy
   for the direct and expanded overhang differences and the expanded-result
   reclip; do not move the original lower slices used by sharp tails or
   bridge removal (`Support/SupportMaterial.cpp:1475–1478,1498–1560,1564–1603`).
   Leave non-belt detection unchanged in this slice.

3. **Use one `BeltFloorContext` for the complementary floor polygons.**
   Extend `belt/floor_context.rs`, shared with belt brim, with the half-plane
   polygon operations from `Support/BeltFloorContext.cpp:16–50,72–130` and
   `Support/BeltFloorContext.hpp:53–57`. Use the existing signed shear and
   local floor shift, with `belt_support_floor_offset` added once. The
   cutoff is `(print_z - z_shift - floor_offset) / shear`; the belt side
   is `u >= cutoff` for positive shear and `u <= cutoff` for negative shear,
   where `u` is slicing Y for X rotation and slicing X for Y rotation
   (`BeltTransform.cpp:90–112`). Use Native's scaled/truncated cutoff,
   1000 mm bounds and polygon winding. The complementary polygon is the
   valid side of the belt; keep the source epsilon distinctions between
   context activation and propagation. Do not substitute an XY bed box or
   change the existing brim formula.

4. **Terminate downward support at the floor, then trim the layer geometry.**
   Use `support_material/bottom_projection.rs` and `bottom_contacts.rs`
   for `SupportMaterial.cpp::bottom_contact_layers_and_layer_support_areas`
   and `detect_belt_floor_bottom_contacts` (`:653–682,2600–2674,2773–2778,2833–2927`).
   - For each object layer in the existing downward walk, keep the ordinary
     object contact first. Intersect the belt-side polygon at the current
     object `print_z` with the same raw projected contact input (general
     projection, or the enforcer projection in build-plate-only mode).
     An empty intersection does not produce a contact. For a nonempty
     intersection, use the bottom-interface flow height, base Z at the
     current object `print_z`, and top Z at base plus height plus the
     object-to-support gap, even for synchronized support layers. Use the
     existing bridging condition and width expansion, reuse the nonzero-gap
     snap to nearby top contacts and subtract the epsilon-expanded touching
     area from already generated support areas below the contact's top Z.
   - After grid projection and before the enforced/general area union,
     intersect both downward projections and both current-layer areas
     with the valid-side polygon (`:2897–2921`). Build-plate-only mode still
     clips general propagation; it does not manufacture a belt bottom
     interface for the general projection. Preserve the source's enqueue
     and reverse order for bottom contacts.
   - Subtract the belt-side polygon at **each support layer's own**
     `print_z` after trimming by the object: bottom contacts at the end of
     the projection, top contacts after intermediate height allocation, and
     base/intermediate layers before interface derivation
     (`:484–488,500–523,2924–2927,3299–3325`). Put these calls in
     `bottom_projection.rs` and `grid_generation.rs`, sharing the floor
     operation. Do not clip only the final paths: that would retain
     subfloor propagation and change where bottom interfaces are planned.

5. **Keep belt support out of flat-bed adhesion.**
   `Support/SupportCommon.cpp:448–456` sets the first base expansion to zero
   with an active belt floor, but leaves ordinary object/interface trimming
   intact. `:1921–1955` uses the ordinary base flow, density and fill even
   when the first support layer starts at local Z zero; it does not launch a
   dense flat-bed flange. Port these conditions in `support_material/adhesion.rs`,
   `first_layer.rs` and the fill selection in `toolpaths.rs` /
   `toolpath_cache/base.rs`. This covers the no-raft first-layer branch of
   `generate_raft_base`; it does not implement a belt raft. Reuse installed
   layers and ordinary support toolpaths (`Support/SupportMaterial.cpp:567–572,601–602`).

6. **Schedule and emit nonempty support layers as belt layers.**
   `GCode::collect_layers_to_print` pairs object and support by Z and the belt
   filter retains any entry with real object, support or apron extrusions
   (`GCode.cpp:2296–2388,2475–2484`). The count includes printed support Z,
   not empty support layers (`:3532–3555`). Extend the existing filter in
   `gcode_emit/layers/schedule/belt_brim.rs` and printed Z accounting in
   `gcode_emit/layers/schedule.rs`; currently the belt filter requires an
   object layer. Count the raw printed Zs before layer pairing averages
   nearby values.
   `GCode/ToolOrdering.cpp:1076–1099` also skips the below-object raft-gap rule
   for belts, even without a brim: mirror that branch in
   `gcode_emit/tool_ordering/partitions.rs`, not by dropping support below Z0.

   Reuse regular support emission (`GCode.cpp:7665–7705`) and the belt writer.
   In `gcode_emit/layers/boundary/belt.rs` and `gcode_emit/layers/support_entry.rs`,
   use the owning object's physical floor and the installed support Z for
   support paths, including support-only groups, rather than leaving stale
   object-layer band state. Native uses the support layer as the active
   layer while emitting and evaluates belt path height from that layer
   (`:7673–7697,11600–11640`). Keep the second-layer check's separate
   object-only behavior and its no-object fallback (`:6874–6879,11570–11597`).
   Do not apply the support floor offset to path speed or fan band height.

## Deferred

- Tree support (`Support/TreeModelVolumes.cpp:155–162`,
  `Support/TreeSupport3D.cpp:3689–3699,3992–3993`), belt raft, and other
  unsupported belt print modes. No changes to their generators.
- Manual or painted support, forced first layers and Snug support. Painted
  facets need belt-aware projection; the current
  `support_material/facet_projection.rs` projects through the ordinary object
  transform rather than the belt mesh transform. Other automatic/Grid
  projects still use their existing support options and detection pipeline.
- Belt brim with support. Native clips the same object's installed support
  fills out of the brim obstacles (`BeltBrim.cpp:386–429,566–567,587–591`);
  the Ares belt brim planner does not have that input. Leave the combination
  unsupported rather than emit overlapping support/brim paths.

## Observable verification

- Use a synthetic, connected, single-volume cantilever with a wide cap and
  a narrower stem, oriented so that some support projects down to the belt
  away from the stem. Package it using an existing belt project fixture's
  single-filament settings and a signed axis remap, with no brim or raft.
  Compare complete G-code from the CLI against pinned Orca in each reference
  mode. Require the reference to contain actual support/base and interface
  extrusion, not merely support tags or successful exit.
- Use the same solid with a translated instance, a nonzero floor offset and
  a Y-axis tilt, to exercise restoration of the global correction and the
  other clipping side. Choose fixture placement so at least one case has a
  printed support-only layer preceding the first object extrusion; verify
  its layer change is present and its raw Z is counted. Compare offset-on vs
  offset-zero using a case where pinned Orca changes support extrusion;
  do not infer the expected change from the Rust floor implementation.
- Include the build-plate-only branch and a support-disabled belt control.
  Keep negative admission coverage for tree support, raft, and active
  belt brim combined with support. Use the existing whole-G-code comparison
  without masking motion, timing, support roles or layer records.
- Require Official and Seeded comparisons, workspace nonparity, exact-tip
  static checks, and byte-identical G-code across core counts.
