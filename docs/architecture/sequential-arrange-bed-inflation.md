# Sequential arrangement bed inflation

## Upstream boundary

Port `8386727e1d6002c1ef71b5b421a079a22aad8ec6` from OrcaSlicer `40ecc66f7`:

- `src/libslic3r/Arrange.cpp::update_arrange_params`, starting at line 150;
- `update_selected_items_inflation`, starting at line 160;
- `update_unselected_items_inflation`, starting at line 228;
- `remove_items_outside_bin`, lines 1110–1124, and its `_arrange` call.

The Rust destinations are `crates/ares-core/src/arrangement/params.rs` and `arrangement/arrange.rs`. Their beta parameter updates and complete-containment filter are incomplete representations of these upstream functions. Replace those parts in place; do not introduce another arrangement pipeline. The surrounding plater session remains a compatibility shell for the separately cited GUI and CLI preparation boundaries.

## Included behavior

1. Add skirt room without expanding the sequential bed in `update_arrange_params`.
2. For an all-short sequential selection, enforce `max(MAX_OUTER_NOZZLE_DIAMETER, 2 * object_skirt_offset) + 0.001` as the minimum spacing, preserving an already larger distance. Keep the tall-object clearance-radius rule.
3. Give bed margin back before expanding the bed. For sequential objects, measure the square-join inflated contour after its existing rotation, subtract that inflation from the measured size and add scaled `EPSILON` for rounding. Preserve the source's original-contour case when the offset produces no contour. Keep skirt room and preserve the non-sequential size calculation.
4. Expand the sequential bed by half the resulting minimum object distance, using the upstream `unscale<float>` conversion (`float(distance) * float(SCALING_FACTOR)`). Do not use half the extruder clearance radius.
5. Derive exclusion spacing from that same expansion. Move virtual exclusion regions only along axes where the bed actually expands; a positive bed margin must not move them.
6. Keep fixed items whose transformed inflated bounding boxes touch or intersect the bin's bounding box. Remove only wholly disjoint non-virtual items. Virtual items remain. This is the source's bounding-box predicate, not complete shape containment or polygon clipping.

The existing square-join offset, integer polygon rotation, geometry bounds and nester remain the geometry owners. Inflation used to separate objects is not printable object geometry and must not consume bed margin.

## Deferred behavior

No changes to item selection, plate locking, brim/skirt generation, wipe-tower generation, nozzle-area mapping, optimization objectives, packing heuristics, cooling, slicing or configuration identity are included. The GUI/CLI distinction in preparing physical fixed items is not redesigned. No new public API or platform adapter is needed.

## Failure modes

Expanding by clearance radius places short objects outside the physical bed. A half-nozzle spacing floor can fail sequential clearance. Reducing margin before accounting for the actual rotated square-join outline can leave a turned object unpackable. Measuring inter-object inflation as object size returns the wrong margin; omitting the scaled epsilon can reject a bed-filling object after rounding. Moving virtual exclusions while the bed shrinks changes their physical location. Discarding partially overlapping fixed items frees occupied space, including items whose inflation crosses the bed edge. Converting scaled distance in double precision instead of the source's float arithmetic can change exported translations by sub-micron amounts.

## Behavioral verification

Use public `PlaterSession::load` and `arrange`, compared with original OrcaSlicer's CLI arrange and exported 3MF transforms and plate assignments. Retain the input archive, Native export and both arrangement views as repeatable artifacts outside the source tree. Do not change the existing comparator or store Native goldens.

Add cases before changing production code: all-short cubes using nozzle clearance, crowded short objects near the bed edge, a short selection beside a bed-exclusion volume while the margin stays positive, a turned near-bed-filling object, a bed-filling straight object, and a by-layer spacing control. Use the existing three-instance cube fixture with explicit configuration and transform edits. Preserve the previously failing full-height bed-exclusion-volume comparison and adjacent by-layer/current-plate Native controls. Compare both Native modes with complete producer bindings.

The public session cases exercise the arrangement preparation, parameter updates, nester, final transforms and plate mapping together. The fixed-item retention rule is additionally checked against the cited source predicate; do not construct a private-kernel-only test seam or invent a GUI selection feature to reach it.

The rewrite remains rendering-neutral and independent of filesystem, terminal, platform runtime and core count, preserving browser WASM, Windows, macOS and Linux boundaries.
