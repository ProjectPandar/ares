# Spec: KSR FDM Test V4 task192 Arachne local-maximum single beads

## Observable contract

A non-central strict local-maximum skeletal node whose retained beading has an odd number of widths emits one odd extrusion line at the middle perimeter index. The line is a six-segment ring centered on the node with radius `middle_width / 8`; every junction carries the middle width and index. Even beadings, central nodes, and non-maxima emit none.

A focused graph test observes six retained ring junctions and odd-line metadata. Existing Arachne tests, workspace formatting, and Clippy remain clean. Local-maxima logic and tests live in child modules below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:2056-2089`, `generateLocalMaximaSingleBeads`, into `segments/local_maxima.rs`. Final segment-stage orchestration, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
