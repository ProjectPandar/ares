# Spec: KSR FDM Test V4 task186 Arachne quad peak edge selection

## Observable contract

Given the first half-edge of a Voronoi quad chain, junction connection selects the edge whose destination node has the greatest boundary radius. If that maximum is the terminal edge and its radius gain is below 0.005 mm, the preceding edge is selected so the result always has an outgoing side from the peak.

A focused quad test observes the selected peak edge. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1536-1559`, `getQuadMaxRedgeTo`, into the junction child module. Quad/domain traversal, adjacent-edge concatenation, odd-edge deduplication, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
