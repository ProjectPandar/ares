# PrintApply PrintObjectStatusDB operations Spec

## Goal

Port OrcaSlicer's private `PrintObjectStatusDB` operations from `PrintApply.cpp` into `ares-core` as staged private helpers over M277 `StagedPrintObjectStatus` records.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:500-540`: `PrintObjectStatusDB` constructor, `iterator_range`, `get_range`, `count`, `begin`, `end`, `clear`, and `std::multiset<PrintObjectStatus> m_db` storage.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:473-498`: M277 `PrintObjectStatus` state, constructors, and id-based ordering operator.

## Requirements

- Extend private module `crates/ares-core/src/print_apply/print_object_status_state.rs`; do not add public APIs.
- Add `StagedPrintObjectStatusDb` backed by deterministic multiset-like storage.
- Add a private constructor from ordered ids that creates one `StagedPrintObjectStatus::new(id)` per input id and preserves duplicates.
- Preserve deterministic sorted iteration by id, including duplicate ids.
- Add `get_range(id) -> &[StagedPrintObjectStatus]` or equivalent private range accessor that returns all matching records for an id.
- Add `count(id) -> usize` that returns the number of matching records.
- Add private iteration/accessor support equivalent to `begin()` / `end()` for later milestones and tests.
- Add `clear()` that removes all records.
- Do not implement real `PrintObject` pointer extraction, concrete `Transform3d`, model-object apply wiring, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
