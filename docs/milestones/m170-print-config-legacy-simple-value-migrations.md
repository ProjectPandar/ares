# M170: PrintConfig legacy simple value migrations

## Goal
Port the next simple `libslic3r::PrintConfigDef::handle_legacy` branch group from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7900-7932` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7900-7932` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization drops only the covered legacy percentage-valued keys when their incoming JSON value is a string containing `%`: `initial_layer_print_height`, `initial_layer_speed`, `internal_solid_infill_speed`, `top_surface_speed`, `support_interface_speed`, `outer_wall_speed`, and `support_object_xy_distance`.
- `SliceOptions` deserialization rewrites the covered cumulative/cooling/timelapse keys: `inherits_cummulative`, `compatible_printers_condition_cummulative`, `compatible_prints_condition_cummulative`, `cooling`, and `timelapse_no_toolhead`.
- `SliceOptions` deserialization rewrites only the covered values for `timelapse_type`, `support_type`, and `support_base_pattern`.
- Ordered `else if` semantics are preserved: a key alias branch does not also trigger a later value branch during the same normalization pass.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7933+`, including recursive `different_settings_to_system`, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
