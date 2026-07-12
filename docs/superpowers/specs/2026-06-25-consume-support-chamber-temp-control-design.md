# Consume Support Chamber Temperature Control Design

## Goal

Consume the existing OrcaSlicer `support_chamber_temp_control` option into Ares chamber-temperature G-code behavior. Ares must stop emitting automatic chamber control commands when the printer profile says chamber temperature control is unsupported, while preserving the current default behavior for profiles that omit the option.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1407` declares `support_chamber_temp_control` on `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3772-3777` defines it as `coBool`, labels it "Support control chamber temperature", documents `M141 S(0-255)`, and defaults it to `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1636-1637` declares the already-consumed per-filament `activate_chamber_temp_control` and `chamber_temperature` options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448-6468` defines the user-facing chamber activation and temperature options, including the automatic `M191` before machine start and `M141` at print end behavior.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2973-3097` computes active chamber control, sets chamber placeholders, and writes startup chamber temperature G-code when active.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3451-3452` writes chamber shutdown G-code when active.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:192-209` formats `M191` and `M141`.

This slice ports the `support_chamber_temp_control` capability gate into the already-existing Ares chamber-temperature control path. It does not port a new chamber pipeline or add option metadata.

## Ares Destination Boundary

- `crates/ares-core/src/options/chamber_temperature.rs` owns chamber-temperature option parsing and the `ChamberTemperatureControl` value consumed by G-code startup and finish generation.
- `crates/ares-core/src/options/tests/chamber_temperature_runtime.rs` owns focused runtime parsing tests for chamber control.
- `crates/ares-core/src/gcode_startup.rs` and `crates/ares-core/src/gcode_finish.rs` already consume `ChamberTemperatureControl` to emit `M191` and `M141`; this slice should not duplicate that writer logic.

## Included Behavior

1. `support_chamber_temp_control` defaults to `true`, matching Orca, so existing Ares profiles that omit the option keep current automatic chamber behavior.
2. When `support_chamber_temp_control` is `false`, `SliceOptions::chamber_temperature_control()` returns `ChamberTemperatureControl::disabled()` even if `activate_chamber_temp_control` is true and `chamber_temperature` is positive.
3. When `support_chamber_temp_control` is `true`, the current activation and maximum chamber temperature behavior stays unchanged.
4. Invalid `support_chamber_temp_control` values are rejected as `SliceError::InvalidInput` at the options boundary.
5. `chamber_temperature_values()` and `overall_chamber_temperature()` remain based only on `chamber_temperature`; disabling supported chamber control must not erase placeholders such as `[chamber_temperature]` or `[overall_chamber_temperature]`.
6. Because startup and finish G-code already consume `ChamberTemperatureControl`, disabling the control must suppress automatic `M191` startup output and automatic `M141 S0` finish output through the existing path.

## Deferred Behavior

- No UI behavior, printer capability database, firmware probing, or machine-profile migration.
- No new G-code writer formatting changes, including Orca's auxiliary-fan side effect around `M191`.
- No changes to Klipper suppression or custom start-G-code suppression for existing `M141`/`M191` commands.
- No `hold_chamber_temp_for_flat_print`, material chamber temperature range validation, or heat-soak macro behavior.
- No changes to placeholder rendering except preserving existing chamber placeholder values while the capability gate disables automatic commands.
- No new dependencies, crates, filesystem behavior, terminal behavior, UI, OpenGL, or WASM-incompatible code.

## Acceptance Criteria

- A focused RED test demonstrates that `support_chamber_temp_control: false` currently still enables chamber control when activation and temperature are set.
- After implementation, `cargo nextest run -p ares-core chamber_temperature_runtime` passes.
- Tests prove the default/true path still enables `ChamberTemperatureControl::enabled(max_temperature)` when activation is true and chamber temperature is positive.
- Tests prove the false path returns `ChamberTemperatureControl::disabled()` and leaves `chamber_temperature_values()` plus `overall_chamber_temperature()` intact.
- Tests prove invalid non-boolean `support_chamber_temp_control` values are rejected with an error mentioning that option key.
- Full verification before commit includes `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.
