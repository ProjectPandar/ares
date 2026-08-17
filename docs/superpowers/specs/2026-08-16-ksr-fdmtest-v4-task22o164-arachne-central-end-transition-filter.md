# Spec: KSR FDM Test V4 task164 Arachne central-end transition filter

## Observable contract

When a transition half-length reaches the end of a central skeletal branch, every reached terminal inside that distance changes to the replacement bead count. If any descendant branch reaches such a terminal, the bead-count change propagates back through its traversed central nodes. Branches whose cumulative length exceeds the limit remain unchanged.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:1010-1038`, `filterEndOfCentralTransition()`, into `crates/ares-core/src/arachne/trapezoidation/transitions.rs`.

Included: central-edge traversal, cumulative distance cutoff, terminal detection, and replacement bead-count propagation. Deferred: nearby-transition dissolution, transition-middle filtering orchestration, transition-end generation, rib application, and final segment generation.

## Acceptance

Focused graph tests cover a reached terminal and an over-limit terminal. Existing Arachne tests, workspace formatting, and Clippy remain clean.
