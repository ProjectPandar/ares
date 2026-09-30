# Internal-solid variable-width rewrite slice

The initial converter hypothesis is superseded by producer evidence: all
target prepared-outline vertices match, but toolpaths differ before stitching.
Investigate `SkeletalTrapezoidation.cpp` construction and `generateToolpaths`,
called by `WallToolPaths.cpp::generate` and the concentric fill adapter.
Destinations are `arachne/trapezoidation.rs::{new,generate_toolpaths}` and its
existing builder/filtering/transition modules, not a new Ares pipeline.
Included behavior is only a demonstrated junction-generation mismatch once
its first differing stage is located. Defer unrelated Arachne behavior;
neither variable-width conversion nor outline preparation currently warrants
a repair.

Current captured first differing stage is
`SkeletalTrapezoidation.cpp::generateTransitioningRibs`, between matching
post-filterNoncentralRegions state and differing pre-generateExtraRibs state.
Its sub-boundaries are generateTransitionMids/filterTransitionMids,
generateAllTransitionEnds/generateTransitionEnds and applyTransitions. Rust
destinations are existing `trapezoidation/transitions.rs` and its filtering,
ends and apply modules. Construction and early central/bead-count updates
are excluded as demonstrated repair targets by matching recorded states.
This narrows investigation, not permission for an unproven substage repair.

Substage probes match mids, filtered mids and ends. Actual snap-check trace
shows the second endpoint selecting the previously inserted bead4 node rather
than the original bead3 endpoint, despite being16016 units from that endpoint
and within the20000-unit snap distance. The included upstream rewrite slice
is now precisely applyTransitions' cached original from/to snap behavior,
mapped to apply_transitions_on_edge's existing local node IDs. Adjacent
ordering, graph insertion, weak-storage lifetime and converter behavior are
deferred. Remove the superseded private mutated-source-endpoint expectation.

Public regression uses an independently triangulated30-vertex closed prism,
height0.6mm, through native project export and the core byte-in API. Check all
requested settings were applied; compare every ordered output byte using the
existing generator-only normalizer. Native initialized-reference provenance
remains explicit. Disabling M73 is an applied small-fixture option, not a
comparison filter or a claim to resolve original KSR M73 differences.

Use the agreed project-bytes to complete-G-code seam. Retain the original KSR
project and both full streams, minimize one input dimension at a time, and
verify applied settings. Compare converter input points/widths before output
flows to distinguish upstream geometry drift from materialization drift.

Exit requires a sensitive public RED/GREEN, original complete KSR revalidation,
affected consumer tests, workspace Nextest, fmt/clippy and Tier-1 compile checks,
then independent six-axis review and repair/re-review. Keep tests in separate
modules and Rust files below400LOC. Code and docs commit/push together only
after the applicable submission gates pass. No source-pinning tests.
