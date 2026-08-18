# Spec: KSR FDM Test V4 task223 internal-region retraction containment

## Observable contract

When `reduce_infill_retraction` is enabled and sparse infill is present, a travel between non-perimeter extrusion paths skips retraction only when the complete travel segment lies inside an original internal layer-region slice. Fill-preparation surfaces are not the containment boundary because shell promotion, fill spacing, and no-overlap processing alter them after slicing.

A travel leaving an external or overhang perimeter remains retractable even when its segment lies inside an internal slice. A travel whose destination is a perimeter also remains retractable. The decision is derived from typed project options, generated layer-region geometry, and extrusion roles; it does not depend on fixture identity or reference G-code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:7358-7364` and `7523-7645`, plus `src/libslic3r/GCode/RetractWhenCrossingPerimeters.cpp:9-53`. Orca caches original `LayerRegion::get_slices()` internal surfaces and suppresses retraction only for contained non-perimeter travel, while forcing retraction when departing a non-internal perimeter. The Rust destinations are `crates/ares-core/src/project_slice/island_print_order.rs::internal_surfaces` and `crates/ares-core/src/project_slice/gcode_emit/motion/path.rs::can_skip_retraction`.

Included: original region-slice selection, complete-segment containment, destination-perimeter gating, and external/overhang departure gating. Deferred: remaining travel/retraction sequence differences, avoidance-path generation, wipe-path parity, arc/numeric differences, cooling, timing/M73, and later normalized G-code differences.
