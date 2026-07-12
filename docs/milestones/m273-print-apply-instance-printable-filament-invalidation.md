# M273: PrintApply instance printable-filament invalidation

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the changed-instance synchronization loop body in `OrcaSlicer/src/libslic3r/PrintApply.cpp:1505-1511`, with branch/bounding-box context from `PrintApply.cpp:1487-1504`, apply-status context from `PrintApply.cpp:1231-1234`, staged printable-filament predicate context from `PrintApply.cpp:297-340`, print-step names from `OrcaSlicer/src/libslic3r/Print.hpp:78-88`, and invalidation API context from `OrcaSlicer/src/libslic3r/PrintBase.hpp:606-612`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private `ares-core` staged instance state and print-step shapes limited to the fields and steps used by `PrintApply.cpp:1505-1511`.
- Evaluate the staged M271 printable-filament predicate against old/new convex hulls before copying fields.
- Return staged `{psWipeTower, psGCodeExport}` invalidation steps only when the predicate is true.
- Copy transformation, print-volume state, and printable fields after successful predicate evaluation.
- Propagate predicate/config errors without mutating the old instance.
- Defer real print-state invalidation, `update_apply_status`, bounding-box invalidation, full instance-vector synchronization, concrete Clipper operations, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
