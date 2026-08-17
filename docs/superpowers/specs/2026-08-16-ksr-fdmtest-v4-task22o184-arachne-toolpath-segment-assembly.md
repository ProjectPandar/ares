# Spec: KSR FDM Test V4 task184 Arachne toolpath segment assembly

## Observable contract

Adding a non-degenerate extrusion segment places it in the vector for its perimeter index. A compatible segment extends the current line when its start (or end, for reversible odd paths) is within 0.010 mm of the current endpoint and its width differs by less than 0.010 mm. Forced boundaries, parity changes, perimeter-index mismatches, and three-way endpoints start a new line.

Focused tests observe new-line creation and compatible forward extension. Existing Arachne tests, workspace formatting, and Clippy remain clean. Toolpath tests live in a child module below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1887-1932`, `addToolpathSegment`, into `transitions/segments/toolpaths.rs`. Quad traversal, junction connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
