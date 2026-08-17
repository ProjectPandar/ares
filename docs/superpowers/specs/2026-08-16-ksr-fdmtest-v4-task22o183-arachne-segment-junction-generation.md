# Spec: KSR FDM Test V4 task183 Arachne segment junction generation

## Observable contract

For each skeletal half-edge directed from lower to higher boundary radius whose endpoint bead counts differ, segment generation emits extrusion junctions at the higher node's beading radii that lie within the edge's radius interval. Each junction carries the source bead width and perimeter index, is placed by integer interpolation on the edge, and is retained on that half-edge. Reverse, flat, and equal-assigned-count edges emit none.

A focused graph test observes the exact generated junction. Existing Arachne tests, workspace formatting, and Clippy remain clean. New junction tests live in a child module so every Rust source stays below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1727-1801`, `generateJunctions`, into `transitions/segments/junctions.rs`. Lazy source beading, junction connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
