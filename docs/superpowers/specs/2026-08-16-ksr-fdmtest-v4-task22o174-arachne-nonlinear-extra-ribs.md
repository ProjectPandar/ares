# Spec: KSR FDM Test V4 task174 Arachne nonlinear extra ribs

## Observable contract

For an upward central edge long enough for discretization, each nonlinear thickness returned by the effective beading strategy whose half-thickness lies strictly between the endpoint radii inserts a central skeletal node at the linearly interpolated radius crossing. The inserted node uses the lower endpoint bead count. Crossings within 0.02 mm of an existing matching-bead endpoint snap instead of duplicating it.

A focused central-cell test derives its crossing from the loaded strategy and observes the inserted node through `SkeletalGraph`. Existing transition and skeletal tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1357-1419`: `generateExtraRibs`. Segment generation, beading propagation, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
