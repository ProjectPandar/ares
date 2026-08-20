# Spec: KSR FDM Test V4 task228 elephant-foot batch preservation

## Observable contract

Batch elephant-foot compensation applies the single-ExPolygon operation independently in input order and returns each result directly. It does not run a second union over compensated outputs. A tiny or natural-result-count fallback therefore preserves the original contour start, orientation, holes, and sibling position exactly.

All results derive from loaded first-layer geometry, the external-perimeter flow, and the effective `elefant_foot_compensation` option. Production code does not inspect fixture identity or known coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/ElephantFootCompensation.cpp:544-643`: the `ExPolygons` overload reserves output and `emplace_back`s one `elephant_foot_compensation` result per input without a union pass. The Rust destination is `crates/ares-core/src/project_slice/elephant_foot.rs::compensate_expolygons` and its focused tests under `project_slice/tests/elephant_foot`.

Included: ordered one-to-one batch mapping, exact fallback preservation, and removal of obsolete tests that pinned the extra Ares union. Deferred: remaining first-layer contour numerics, classic perimeter offsets, Arachne topology, travel/retraction, cooling, timing/M73, and later normalized G-code differences.
