# Spec: KSR FDM Test V4 task191 Arachne odd-segment deduplication

## Observable contract

A polygon-domain junction walk emits a single-bead odd segment only once across the two quad sides that share its central edge. Before adding an odd segment, connection checks whether the current central edge's twin was already passed; after processing a segment, it records the current central edge. Even segments remain unaffected.

A focused odd-quad test seeds the opposite edge as passed and observes no duplicate toolpath. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1947,2038-2046`, `passed_odd_edges` handling in `connectJunctions`. Local maxima, final segment-stage orchestration, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
