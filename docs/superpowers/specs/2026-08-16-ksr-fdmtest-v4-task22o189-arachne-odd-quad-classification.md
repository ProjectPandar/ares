# Spec: KSR FDM Test V4 task189 Arachne odd-quad classification

## Observable contract

For each paired quad segment, only the innermost pair is classified as an odd single-bead segment, and only when both quad-side nodes have positive odd bead counts, zero transition ratio, and junction points within 0.005 mm of those nodes. Multi-intersection endpoints force three-way path breaks. Other perimeter pairs remain even.

A focused odd two-edge quad test observes an odd generated extrusion line. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:2028-2048`, odd-segment and three-way classification in `connectJunctions`. Domain traversal, odd-edge deduplication across adjacent quads, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
