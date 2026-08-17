# Spec: KSR FDM Test V4 task175 Arachne node beading storage

## Observable contract

Before segment propagation, every skeletal node with a positive bead count and zero transition ratio receives the effective beading strategy's result for twice its boundary radius. The beading is retained for later propagation and junction generation through `SkeletalJoint::beading()`.

A focused node test observes exact `Beading` equality through the skeletal graph. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the zero-transition branch of OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1469-1486` in the `generateSegments` setup. Transitional interpolation, upward/downward propagation, junction generation, connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
