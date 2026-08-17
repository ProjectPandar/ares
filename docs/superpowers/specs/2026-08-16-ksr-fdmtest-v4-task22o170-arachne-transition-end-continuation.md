# Spec: KSR FDM Test V4 task170 Arachne transition-end continuation

## Observable contract

When a transition half extends past its current central edge, transition-end generation continues the remaining distance through connected central edges. A focused two-edge chain test observes the upper transition end on the second upward edge at the source-derived remaining coordinate. Traversed joint bead count and transition ratio follow the interpolated half-transition state.

Existing transition tests, workspace formatting, and Clippy remain clean. Tests are added under the dedicated `transitions/ends/tests.rs` module so each Rust source remains below 400 lines.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1097-1161`, the recursive central-edge continuation in `generateTransitionEnd`. Multi-branch upward/downward direction rejection through `isGoingDown` remains deferred to the next source-cited slice.
