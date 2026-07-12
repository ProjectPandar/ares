# PrintApply instance printable-filament invalidation Spec

## Goal
Port the changed-instance synchronization slice that calls `is_printable_filament_changed(...)` from OrcaSlicer's `PrintApply.cpp` into `ares-core` as a private staged helper, preserving the wipe-tower/G-code invalidation decision while deferring full `Print::apply` mutation and real print-state invalidation.

## Rewrite gate mapping
Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1505-1511`: call `is_printable_filament_changed(...)`, invalidate `{psWipeTower, psGCodeExport}` when true, then copy transformation, `print_volume_state`, and `printable`.

Required context:
- `PrintApply.cpp:1487-1504`: instance branch/bounding-box context.
- `PrintApply.cpp:1231-1234`: `update_apply_status` aggregation context.
- `PrintApply.cpp:297-340`: M271 staged printable-filament predicate.
- `Print.hpp:78-88`: `psWipeTower`, `psGCodeExport`.
- `PrintBase.hpp:606-612`: `invalidate_step(s)` context.

## Requirements
- Extend private `crates/ares-core/src/print_apply.rs`; add no public APIs.
- Add a private staged instance type containing only convex hull, transform token, print-volume-state token, and printable flag.
- Add a private staged print-step enum for `WipeTower` and `GCodeExport`.
- Add a private helper accepting mutable old instance, immutable new instance, config map, and injected M271 geometry callbacks grouped in a private operations struct.
- Preserve order: predicate before mutation; field copy after successful predicate evaluation.
- Return both staged steps only when predicate returns true; return empty steps when false.
- Propagate predicate errors and leave old instance unchanged on error.
- Defer real `Print::invalidate_steps`, `update_apply_status`, bounding-box invalidation, full instance-vector comparison, Clipper backend, public APIs, profiles, UI, slicing, extrusion, G-code, dependencies, crates, and Ares-owned pipeline behavior.
