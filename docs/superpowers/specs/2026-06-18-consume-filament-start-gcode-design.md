# Consume Filament Start G-code Design

## Goal

Consume the existing `filament_start_gcode` option in `ares-core` by emitting the active filament start template during G-code startup, before the first layer begins.

This is a concrete slicing/G-code behavior slice. It must not add new option metadata except for the runtime accessor needed to consume the already-registered option.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1387` declares `filament_start_gcode` as `ConfigOptionStrings` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5798-5804` defines the option label, tooltip, multiline UI behavior, and default `ConfigOptionStrings { " " }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3114-3115` processes the active initial filament start template after `machine_start_gcode` for Bambu printer startup.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3134-3138` documents the non-wipe-tower filament-specific G-code path that writes the initial filament start template.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11132` lists `filament_start_gcode` placeholders as `{filament_extruder_id}`.

## Ares Destination Boundary

- `crates/ares-core/src/options/custom_gcode.rs` owns string custom G-code accessors for `SliceOptions`.
- `crates/ares-core/src/gcode_placeholders.rs` owns direct placeholder replacement for custom G-code templates until the full Orca placeholder parser is ported.
- `crates/ares-core/src/gcode.rs` owns the current G-code startup sequence.
- `crates/ares-core/src/tests/custom_gcode_end.rs` owns start/end custom G-code runtime tests.
- `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs` currently owns accessor tests for custom G-code runtime options.

## Included Behavior

1. Add a `SliceOptions::filament_start_gcode()` accessor that accepts:
   - a string value,
   - a string array value, returning the first string as the active filament template,
   - an empty string array as no-op,
   - absent value as no-op.
2. Reject invalid `filament_start_gcode` values with `SliceError::InvalidInput`, including non-string scalars, objects, null, and arrays containing non-strings.
3. Add a renderer for `filament_start_gcode` that replaces both brace and bracket forms of the only upstream-listed placeholder:
   - `{filament_extruder_id}`
   - `[filament_extruder_id]`
4. Emit rendered filament start G-code after the existing `machine_start_gcode` output and before the first `;LAYER_CHANGE`.
5. Use active first filament semantics for the current Ares pipeline:
   - select the first configured string in the array,
   - use `filament_extruder_id = 0`.
6. Preserve existing default output when the option is absent, empty, or an empty array.
7. Preserve unknown placeholders, conditionals, and expressions literally.
8. Keep `ares-core` platform-neutral and WASM-safe.

## Deferred Behavior

These upstream branches are intentionally out of scope for this slice:

- full Orca placeholder parser expression and conditional evaluation,
- Bambu-specific `;VT` marking,
- pressure advance startup behavior,
- first-layer extruder temperature startup behavior,
- wipe tower control of filament start G-code,
- multi-filament active extruder selection beyond the first configured template,
- tool-change-time `filament_start_gcode`,
- interaction with `change_filament_gcode`,
- Orca E2E parity for default `" "` output.

Future expansion must cite the exact upstream branch it ports before changing this behavior.

## Acceptance Criteria

- Runtime tests prove `filament_start_gcode` is emitted after `machine_start_gcode` and before the first layer boundary.
- Runtime tests prove brace and bracket `filament_extruder_id` replacement.
- Runtime tests prove unknown conditionals/expressions are preserved.
- Runtime tests prove invalid values reach `SliceError::InvalidInput`.
- Runtime tests prove absent, empty string, and empty array values preserve default output.
- Accessor tests prove string, string array, empty array, absent, and invalid inputs.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted tests, full `cargo test -p ares-core --lib`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass before commit.

## Design Notes

This slice mirrors the just-ported `filament_end_gcode` shape because upstream stores both as `ConfigOptionStrings` and current Ares has no active extruder state. The first configured string is the narrowest active-filament approximation that turns existing metadata into behavior while keeping future multi-material parity work explicit and source-cited.
