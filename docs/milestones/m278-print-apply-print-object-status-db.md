# M278: PrintApply PrintObjectStatusDB operations

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintObjectStatusDB` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:500-540`, with M277 `PrintObjectStatus` state context from `PrintApply.cpp:473-498`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private staged print-object status database over M277 `StagedPrintObjectStatus` records.
- Preserve constructor-from-print-objects behavior as a staged constructor from an ordered list of ids, inserting one `Unknown` record per input entry in source order.
- Preserve `std::multiset` semantics: duplicate ids are allowed and deterministic iteration is sorted by id while retaining duplicates.
- Preserve `get_range(...)` behavior as an id-keyed range over all records matching the requested id.
- Preserve `count(...)` behavior as the number of records matching the requested id.
- Preserve `begin`/`end` iteration and `clear()` behavior through private iterator/accessor helpers suitable for tests and later milestones.
- Defer real `PrintObject` pointers, concrete `Transform3d`, model-object apply wiring, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
