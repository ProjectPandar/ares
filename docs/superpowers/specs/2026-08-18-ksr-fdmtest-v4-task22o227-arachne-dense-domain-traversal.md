# Spec: KSR FDM Test V4 task227 Arachne dense domain traversal

## Observable contract

Arachne junction domains are visited in the same order as OrcaSlicer’s `ankerl::unordered_dense::set`: candidate starts retain graph-edge insertion order, and removing a visited edge fills its dense slot with the last candidate. Repeated slices of the same project therefore produce identical concentric solid-infill geometry and G-code.

The traversal is derived only from the generated skeletal graph. It does not depend on fixture identity, reference G-code, or known coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:1935-1966`, including the iteration and erase behavior supplied by `deps_src/ankerl/unordered_dense.h:1063-1082,1360-1369`. The Rust destination is `crates/ares-core/src/arachne/trapezoidation/transitions/segments/junctions/domains.rs`.

Included: source-compatible dense domain-start ordering and deterministic removal. Deferred: other Arachne topology differences, remaining arc numerics, rectilinear geometry, travel/retraction and wipe parity, cooling, timing/M73, and later normalized G-code differences.
