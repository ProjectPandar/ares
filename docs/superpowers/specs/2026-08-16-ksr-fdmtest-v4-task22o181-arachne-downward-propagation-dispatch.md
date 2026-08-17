# Spec: KSR FDM Test V4 task181 Arachne downward propagation dispatch

## Observable contract

Downward beading propagation visits ordered non-central upward edges, skips central edges, reverses equal-radius edges when the known beading is on the nominal lower side, and dispatches each eligible edge to either empty-lower copy or existing-lower merge. This transfers final peak beadings toward the outline without overwriting central paths.

A focused graph test observes non-central dispatch into an empty lower node. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1591-1612`, the ordered dispatcher of `propagateBeadingsDownward`. Lazy source beading, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
