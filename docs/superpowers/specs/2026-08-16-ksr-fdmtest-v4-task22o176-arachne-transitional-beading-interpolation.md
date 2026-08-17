# Spec: KSR FDM Test V4 task176 Arachne transitional beading interpolation

## Observable contract

A positive-count skeletal node with nonzero transition ratio stores a beading interpolated between the effective strategy results for its lower bead count and the next bead count. The lower-count weight is `1 - transition_ratio`; zero-width markers remain zero, shared widths and toolpath locations are linearly interpolated, and unmatched entries plus total thickness/leftover come from the thicker result selected by the source rule.

A focused deterministic strategy test observes the complete retained `Beading`. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1487-1499,1706-1725`. Radius-aware unequal-count propagation interpolation, upward/downward propagation, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
