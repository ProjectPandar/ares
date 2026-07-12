# M259: ConfigOptionVector apply_override mapping

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `ConfigOptionVector<T>::apply_override(...)` in `OrcaSlicer/src/libslic3r/Config.hpp:713-753`, with caller context from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10073-10076` inside `compute_filament_override_value(...)`, M258 long-retraction input-preparation context from `PrintConfig.cpp:10051-10071`, and deferred changed-key/output context from `PrintConfig.cpp:10077-10082`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned filament override pipeline.

## Exit criteria

- Add an internal `ares-core` helper that applies Orca vector override semantics to staged JSON vectors.
- Preserve upstream behavior: non-nullable override vectors replace machine values only when different and report whether they modified values; nullable override vectors return no-op when either vector has zero overlap, otherwise resize the machine vector to override length using the first machine value as fill, copy non-nil override entries, restore nil entries from `default_index[i] - 1` when valid, otherwise restore from the first original machine value, and report modification only when at least one non-nil override entry was copied.
- Use existing M258 staging conventions: JSON string `"nil"` represents nullable nil entries.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `compute_filament_override_value` changed-key insertion, `filament_overrides` config mutation, option type hierarchy dispatch, public API wiring, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
