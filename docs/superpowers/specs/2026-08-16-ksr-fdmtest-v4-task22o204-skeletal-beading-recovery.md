# Spec: KSR FDM Test V4 task204 skeletal beading recovery

## Observable contract

Junction generation does not assume every higher-radius node already owns beading. It first reuses the nearest reachable beading within 0.1 mm using the source bounded distance-priority graph walk. If a missing node has unresolved bead count, it derives that count from the closest incident destination radius plus edge length, then computes and stores a beading at the node radius. Nodes with an existing bead count compute and store directly.

A focused test creates a node with bead count but no beading and observes source strategy output stored on that node. Fixture slicing must pass the task203 missing-beading panic. Files remain below 400 LOC; focused transition tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1746-1751,1804-1885`, `getOrCreateBeading` and `getNearestBeading`, into the Rust junction-generation seam. Outline near-intersection nudging, cooling, timing, and remaining G-code differences are deferred.
