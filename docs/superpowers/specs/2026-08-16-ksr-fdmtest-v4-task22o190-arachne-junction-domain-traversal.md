# Spec: KSR FDM Test V4 task190 Arachne junction domain traversal

## Observable contract

Junction connection starts from every unprocessed quad-chain edge with no predecessor, walks each polygon domain through `next_unconnected`, connects every quad exactly once, and forces a new toolpath only at the domain's first quad. A two-sided closed domain therefore assembles both directions into one closed extrusion line.

A focused closed-domain graph test observes one line whose first and last junction coincide. Existing Arachne tests, workspace formatting, and Clippy remain clean. Domain traversal and its tests live in child modules below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1934-1967,2048-2053`, the domain walk in `connectJunctions`, into `segments/junctions/domains.rs`. Odd-edge deduplication across adjacent quads, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
