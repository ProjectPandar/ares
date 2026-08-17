# Spec: KSR FDM Test V4 task178 Arachne upward beading propagation

## Observable contract

An ordered upward skeletal edge propagates an existing lower-node beading to an unassigned higher node when the higher node has no local bead-count assignment. The copied propagation records the traveled edge length, is marked upward-only, and never replaces a local or existing higher-node beading.

A focused graph test observes the propagated beading state. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1561-1588`, `propagateBeadingsUpward`. Downward propagation, radius-aware unequal-count interpolation, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
