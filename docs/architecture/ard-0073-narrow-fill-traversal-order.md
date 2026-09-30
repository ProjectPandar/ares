# ARD-0073: Narrow-fill traversal order

Upstream boundary (8500fcdcc): `Fill/FillConcentricInternal.cpp:48-96`
finalizes thick polylines, calls `ShortestPath.hpp::reorder_by_shortest_traverse`
and preserves them in a no-sort collection. `ShortestPath.cpp::chain_points`
and `GCode.cpp::extrude_infill` own producer and hierarchical consumer order;
`ExtrusionEntityCollection.hpp::chained_path_from` respects `no_sort`.

Rust destination: `project_slice/fill_entities/concentric.rs`,
its `concentric/generate.rs` source parameter construction,
`perimeters/classic/shortest_path` and the existing `gcode_emit/motion.rs`
infill-consumer shell. Locate a source-proven divergence before changing
production code; if the owner lies elsewhere, revise this boundary first.
This is a cited rewrite slice, not an independently designed Ares pipeline.

Producer comparison localizes the reproduced order difference before sorting:
`Arachne/WallToolPaths.cpp::removeSmallLines` consumes the parameter built by
`FillConcentricInternal.cpp`, and its swap-last deletion changes the collection
order. The Rust owner is `concentric/generate.rs`'s parameter construction,
consumed by `arachne/wall_toolpaths/postprocess.rs::remove_small_lines`.
Use ARD-0069's explicitly initialized independent-reference convention;
the defective untouched constructors do not define a deterministic factor.
Do not change the downstream sorting rules on this evidence.

Include first-point chaining, no-sort preservation and nesting only as
needed to repair the independently reproduced narrow-fill order. Defer
adjacent bridge geometry, M73 scheduling, cooling and perimeter extrusion
rounding to their source owners; preserve their complete failing streams.
The ARD-0069 initialized common min-length convention is a prerequisite
producer-input hypothesis, not permission to retune the sort. Its authorized
initialized reference remains distinct from untouched official semantics.

Non-negotiable: native project-byte to full G-code RED/GREEN, no new output
filters or injected native coordinates, no source-structure pinning tests,
all changed Rust modules below 400 LOC, Tier-1 portability, independent
six-axis review and code/docs commit/push together. Original disabled-support
and KSR cases remain global acceptance gates, not excused by a smaller fixture.
