# Spec: KSR FDM Test V4 task193 Arachne segment-stage orchestration

## Observable contract

One segment-stage operation executes retained-beading propagation, edge-junction generation, polygon-domain connection, and local-maximum single-bead generation in source order. A graph with a strict odd local maximum and no pre-attached beading produces its ring through that operation, proving propagation feeds later segment generation.

The orchestration stays inside the Arachne trapezoidation module; no fixture identifiers or output constants enter production code. Focused transition tests, workspace formatting, and Clippy remain clean; files stay below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1936-1941`, `generateToolpaths`, into the existing Rust transition segment seam. Invocation from the full trapezoidation pipeline, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
