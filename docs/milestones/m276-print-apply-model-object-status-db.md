# M276: PrintApply ModelObjectStatusDB operations

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `ModelObjectStatusDB` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:442-470`, with M275 `ModelObjectStatus` state context from `PrintApply.cpp:407-440`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private staged model-object status database over M275 `StagedModelObjectStatus` records.
- Preserve `add(...)` behavior: reject duplicate object ids and insert the supplied status for absent ids.
- Preserve `add_if_new(...)` behavior: insert absent ids and return `true`; leave existing records unchanged and return `false`.
- Preserve `get(...)` behavior: return the status record for an existing id and fail on missing ids.
- Preserve `reuse(...)` behavior: return the existing status record and fail when the record status is `Deleted`.
- Preserve set-like id-keyed uniqueness and deterministic id ordering.
- Defer `PrintObjectStatus`, `PrintObjectStatusDB`, ref-counted regions, print object transformations, model-object apply wiring, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
