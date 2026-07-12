# PrintApply ModelObjectStatus state Spec

## Goal

Port OrcaSlicer's private `ModelObjectStatus` state vocabulary and default record shape from `PrintApply.cpp` into `ares-core` as staged private types for later model-object apply database work.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:407-440`: `ModelObjectStatus` comment, `Status`, `PrintObjectRegionsStatus`, constructor defaults, member defaults, and id ordering operator.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:442-470`: `ModelObjectStatusDB` uses id-keyed set lookup and rejects `Deleted` status in `reuse(...)`; this milestone only stages the record and vocabulary needed by those later operations.

## Requirements

- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add `StagedModelObjectApplyStatus` with variants matching upstream source order: `Unknown`, `Old`, `New`, `Moved`, `Deleted`.
- Add `StagedPrintObjectRegionsStatus` with variants matching upstream source order: `Invalid`, `Valid`, `PartiallyValid`.
- Add `StagedModelObjectStatus` with fields sufficient for this slice:
  - `id: u64`
  - `status: StagedModelObjectApplyStatus`
  - `print_object_regions_status: StagedPrintObjectRegionsStatus`
- Provide a private constructor equivalent to `ModelObjectStatus(ObjectID id, Status status = Unknown)` that sets the supplied id/status and defaults `print_object_regions_status` to `Invalid`.
- Preserve id-based ordering and equality semantics so collections sort/search by id only, matching `operator<(const ModelObjectStatus &rhs) const { return id < rhs.id; }`.
- Keep `print_instances` and `print_object_regions` out of the Rust type for now except as documented deferrals; do not invent placeholder print-object/region payloads.
- Do not implement `ModelObjectStatusDB`, `PrintObjectStatus`, ref-count behavior, model-object apply wiring, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
