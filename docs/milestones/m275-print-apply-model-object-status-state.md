# M275: PrintApply ModelObjectStatus state

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `ModelObjectStatus` state declaration in `OrcaSlicer/src/libslic3r/PrintApply.cpp:407-440`, with follow-on database operation context from `PrintApply.cpp:442-470`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private `ares-core` staged data structures for `ModelObjectStatus::Status` values: `Unknown`, `Old`, `New`, `Moved`, `Deleted`.
- Add private `ares-core` staged data structures for `ModelObjectStatus::PrintObjectRegionsStatus` values: `Invalid`, `Valid`, `PartiallyValid`.
- Add a private staged model-object status record keyed by object id, defaulting `status` to `Unknown` and `print_object_regions_status` to `Invalid`.
- Preserve source-order enum vocabulary and id-based ordering/equality semantics for later `ModelObjectStatusDB` work.
- Keep `print_instances` and `print_object_regions` as explicit deferred fields rather than inventing Ares-owned print-object or region pipelines.
- Defer `ModelObjectStatusDB` methods, `PrintObjectStatus`, ref-counted `PrintObjectRegions`, `PrintObjectTrafoAndInstances`, model-object apply wiring, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
