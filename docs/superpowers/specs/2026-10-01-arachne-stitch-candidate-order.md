# Arachne stitch candidate selection rewrite slice

Port only the proven divergent behavior of
`Arachne/utils/PolylineStitcher.hpp::stitch` and its
`SparseGrid.hpp::processFromCell` / `SquareGrid::processNearby` dependencies,
through `WallToolPaths.cpp::stitchToolPaths`, into existing
`ares-core/src/arachne/wall_toolpaths/stitch.rs`. The current adapter remains
an upstream integration shell; do not design an independent Ares pipeline.

Prepared contours and pre-stitch paths match on KSR layer64. Post-stitch
connections differ before small-line removal and simplification. Include only
the eventual demonstrated candidate-order, tie or closing-choice mismatch;
defer upstream graph generation, unrelated postprocessing, flow conversion,
M73 nondeterminism and additional context RNG plumbing.

Add a public closed-mesh complete-output regression in a separate test module.
Verify requested settings are applied and reference producer identity is
explicit. Before implementing, establish a sensitive RED through the existing
full byte comparator and prove it exercises the captured stitch discrepancy.
Original KSR and all affected consumers must be rerun without comparator
changes. Retain full project and both G-code streams for every comparison.

Exit requires RED/GREEN evidence, original KSR revalidation, workspace Nextest
without fail-fast, fmt/clippy, Tier-1 compilation and independent six-axis
review with repair/re-review. No private source-pinning assertions; Rust files
stay below400LOC and use mod, not include macros. Commit code and docs together
only after applicable submission gates close.
