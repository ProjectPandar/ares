# Render `enable_high_low_temp_mix` in machine start G-code

## Source boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:3003` registers the `enable_high_low_temp_mix` placeholder as `!print.need_check_multi_filaments_compatibility()`.
- `OrcaSlicer/src/libslic3r/Print.hpp:1083-1084` exposes `set_check_multi_filaments_compatibility` and `need_check_multi_filaments_compatibility`.
- `OrcaSlicer/src/libslic3r/Print.hpp:1186` initializes `m_need_check_multi_filaments_compatibility` to `true`, which makes the placeholder false unless the application disables the compatibility check.
- `OrcaSlicer/src/libslic3r/AppConfig.cpp:415-416` defaults `enable_high_low_temp_mixed_printing` to `false`.
- `OrcaSlicer/src/slic3r/GUI/Plater.cpp:7952` sets `Print::set_check_multi_filaments_compatibility(app_config["enable_high_low_temp_mixed_printing"] == "false")`, making the G-code placeholder equal to the app preference.

## Current Ares boundary

- Ares already renders machine-start placeholders in `crates/ares-core/src/gcode_machine_start_placeholders.rs`.
- Ares has no GUI `AppConfig` layer and no `Print::m_need_check_multi_filaments_compatibility` state yet.
- `SliceOptions::bool_option` already parses boundary boolean options and returns a default when the key is absent.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` and `crates/ares-core/src/gcode.rs` are both at 400 LOC, so this slice must not add net lines to those files. New behavior should live in the existing `gcode_machine_start_extruder_used_placeholders.rs` helper, renamed to a broader machine-start runtime placeholder helper if needed.

## Behavior to implement

Render `[enable_high_low_temp_mix]` in `machine_start_gcode` as a bool placeholder:

- If `enable_high_low_temp_mixed_printing` is absent, render `0`, matching Orca's default `m_need_check_multi_filaments_compatibility{true}` and AppConfig default `false`.
- If `enable_high_low_temp_mixed_printing` is `true`, render `1`.
- If `enable_high_low_temp_mixed_printing` is `false`, render `0`.
- Non-boolean `enable_high_low_temp_mixed_printing` values return `SliceError::InvalidInput` mentioning that key.
- The placeholder is expanded only in `machine_start_gcode`; the same token in layer-change or other custom G-code scopes remains literal unless those scopes later port an upstream behavior.
- Existing machine-start placeholders including `[is_extruder_used]`, `[num_extruders]`, and stat reserved placeholders continue to compose with this placeholder.

## Deferred upstream behavior

- Do not add GUI/AppConfig storage, compatibility-warning logic, material-temperature compatibility validation, or a `Print`-level `need_check_multi_filaments_compatibility` field in this slice.
- Do not add public API, dependencies, feature flags, file I/O, terminal behavior, UI behavior, or non-WASM-safe behavior.
- This is a machine-start placeholder rendering slice only.

## Implementation shape

- Rename `crates/ares-core/src/gcode_machine_start_extruder_used_placeholders.rs` to `crates/ares-core/src/gcode_machine_start_runtime_placeholders.rs`.
- Keep `is_extruder_used` rendering in that helper and add `enable_high_low_temp_mix` rendering there.
- Register the renamed helper in `crates/ares-core/src/lib.rs`.
- Reuse the existing single helper call in `crates/ares-core/src/gcode_machine_start_placeholders.rs`, passing `&SliceOptions` so the helper can parse `enable_high_low_temp_mixed_printing`.
- Add focused tests in `crates/ares-core/src/tests/enable_high_low_temp_mix_placeholder_gcode.rs` and register the module in `crates/ares-core/src/tests/mod.rs`.

## Acceptance criteria

- A default slice with `machine_start_gcode = ";MIX [enable_high_low_temp_mix]"` emits `;MIX 0` before `;LAYER_CHANGE`.
- With `enable_high_low_temp_mixed_printing = true`, the same placeholder emits `;MIX 1`.
- With `enable_high_low_temp_mixed_printing = false`, the same placeholder emits `;MIX 0`.
- Invalid non-boolean input for `enable_high_low_temp_mixed_printing` reaches `SliceError::InvalidInput` and includes the option key.
- `[enable_high_low_temp_mix]` remains literal in `layer_change_gcode`.
- The placeholder composes with `[is_extruder_used]` without changing `is_extruder_used` vector behavior.
- Focused RED/GREEN verification uses `cargo nextest run -p ares-core enable_high_low_temp_mix_placeholder_gcode`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a Rust LOC guard for touched Rust files.
