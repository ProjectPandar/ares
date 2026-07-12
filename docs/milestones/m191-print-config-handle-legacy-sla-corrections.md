# M191: PrintConfig handle_legacy_sla correction expansion

## Goal
Port OrcaSlicer's `handle_legacy_sla` correction-vector expansion into Ares `SliceOptions` ingestion so legacy SLA correction vectors populate newer scalar correction keys.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8507-8527` plus declaration context in `PrintConfig.hpp:693` and call-site context in `Preset.cpp:486` / `Model.cpp:456`. It covers only `relative_correction` and `material_correction` expansion into missing `_x`, `_y`, and `_z` scalar keys. No `PrintConfig.cpp:8529+` parameter sizing, extruder-variant extension, preset/model loading machinery, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- Present `relative_correction` creates missing `relative_correction_x` and `_y` from vector index `0`, and missing `_z` from vector index `1`.
- Present `material_correction` creates missing `material_correction_x` and `_y` from vector index `0`, and missing `_z` from vector index `1`.
- Existing scalar correction keys are preserved.
- Absent correction vector keys do not create scalar keys.
- Invalid or too-short correction vector values fail deserialization without panicking.
- Existing legacy composite normalization remains intact.
- `PrintConfig.cpp:8529+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
