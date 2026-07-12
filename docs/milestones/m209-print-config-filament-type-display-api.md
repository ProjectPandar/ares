# M209: DynamicPrintConfig filament type display API

## Goal
Port OrcaSlicer's UI-facing `DynamicPrintConfig::get_filament_type(std::string&, int)` support-filament display mapping into Ares.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8649-8714`, with `PrintConfig.hpp:657` declaration context, `Config.hpp:624-630` / `1886-1892` vector `get_at` fallback semantics, and `PrintConfig.cpp:2784-2797`, `2812-2816` / `PrintConfig.hpp:1322`, `1327` filament option context. It adds only a read-only `SliceOptions::filament_type_display(id)` helper and public `FilamentTypeDisplay` result type returning the source function's returned value plus displayed value. It does not port `is_using_different_extruders`, plural `filament_ids` behavior, preset bundle materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit checklist
- Missing `filament_type` returns empty returned/displayed strings.
- Present `filament_type` with missing `filament_is_support` returns the indexed filament type for both returned/displayed values.
- `filament_is_support=false` returns the indexed filament type for both values.
- Support filament with source singular `filament_id = "GFS00"` returns value `PLA-S` and displayed `Sup.PLA`.
- Support filament with source singular `filament_id = "GFS01"` returns value `PA-S` and displayed `Sup.PA`.
- Support filament without known support ID maps raw `PLA` to `PLA-S`/`Sup.PLA` and raw `PA` to `PA-S`/`Sup.PA`; other materials pass through.
- Vector `get_at` behavior uses first value when `id` exceeds vector length.
- Invalid non-vector/non-string/non-bool boundary values and empty present vectors return `SliceError::InvalidInput`.
- Existing validation APIs remain unchanged.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
