# Spec: KSR FDM Test V4 task165 Arachne bead-count region dissolution

## Observable contract

Transition filtering can replace one bead count across the connected central region reached above a starting edge. The traversal changes only nodes with the requested source bead count and ignores noncentral branches and regions that already carry another bead count.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:993-1008`, `dissolveBeadCountRegion()`, into `crates/ares-core/src/arachne/trapezoidation/transitions.rs`.

Included: recursive central-edge traversal and exact source-count replacement. Deferred: nearby-transition discovery, transition-middle filtering orchestration, transition ends, ribs, segment generation, and activation in concentric internal fill.

## Acceptance

A focused branched-graph test proves matching central nodes change while noncentral and nonmatching branches do not. Existing Arachne tests, workspace formatting, and Clippy remain clean.
