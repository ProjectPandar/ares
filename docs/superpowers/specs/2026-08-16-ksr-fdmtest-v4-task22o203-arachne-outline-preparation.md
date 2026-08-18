# Spec: KSR FDM Test V4 task203 Arachne outline preparation

## Observable contract

`wall_toolpaths::generate` accepts raw fill polygons, not pre-sanitized Voronoi input. Before skeletal trapezoidation it applies the source epsilon close-open-close offset sequence, removes degenerate/repeated vertices, simplifies near-collinear geometry, normalizes self-intersections through Clipper, drops sub-threshold areas, and unions the result. Empty or nonpositive prepared geometry returns no toolpaths.

A regression polygon captured from the task202 fixture previously trips `Transitions::insert_extra_ribs` because it bypasses preparation. The focused test now completes without panic and any retained junctions have positive widths. Files remain below 400 LOC; focused Arachne/geometry tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the reached preparation sequence from OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:86-201,314-405,407-515`: triple epsilon offsets, accumulated-area simplification, duplicate/near-collinear cleanup, even-odd and nonzero normalization, and small-area filtering before `SkeletalTrapezoidation`. Grid-guided near-self-intersection point nudging and exact hole-preserving small-area ordering remain deferred. Fixture smoke now reaches the next omitted source behavior, `SkeletalTrapezoidation.cpp:1804-1844` `getOrCreateBeading`, at a node without stored beading; that recovery, cooling, timing, and remaining exact G-code differences are deferred.
