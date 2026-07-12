# M184: PrintConfig variant option key sets

## Goal
Port the global variant-related option key sets from `OrcaSlicer/src/libslic3r/PrintConfig.cpp` into `ares-core` as read-only, source-cited option registry API data for future UI/API consumers and later variant-expansion milestones.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8154-8287`, covering `print_options_with_variant`, `filament_options_with_variant`, `printer_extruder_options`, `printer_options_with_variant_1`, `printer_options_with_variant_2`, and `empty_options`. No variant expansion, silent-mode resolution, extruder-count expansion, filament override behavior, option parsing behavior, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, new crate, or dependency is added.

## Exit checklist
- `ares-core` exposes read-only key-list functions for Orca's five non-empty variant option sets.
- Each exposed list preserves upstream `std::set` semantics as sorted unique keys; `printer_options_with_variant_1` deduplicates the repeated `nozzle_volume` initializer.
- Every exposed key resolves through `option_definition()` metadata.
- `empty_options` is documented as a zero-behavior upstream placeholder and remains unexposed unless a later source-cited consumer requires it.
- Public API coverage exists for all five lists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for variant expansion, silent mode, extruder count, filament overrides, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `DynamicPrintConfig::full_print_config()` and later behavior from `PrintConfig.cpp:8289+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
