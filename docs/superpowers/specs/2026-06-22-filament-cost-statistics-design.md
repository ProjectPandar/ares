# Filament Cost Statistics Design

## Goal

Consume OrcaSlicer `filament_cost` as concrete Ares G-code statistics output, not as metadata. The slice ports the single-extruder subset of Orca's filament statistics path so a configured filament price changes generated G-code comments after slicing.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1330` declares `((ConfigOptionFloats, filament_cost))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2837-2843` defines the option as `Price`, `money/kg`, default `0.`, minimum `0`, and "For statistics only."
- `OrcaSlicer/src/libslic3r/GCode.cpp:2279-2343` implements `update_print_stats_and_format_filament_stats`, deriving used filament length, extruded volume, weight, and cost from extruder statistics.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3471-3488` writes those filament statistics near the G-code footer.

## Ares Destination Boundary

- Add a focused `crates/ares-core/src/gcode_filament_stats.rs` module for statistics formatting and option parsing.
- Wire it through the existing footer path so statistics are written after filament/machine custom end G-code and before the final `M2`.
- Add focused pipeline coverage in `crates/ares-core/src/pipeline/tests/filament_cost_gcode.rs`.

## Included Behavior

- Parse `filament_cost` from Orca-style numeric vector forms already used in Ares: number, numeric string, comma/semicolon string list, or JSON array.
- Default missing `filament_cost` to `0`.
- Reject negative, non-finite, empty, nested, object, null, and non-numeric values with `SliceError::InvalidInput`.
- Use Ares' current single-extruder pipeline scope:
  - validate all configured vector entries for `filament_cost` and `filament_density`;
  - use the first vector entry for `filament_cost`, `filament_density`, and `filament_diameter`, matching Ares' current single-extruder runtime convention;
  - ignore additional valid vector entries until a later multi-extruder statistics slice;
  - total used filament length is the sum of `LayerExtrusionMoves::total_extrusion_mm()`;
  - extruded volume is `used_filament_mm * pi * (filament_diameter_mm / 2)^2`;
  - filament weight is `extruded_volume_mm3 * filament_density_g_cm3 * 0.001`;
  - filament cost is `filament_weight_g * filament_cost_money_per_kg * 0.001`.
- Emit Orca-style statistics comments:
  - `; filament used [mm] = ...`
  - `; filament used [cm3] = ...`
  - `; filament used [g] = ...` only when density produces positive weight
  - `; filament cost = ...` only when cost produces a positive cost
- Format statistics values with two decimal places to match the upstream `%.2lf` behavior.
- Preserve movement and extrusion G-code commands for equal geometry and print options.
- Keep statistics outside BTT thumbnail header suppression; that suppression only skips Ares' header block, while this slice writes footer statistics.
- Apply optional line numbering after the statistics are inserted, preserving the existing final line-numbering pass.

## Output Placement

The statistics block must be emitted after `filament_end_gcode` and `machine_end_gcode` expansion and before the final `M2` emitted by `crates/ares-core/src/gcode_finish.rs`. This keeps custom shutdown commands before the report while avoiding statistics after a program-end command.

## Docs Impact

Update `docs/roadmap.md` with a dated runtime slice entry for `filament_cost` after implementation review. No public API documentation changes are required because the behavior is reached through existing `SliceOptions` and generated G-code comments.

## Deferred Behavior

- Multi-extruder per-tool output and zero filling for inactive extruders.
- Wipe tower material accounting.
- Orca `PrintStatistics` storage, UI statistics, total non-BBL footer lines, estimated time placeholders, and full config block generation.
- Header formatting changes for `filament_density`.
- Exact BBL/non-BBL printer branching.

## Acceptance Criteria

- A default slice emits used filament length and volume statistics, but no cost line.
- A slice with positive `filament_density` and positive `filament_cost` emits weight and cost lines whose cost is derived from the generated extrusion total and configured filament diameter.
- `filament_cost = 0` or `filament_density = 0` suppresses the cost line.
- Invalid `filament_cost` values fail through `format_gcode` with `SliceError::InvalidInput`.
- Existing movement/extrusion command lines do not change when only `filament_cost` changes.
- `cargo nextest run -p ares-core filament_cost` covers the focused behavior.
