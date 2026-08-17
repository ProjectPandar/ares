# Spec: KSR FDM Test V4 task167 Arachne transition-middle filtering

## Observable contract

Generated Arachne transition middles are filtered from both ends of every upward central edge. Opposing same-count transitions inside `transition_filter_dist` dissolve together, their intervening bead-count region is rewritten, and transition halves that end before a central terminal are removed. Remaining middles preserve ascending bead-count order.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:864-931`, `filterTransitionMids()`, into `crates/ares-core/src/arachne/trapezoidation/transitions.rs`, using the already ported nearby-discovery, bead-region, and central-end helpers.

Included: back/front filtering order, referenced-middle erasure, bead-count rewrites, and terminal half-length checks. Deferred: transition-end generation and application, ribs, segments, and concentric internal fill activation.

## Acceptance

A focused central-chain test proves a nearby opposing transition pair and its bead-count region are dissolved. Existing transition tests, workspace formatting, and Clippy remain clean.
