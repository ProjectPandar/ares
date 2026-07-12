# Consume First Layer Bed Temperature Placeholder Design

## Goal

Port OrcaSlicer's `machine_start_gcode` placeholder `[first_layer_bed_temperature]` into Ares G-code output by consuming the existing first-layer bed temperature options. This is a concrete start-G-code placeholder behavior slice, not new option metadata and not a redesign of the existing Ares startup bed-temperature command.

## Upstream Boundary

Line citations are pinned to the checked-out `OrcaSlicer` source in this repository.

- `OrcaSlicer/src/libslic3r/GCode.cpp:2987-3008` resolves the current bed type, obtains `first_bed_temp_opt` through `get_bed_temp_1st_layer_key((BedType)curr_bed_type)`, computes the formula-selected `target_bed_temp` separately, and registers `first_layer_bed_temperature` as `ConfigOptionInts(*first_bed_temp_opt)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` through the placeholder parser before automatic startup bed-temperature emission checks the rendered start G-code.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:489-509` maps `BedType` values to the first-layer bed temperature option keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489-1501` declares `curr_bed_type` and the bed temperature option vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041` registers first-layer bed temperature defaults: SuperTack Plate `35`, Cool Plate `35`, Textured Cool Plate `40`, Engineering Plate `45`, High Temp Plate `45`, and Textured PEI Plate `45`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2503-2510` registers `bed_temperature_formula`; this formula affects `bed_temperature_initial_layer_single` and automatic bed startup temperature selection, but not the vector placeholder registered as `first_layer_bed_temperature`.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:906-943` expands vector placeholders by `current_extruder_id`, falling back to index `0` when the index is outside the vector.
- `OrcaSlicer/src/libslic3r/Config.hpp:1023-1040` serializes `ConfigOptionInts` values as integer strings.

## Current Ares State

- Ares already consumes first-layer bed temperature options into automatic startup bed G-code through `SliceOptions::first_layer_bed_temperature()` and existing `M190 S...` integration tests.
- Ares already handles `curr_bed_type`, default first-layer bed temperature values, vector parsing, `bed_temperature_formula`, and custom start G-code temperature suppression.
- Ares already renders nearby `machine_start_gcode` placeholders such as `[first_layer_temperature]`, `[max_print_height]`, `[first_layer_height]`, `[z_offset]`, `[retract_length]`, `[num_extruders]`, and `[total_layer_count]` in `crates/ares-core/src/gcode_placeholders.rs`.
- Ares currently leaves `[first_layer_bed_temperature]` literal in `machine_start_gcode`.

## Ares Destination Boundary

- `crates/ares-core/src/options/bed_temperature.rs`: expose a crate-private accessor that returns the selected first-layer bed temperature vector without applying `bed_temperature_formula`.
- `crates/ares-core/src/gcode_placeholders.rs`: render `[first_layer_bed_temperature]` in `machine_start_gcode` from the first value of that selected vector.
- `crates/ares-core/src/tests/bed_temperature_gcode.rs`: add focused integration tests for the placeholder and reuse existing bed temperature startup tests as adjacent coverage.

## Included Behavior

1. `machine_start_gcode` containing `[first_layer_bed_temperature]` renders before the first generated layer G-code.
2. `curr_bed_type` selects the same first-layer bed temperature key and defaults already used by automatic startup bed G-code:
   - `Cool Plate` -> `cool_plate_temp_initial_layer`, default `35`
   - `Textured Cool Plate` -> `textured_cool_plate_temp_initial_layer`, default `40`
   - `Engineering Plate` -> `eng_plate_temp_initial_layer`, default `45`
   - `High Temp Plate` -> `hot_plate_temp_initial_layer`, default `45`
   - `Textured PEI Plate` -> `textured_plate_temp_initial_layer`, default `45`
   - `SuperTack Plate` and `Supertack Plate` -> `supertack_plate_temp_initial_layer`, default `35`
3. Configured scalar integer, integer string, semicolon/comma-separated integer string list, and non-empty integer arrays use the existing bed temperature vector parser.
4. For multi-value first-layer bed temperature vectors, Ares renders the first value. This matches Orca's initial-extruder `0` case until Ares ports Orca's current/initial extruder routing.
5. Placeholder rendering is independent of `bed_temperature_formula`: `[first_layer_bed_temperature]` renders the first vector element, while the existing automatic `M190` startup command continues to use `first_layer_bed_temperature()` and therefore still applies `by_highest_temp` or `by_first_filament`.
6. Invalid selected first-layer bed temperature values and invalid `curr_bed_type` still return `SliceError::InvalidInput` through the existing option parsing path.
7. `[first_layer_bed_temperature]` remains unexpanded in layer-change, before-layer-change, time-lapse, end, and filament custom G-code scopes.

## Deferred Behavior

- `bed_temperature_initial_layer`, `bed_temperature_initial_layer_single`, `bed_temperature_initial_layer_vector`, `[bed_temperature]`, explicit vector indexing such as `[first_layer_bed_temperature_1]`, full Orca placeholder expression parsing, nonzero current-extruder selection, toolchange-specific placeholder routing, chamber/overall temperature placeholders, and full multi-filament bed-temperature scheduling remain deferred.
- This slice does not add registry entries, new crates, dependencies, feature flags, UI behavior, filesystem behavior, terminal behavior, OpenGL behavior, or independent Ares pipeline behavior.

## Tests

Add tests named with the `machine_start_first_layer_bed_temperature` prefix:

- `machine_start_first_layer_bed_temperature_renders_configured_value`: `machine_start_gcode = ";BED [first_layer_bed_temperature]"` and `cool_plate_temp_initial_layer = 47` render `;BED 47` before `;LAYER_CHANGE`.
- `machine_start_first_layer_bed_temperature_defaults_to_selected_bed_type`: `curr_bed_type = "Textured Cool Plate"` and no configured temperature render `;BED 40`.
- `machine_start_first_layer_bed_temperature_uses_initial_extruder_value`: `cool_plate_temp_initial_layer = [35, 65, 45]` renders `;BED 35` while the existing automatic startup bed command still emits `M190 S65` under the default `by_highest_temp` formula.
- `machine_start_first_layer_bed_temperature_accepts_numeric_string_and_composes`: a numeric string/list value composes with nearby machine-start placeholders and renders the first integer.
- `machine_start_first_layer_bed_temperature_does_not_expand_in_layer_change_scope`: `layer_change_gcode = ";LC [first_layer_bed_temperature] [layer_num]"` preserves `[first_layer_bed_temperature]` while expanding `[layer_num]`.
- `machine_start_first_layer_bed_temperature_rejects_invalid_values`: invalid selected bed temperature values and invalid `curr_bed_type` return `SliceError::InvalidInput`.

Focused RED/GREEN verification uses `cargo nextest run -p ares-core machine_start_first_layer_bed_temperature`.

## Acceptance Criteria

1. Focused nextest proves `[first_layer_bed_temperature]` is rendered in `machine_start_gcode` with configured values, selected-bed defaults, multi-value first-entry behavior, numeric string/list values, and composition with existing machine-start placeholders.
2. Focused nextest proves the placeholder does not expand in layer-change scope.
3. Focused nextest proves invalid selected bed temperature and invalid bed type inputs return `SliceError::InvalidInput`.
4. Adjacent nextest proves existing automatic bed startup command behavior still applies `bed_temperature_formula` and remains unchanged.
5. `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass before commit.

## Safety

The change is additive and limited to existing in-memory custom start G-code rendering. It uses existing validated option parsing and keeps `ares-core` platform-neutral and WASM-compatible. No filesystem, terminal, UI, OpenGL, networking, or native-only behavior is introduced.
