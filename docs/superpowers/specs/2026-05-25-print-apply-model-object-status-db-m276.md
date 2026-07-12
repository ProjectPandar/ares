# PrintApply ModelObjectStatusDB operations Spec

## Goal

Port OrcaSlicer's private `ModelObjectStatusDB` operations from `PrintApply.cpp` into `ares-core` as staged private helpers over M275 `StagedModelObjectStatus` records.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:442-470`: `ModelObjectStatusDB::add`, `add_if_new`, `get`, `reuse`, and id-keyed `std::set<ModelObjectStatus> db` storage.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:407-440`: M275 `ModelObjectStatus` state, constructor defaults, and id-based ordering operator.

## Requirements

- Extend private module `crates/ares-core/src/print_apply/model_object_status_state.rs`; do not add public APIs.
- Add `StagedModelObjectStatusDb` backed by deterministic id-keyed storage.
- Add `add(id, status)` that inserts absent ids and panics/asserts on duplicate ids, matching upstream `assert(db.find(...) == db.end())`.
- Add `add_if_new(id, status) -> bool` that inserts and returns `true` only when the id is absent; existing records must remain unchanged and return `false`.
- Add `get(id) -> &StagedModelObjectStatus` that returns the existing record and panics/asserts on missing ids.
- Add `reuse(id) -> &StagedModelObjectStatus` that delegates to `get` and panics/asserts when the stored status is `Deleted`.
- Preserve id-keyed uniqueness and deterministic ordering by id.
- Do not implement `PrintObjectStatus`, `PrintObjectStatusDB`, ref-count behavior, model-object apply wiring, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
