# Spec: KSR FDM Test V4 infill entity chaining

## Observable contract

For `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`, after a layer island's perimeters finish, Ares chooses the nearest eligible infill collection from the current nozzle position before emitting its paths. The first layer therefore selects the same collection near X137 as the reference without retracting and jumping to the collection near X169. Its retained monotonic polyline currently starts at the opposite endpoint (`Y101.342` before `Y101.382`); matching that upstream fill-generation orientation is deferred rather than hard-coded into G-code ordering.

Collections marked `no_sort` retain their internal path order and may not be reversed. Sortable collections are internally chained from the live nozzle position. Gap-fill loops are not reversible. Ordering derives only from generated entities and the live motion state.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:5432-5468` and `6130-6175`, where islands are visited in layer order, infill collections are chained from `m_last_pos`, and each selected sortable collection calls `chained_path_from(m_last_pos)`. Reuse the existing Rust rewrite of `ShortestPath.cpp` in `project_slice/perimeters/classic/shortest_path/entity_chain.rs`; remove its temporary test-only/dead-code pinning attributes when the production emitter activates it.

Included: live-cursor ordering of infill and thin-fill entities inside each island, collection reversal constraints, and per-collection internal chaining. Deferred: monotonic polyline generation orientation, support-material ordering, multi-extruder override passes, ironing, and print-object instance chaining.
