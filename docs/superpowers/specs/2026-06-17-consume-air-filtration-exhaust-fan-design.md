# Consume Air Filtration Exhaust Fan Options Design

## Goal

Consume the existing Orca air-filtration / exhaust-fan options as concrete G-code behavior in Ares, instead of only preserving their registry metadata. This slice emits `M106 P3 S...` commands for the exhaust fan during startup and completion using the same upstream boundaries Orca uses for air filtration.

## User-facing Scope

This slice consumes only existing options:

- `support_air_filtration`
- `activate_air_filtration`
- `activate_air_filtration_during_print`
- `activate_air_filtration_on_completion`
- `during_print_exhaust_fan_speed`
- `complete_print_exhaust_fan_speed`

It must not add option metadata, new registry keys, new crates, new dependencies, or independently designed fan behavior.

## Upstream Source Boundary

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1405` defines `support_air_filtration`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1522-1526` defines air-filtration activation vectors plus `during_print_exhaust_fan_speed` and `complete_print_exhaust_fan_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1800-1835` defines `activate_air_filtration`, during-print/on-completion activation defaults, and exhaust fan speed ranges/defaults.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3779-3783` defines printer support for air filtration.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3144-3156` emits the during-print exhaust fan command by checking activated extruders and taking the maximum configured during-print speed.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3455-3464` emits the completion exhaust fan command by checking activated extruders and taking the maximum configured completion speed.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1157-1165` formats exhaust fan G-code as `M106 P3 S{int(speed / 100.0 * 255)}`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:109` declares `set_exhaust_fan`.

## Ares Destination Boundary

- Add a focused `crates/ares-core/src/options/exhaust_fan.rs` runtime accessor module.
- Register the module from `crates/ares-core/src/options.rs` by adding `exhaust_fan` to an existing compact `option_modules!(...)` line; do not add a new line because `options.rs` is already at the 400 LOC limit.
- Add `GCodeWriter::set_exhaust_fan(speed: u8) -> String` in `crates/ares-core/src/gcode_writer.rs`.
- Add startup/completion helpers in `crates/ares-core/src/gcode_startup.rs` or the smallest nearby G-code helper boundary.
- Wire `crates/ares-core/src/gcode.rs` to emit during-print exhaust fan command before `;LAYER_CHANGE` and completion command before `M2`, after existing chamber shutdown behavior. Keep `gcode.rs` under 400 LOC; if wiring grows beyond a few lines, move phase formatting into `gcode_startup.rs` rather than expanding `gcode.rs`.

## Runtime Semantics

### Parsing

`support_air_filtration` is a printer-level boolean:

- Missing default: `true`, matching registry metadata.
- Accepted forms: JSON bool and lowercase string `"true"` / `"false"`.
- Rejected forms: arrays, empty strings, uppercase booleans, numbers, objects, null.

Activation options are extruder/vector booleans:

- `activate_air_filtration` missing default: `[false]`.
- `activate_air_filtration_during_print` missing default: `[true]`.
- `activate_air_filtration_on_completion` missing default: `[true]`.
- Accepted forms: JSON bool, lowercase boolean string, JSON bool array, semicolon/comma-separated lowercase boolean string.
- Arrays must contain JSON booleans only.
- Empty arrays, empty strings/separators, uppercase strings, numbers, objects, null are invalid.

Speed options are extruder/vector integer percentages:

- `during_print_exhaust_fan_speed` missing default: `[60]`.
- `complete_print_exhaust_fan_speed` missing default: `[80]`.
- Accepted forms: JSON integer number, integer string, JSON integer-number array, semicolon/comma-separated integer string.
- Arrays must contain JSON integer numbers only; string elements inside arrays are invalid.
- Values must be `0..=100`; invalid input returns `SliceError::InvalidInput` naming the offending key.

### Selection

Ares currently emits single-tool output but can receive vector profile values. To mirror Orca's multi-extruder max behavior without adding tool scheduling:

- If `support_air_filtration` is false, emit no exhaust fan commands even if process activation values are true.
- For each phase, iterate indexes in `0..max_len`, where `max_len` is the maximum length of `activate_air_filtration`, that phase's activation vector, and that phase's speed vector.
- During print is active when any iterated index has both `activate_air_filtration[index]` and `activate_air_filtration_during_print[index]` true. Missing per-index values use the last available value for that option vector, matching a conservative repeated-profile interpretation for Ares until true extruder scheduling exists.
- Completion is active when any iterated index has both `activate_air_filtration[index]` and `activate_air_filtration_on_completion[index]` true, using the same iteration and last-value fallback rule.
- The emitted speed is the maximum speed at active indexes for the corresponding phase.
- Active phase with speed `0` emits `M106 P3 S0`; inactive phase emits no command.

### G-code Formatting and Placement

- `GCodeWriter::set_exhaust_fan(60)` emits `M106 P3 S153\n` because Orca truncates `speed / 100.0 * 255`.
- `set_exhaust_fan(80)` emits `M106 P3 S204\n`.
- `set_exhaust_fan(0)` emits `M106 P3 S0\n`.
- Klipper output skips exhaust fan commands, matching the current Ares pattern for startup hardware commands.
- During-print exhaust fan command is emitted after existing bed/nozzle/chamber startup commands and before the first `;LAYER_CHANGE`.
- Completion exhaust fan command is emitted before final `M2`. If chamber shutdown is present, keep the existing chamber shutdown first, then emit completion exhaust fan, then `M2`.

## Tests

Runtime option tests:

- Defaults: supported printer but inactive filtration emits disabled runtime control.
- Scalar/string/array/separated bool and integer forms are accepted.
- `support_air_filtration=false` disables both phases.
- Any active index plus max corresponding speed is selected for during-print and completion.
- Active speed zero remains enabled and returns speed zero.
- Invalid support, activation, and speed values return `SliceError::InvalidInput` with the key.

Writer tests:

- `set_exhaust_fan(0) == "M106 P3 S0\n"`.
- `set_exhaust_fan(60) == "M106 P3 S153\n"`.
- `set_exhaust_fan(80) == "M106 P3 S204\n"`.
- `set_exhaust_fan(100) == "M106 P3 S255\n"`.

Slice G-code tests:

- Default output emits no `M106 P3` lines.
- Enabled during-print filtration emits `M106 P3 S153` before first layer.
- Enabled completion filtration emits `M106 P3 S204` before `M2`.
- When both phases are enabled, both commands appear in correct order.
- Klipper skips both exhaust fan commands.
- Invalid exhaust fan speed reaches `SliceError::InvalidInput`.

Final verification:

- Independent SDD spec reviewer approval is required before planning.
- Independent SDD plan reviewer approval is required before implementation.
- Independent SDD implementation reviewer approval is required before commit/push.
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `crates/ares-core/src` Rust LOC gate: every `*.rs` file is `<=400` lines.

## Out of Scope

- No UI behavior.
- No user documentation changes are expected beyond the SDD spec and implementation plan for this slice.
- No custom placeholder expansion for exhaust fan variables.
- No Bambu-specific auxiliary fan `P2` behavior.
- No fan speedup/kickstart/FanMover behavior.
- No multi-tool scheduling beyond vector max selection for current single-output Ares.
- No changes to active `gcode_flavor` parsing.
