# PrintApply PrintObjectStatus state Spec

## Goal

Port OrcaSlicer's private `PrintObjectStatus` state vocabulary and id-keyed record shape from `PrintApply.cpp` into `ares-core` as staged private types for later print-object status database work.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:473-498`: `PrintObjectStatus`, `Status`, constructor defaults, member fields, and id ordering operator.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:500-540`: `PrintObjectStatusDB` stores a `std::multiset<PrintObjectStatus>` and uses id-keyed range/count lookups; this milestone only stages the record and vocabulary needed by those later operations.

## Requirements

- Add private module `crates/ares-core/src/print_apply/print_object_status_state.rs` and register it from `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add `StagedPrintObjectApplyStatus` with variants matching upstream source order: `Unknown`, `Deleted`, `Reused`, `New`.
- Add `StagedPrintObjectStatus` with fields sufficient for this slice:
  - `id: u64`
  - `status: StagedPrintObjectApplyStatus`
- Provide a private constructor equivalent to `PrintObjectStatus(ObjectID id)` that sets the supplied id and defaults status to `Unknown`.
- Preserve id-based ordering and equality semantics so collections sort/search by id only, matching `operator<(const PrintObjectStatus &rhs) const { return id < rhs.id; }`.
- Keep `print_object` and `trafo` out of the Rust type for now except as documented deferrals; do not invent placeholder print-object pointers or transform payloads.
- Do not implement `PrintObjectStatusDB`, real `PrintObject` storage, concrete `Transform3d`, model-object apply wiring, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
