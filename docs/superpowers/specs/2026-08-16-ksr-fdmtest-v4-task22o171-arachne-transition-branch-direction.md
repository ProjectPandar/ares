# Spec: KSR FDM Test V4 task171 Arachne transition-end branch direction

## Observable contract

When an increasing transition reaches a junction with multiple central continuations, transition-end generation rejects a branch that is already descending toward the boundary and continues on the non-descending branch. A focused branched-graph test observes an upper transition end only on the valid upward branch.

Existing transition tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1126-1154,1203-1259`: multi-branch counting and `isGoingDown`, including boundary termination, bead-count and nearby-transition checks, distance limiting, and recursive all-central-branch classification. Transition insertion into the graph, ribs, segments, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
