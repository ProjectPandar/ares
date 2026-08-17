# Spec: KSR FDM Test V4 task180 Arachne downward beading merge

## Observable contract

When downward propagation reaches a lower node that already contains an upward-propagated beading, the two propagation sources merge over the configured transition distance. The lower node keeps the merged beading, clears upward-only state, and resets propagation distances. If the upper source fully dominates, its state replaces the lower source and accumulates the traversed edge length. Unequal-count interpolation respects the lower-node switching radius so an inset does not cross the skeletal center during blending.

Focused graph tests observe merged widths, locations, and propagation state. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1637-1703`, the existing-lower-node branch of `propagateBeadingsDownward` and switching-radius interpolation. Downward edge dispatch, lazy source beading, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
