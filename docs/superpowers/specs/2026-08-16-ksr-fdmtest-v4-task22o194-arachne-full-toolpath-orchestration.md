# Spec: KSR FDM Test V4 task194 Arachne full toolpath orchestration

## Observable contract

A constructed skeletal trapezoidation can be consumed into variable-width extrusion lines. Generation executes source stages in order: central marking/filtering, optional outer-central filtering, bead-count update, noncentral-region filtering, transition middle/filter/end/application, extra ribs, then segment generation. A normal-scale rectangle yields at least one non-empty extrusion line with at least two junctions.

The caller chooses the upstream `filter_outermost_central_edges` behavior. Generated lines are returned without exposing graph/storage internals. Focused Arachne tests, formatting, and Clippy remain clean; files stay below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:496-552`, `generateToolpaths`, and `:746-781`, `generateTransitioningRibs`, into `trapezoidation.rs`. Consumption by Arachne wall/fill generation, concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
