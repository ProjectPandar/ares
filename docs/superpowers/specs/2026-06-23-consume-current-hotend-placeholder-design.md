# Consume Current Hotend Placeholder Design

## Source Boundary

This slice ports the `current_hotend` machine-start placeholder boundary from `OrcaSlicer/src/libslic3r/GCode.cpp`:

- `GCode.cpp:102-106` defines `hotend_id_for_gcode_placeholder`, returning `-1` for `printer_model == "Bambu Lab X2D"` and otherwise returning the supplied hotend id.
- `GCode.cpp:2779` computes the initial `extruder_id` from `initial_extruder_id`.
- `GCode.cpp:2821-2822` registers `current_extruder` and `current_hotend` after initializing the placeholder parser.
- `GCode.cpp:7936-7937` updates `current_extruder` and `current_hotend` during later tool changes; runtime tool changes are outside this startup-only slice.
- `PrintConfig.cpp:4957-4961` defines `printer_model` as a string option with an empty default.

## Rust Destination Boundary

Implement this slice in `ares-core` only:

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders `[current_hotend]` during `machine_start_gcode` placeholder replacement.
- `crates/ares-core/src/tests/current_hotend_placeholder_gcode.rs` covers rendered G-code through the public async `slice` API.
- `crates/ares-core/src/tests/mod.rs` registers the focused test module.

No CLI, WASM adapter, filesystem, UI, OpenGL, native-only behavior, new dependencies, option metadata additions, or full toolchange parser changes are included.

## Current Ares Runtime Mapping

Ares currently starts every print on logical extruder `0`, has no physical hotend remapping, and does not emit runtime tool changes. For this slice:

- `[current_hotend]` renders `0` in `machine_start_gcode` for missing `printer_model`, empty `printer_model`, and non-X2D printer models.
- `[current_hotend]` renders `-1` in `machine_start_gcode` when `printer_model` is exactly `Bambu Lab X2D`, matching Orca's `hotend_id_for_gcode_placeholder` special case.
- Non-string `printer_model` values are rejected only when `[current_hotend]` is used, because this placeholder is the first runtime consumer in this slice.
- `[current_hotend]` remains literal in `layer_change_gcode`; this slice does not broaden the layer-change placeholder scope.

## Included Behavior

1. `[current_hotend]` renders in `machine_start_gcode`.
2. The default and normal-printer startup value is `0`.
3. The X2D startup value is `-1`.
4. `[current_hotend]` composes with existing `[current_extruder]`, `[initial_tool]`, and `[num_extruders]` replacements.
5. Invalid non-string `printer_model` reaches `SliceError::InvalidInput` when the machine-start template references `[current_hotend]`.
6. Invalid non-string `printer_model` is ignored when `[current_hotend]` is not referenced by `machine_start_gcode`.
7. `[current_hotend]` remains literal in `layer_change_gcode`.

## Deferred Behavior

- `get_extruder_id` parity beyond Ares' current initial logical extruder `0`.
- `physical_extruder_map` and other hotend remapping behavior.
- Runtime updates from `GCode.cpp:7936-7937` during real tool changes.
- `[next_hotend]`, filament change dynamic config placeholders, and old/new hotend placeholders.
- Bambu X2D behavior outside `current_hotend` machine-start placeholder rendering.
- Placeholder expression/index evaluation beyond the existing literal replacement path.

## Acceptance Criteria

- A RED nextest run proves focused `[current_hotend]` machine-start tests fail while the placeholder is still literal.
- A GREEN nextest run proves default and normal-printer machine-start G-code renders `current_hotend` as `0`.
- A GREEN nextest run proves `printer_model == "Bambu Lab X2D"` renders `current_hotend` as `-1`.
- A GREEN nextest run proves `current_hotend` composes with existing current-extruder and nozzle-count placeholders.
- A GREEN nextest run proves layer-change G-code leaves `[current_hotend]` literal.
- A GREEN nextest run proves non-string `printer_model` is rejected when `[current_hotend]` is used.
- A GREEN nextest run proves non-string `printer_model` is accepted when `[current_hotend]` is not used.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, diff checks, and the touched Rust LOC guard.

## Safety And Rollback

The change is an additive machine-start placeholder replacement in platform-neutral `ares-core`. Rollback is removing the focused replacement helper, the new tests, the test-module registration, and this spec/plan pair. No persisted data, external services, dependencies, public crate boundaries, or platform-specific behavior are changed.
