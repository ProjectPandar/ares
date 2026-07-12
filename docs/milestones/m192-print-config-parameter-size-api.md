# M192: PrintConfig get_parameter_size API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::get_parameter_size` into Ares as a read-only `SliceOptions::parameter_size(param_name, extruder_nums)` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8529-8556` plus declaration context in `PrintConfig.hpp:633` and the already-ported M184 key-set data from `PrintConfig.cpp:8154-8287`. It covers only read-only parameter-size calculation. No `PrintConfig.cpp:8558+` extruder-variant extension, `set_num_extruders`, `set_num_filaments`, vector resizing, `FullPrintConfig::defaults`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- Missing variant source keys default filament/process/machine variant lengths to `1`.
- `printer_options_with_variant_1` keys return machine variant length.
- `printer_options_with_variant_2` keys return doubled machine variant length.
- `filament_options_with_variant` keys return filament variant length.
- `print_options_with_variant` keys return process variant length.
- Other keys return `extruder_nums`.
- Invalid present variant source values return `SliceError::InvalidInput`.
- Existing M184 variant key-set API behavior remains intact.
- `PrintConfig.cpp:8558+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
