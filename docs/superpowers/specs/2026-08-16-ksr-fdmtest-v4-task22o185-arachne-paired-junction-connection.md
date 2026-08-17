# Spec: KSR FDM Test V4 task185 Arachne paired junction connection

## Observable contract

Two junction sides of one skeletal quad connect from their innermost junctions outward. Only the shared junction count is connected, and each pair must carry the same perimeter index. Every matched pair is passed to toolpath assembly with the quad's parity, domain-boundary, and three-way conditions.

A focused test observes exact inner/outer perimeter toolpaths from paired edge storage. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the paired-side matching core of OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:2012-2049`, `connectJunctions`, into the junction child module. Quad/domain traversal, adjacent-edge concatenation, odd-edge deduplication, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
