# Spec: KSR FDM Test V4 task182 Arachne node-beading propagation stage

## Observable contract

The segment stage executes node-beading preparation in source order: collect and sort upward quad edges, retain local strategy beadings, propagate known lower beadings upward, then propagate final non-central peak beadings downward. A peak-local/non-central edge therefore leaves its previously empty lower node with the peak beading in one stage call.

A focused graph test observes the end-to-end node propagation state. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice wires the beading preparation prefix of OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1430-1612`, `generateSegments`, `propagateBeadingsUpward`, and `propagateBeadingsDownward`. Lazy source beading, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
