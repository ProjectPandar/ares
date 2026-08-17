# Spec: KSR FDM Test V4 task172 Arachne transition application

## Observable contract

Applying generated transition ends normalizes twin-edge endpoint coordinates, sorts endpoints along their owning edge, and inserts central skeletal nodes at the geometric transition positions with the lower or upper bead count selected by the endpoint flag. Endpoints within 0.02 mm of an existing matching-bead node snap to that node and reset its transition ratio instead of creating a duplicate.

A focused cell-graph test observes the inserted central node and bead count through `SkeletalGraph`. Existing transition and skeletal tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1261-1334`: `normal` and `applyTransitions`. Extra ribs, segment generation, beading propagation, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
