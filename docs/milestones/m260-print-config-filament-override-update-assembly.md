# M260: compute_filament_override_value update assembly

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the clone/apply/change/output suffix of `compute_filament_override_value(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10073-10082`, with M258 input-preparation context from `PrintConfig.cpp:10051-10071`, declaration context from `PrintConfig.hpp:690-691`, and M259 vector override semantics from `Config.hpp:713-753`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned filament override pipeline.

## Exit criteria

- Add an internal `ares-core` helper that stages `compute_filament_override_value(...)` update assembly for JSON vector options.
- Preserve upstream behavior: clone the new machine value, apply the prepared filament override value using existing vector override semantics, compare the result against the old machine value, append the key and store the computed override value only when changed, and leave outputs untouched when unchanged.
- Reuse M258 long-retraction input preparation and M259 vector `apply_override` mapping instead of reimplementing those semantics.
- Preserve that output emission uses the final old-machine versus computed-value comparison, not the `apply_override(...)` modified flag.
- Preserve zero-overlap no-op behavior inherited from `ConfigOptionVector<T>::apply_override(...)`.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement concrete Orca `ConfigOption` type hierarchy dispatch, scalar option override dispatch, public API wiring, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
