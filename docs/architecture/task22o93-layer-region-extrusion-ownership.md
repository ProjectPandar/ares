# Task 22O.93 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Layer.hpp:43-76` (`LayerRegion::perimeters`, `thin_fills`, and
`fills`), into the post-O92 layer output ownership boundary.

Move each aligned retained perimeter collection beside generated fills and moved
thin fills. Preserve source region/collection/tree/path order and all role/flow
geometry. This creates one owning layer-region extrusion record for later island
sorting without adapting through Ares's legacy independently-generated pipeline.

Deferred: `Layer::sort_perimeters_into_islands`, island/chaining order, motion,
and G-code. No fallback or fixture branch.

Compile RED proved O92 lacked perimeter ownership. The KSR oracle freezes 2,881
collections, 5,243 loops, 5,483 paths, and 111,933 points and proves predecessor
perimeter/thin inventories are drained. Three lifecycle/repeatability tests and
strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass.
