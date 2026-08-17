# Spec: KSR FDM Test V4 task169 Arachne transition-end generation

## Observable contract

For every retained Arachne transition middle on an upward central edge, transition-end generation produces the lower and upper half-transition endpoints at the beading strategy's configured transition length and anchor. Ends are stored on the upward edge in ascending edge coordinates with the source lower bead count and the correct lower/upper-end flag.

A focused straight-edge test observes the generated endpoints through `SkeletalEdge::transition_ends()`. Existing transition tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1040-1200`: `generateAllTransitionEnds`, `generateTransitionEnds`, and the local endpoint insertion path of `generateTransitionEnd`. Recursive propagation across a central junction and `isGoingDown` are deferred to the next source-cited slice.
