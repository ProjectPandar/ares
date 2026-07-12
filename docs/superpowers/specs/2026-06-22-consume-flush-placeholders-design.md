# Consume Flush Placeholders Design

## Goal

Consume OrcaSlicer `filament_flush_volumetric_speed` and `filament_flush_temp` as concrete machine-start G-code placeholder behavior in Ares. This slice ports the narrow Orca path that resolves zero flush values to existing filament/nozzle fallback options and exposes the resolved vectors as `flush_volumetric_speeds` and `flush_temperatures`, instead of adding more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1343-1344` declares `ConfigOptionFloatsNullable filament_flush_volumetric_speed` and `ConfigOptionIntsNullable filament_flush_temp`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2442-2460` registers `filament_flush_temp` default `[0]`, min `0`, max `max_temp`, and `filament_flush_volumetric_speed` default `[0]`, min `0`, max `200`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2462-2470` registers `filament_max_volumetric_speed`, whose default is already represented in Ares registry metadata as `[2]`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6495-6501` registers `nozzle_temperature_range_high` default `[240]`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2841-2853` copies flush speed and temperature vectors, replaces every zero flush speed with `filament_max_volumetric_speed.get_at(idx)`, replaces every zero flush temperature with `nozzle_temperature_range_high.get_at(idx)`, and registers `flush_volumetric_speeds` plus `flush_temperatures` in the placeholder parser.

## Destination Boundary

- `crates/ares-core/src/options/flush_placeholders.rs`: add a platform-neutral runtime accessor on `SliceOptions` that resolves the two placeholder vectors.
- `crates/ares-core/src/options.rs`: register the new internal options module by extending an existing compact module list, keeping the file at 400 LOC.
- `crates/ares-core/src/options/tests/flush_placeholders_runtime.rs`: focused runtime parsing and fallback tests.
- `crates/ares-core/src/options/tests.rs`: register the focused test module by extending an existing compact test-module list, keeping the file at 400 LOC.
- `crates/ares-core/src/gcode_placeholders.rs`: replace `[flush_volumetric_speeds]` and `[flush_temperatures]` in `machine_start_gcode`.
- `crates/ares-core/src/tests/flush_placeholders_gcode.rs`: end-to-end G-code tests for placeholder rendering and invalid input.
- `crates/ares-core/src/tests/mod.rs`: register the end-to-end test module.
- `docs/roadmap.md`: add this consumed runtime slice to the current progress list after implementation review.

## Design

`SliceOptions::flush_placeholders()` will parse the existing dynamic option keys `filament_flush_volumetric_speed`, `filament_flush_temp`, `filament_max_volumetric_speed`, and `nozzle_temperature_range_high`.

Missing `filament_flush_volumetric_speed` behaves like Orca's default `[0]`. Missing `filament_flush_temp` behaves like Orca's default `[0]`. Missing `filament_max_volumetric_speed` uses the existing Ares/Orca registry default `[2]`. Missing `nozzle_temperature_range_high` uses the existing Ares/Orca registry default `[240]`.

For each provided flush speed element, `0` is replaced by the corresponding max volumetric speed using Orca-style `get_at` fallback to the first available source value when the fallback vector is shorter than the flush vector. Non-zero flush speeds are preserved. The resolved output vector length stays equal to the flush speed vector length.

For each provided flush temperature element, `0` is replaced by the corresponding high nozzle temperature using the same fallback rule. Non-zero flush temperatures are preserved. The resolved output vector length stays equal to the flush temperature vector length.

`gcode_placeholders::machine_start_gcode(...)` will render `[flush_volumetric_speeds]` and `[flush_temperatures]` as comma-separated vectors formatted with Ares' existing placeholder-number style: integer-valued floats render without a decimal suffix, fractional floats keep their Rust string representation, and integer temperatures render as decimal integers. The rendered machine-start G-code remains part of existing startup-command suppression behavior.

Only the bracket placeholder form is included in this slice because Ares machine-start placeholders already use bracket tokens for auxiliary fan, adaptive bed mesh, and vitrification variables. `filament_cooling_before_tower`, full wipe-tower behavior, placeholder expression parsing, and tool-change flushing remain out of scope.

## Approaches Considered

1. **Chosen: add a small runtime accessor and render the two machine-start placeholders.** This directly consumes already-registered options into visible G-code behavior, follows the established Ares pattern for placeholder slices, and keeps the diff narrow.
2. Add a generic Orca custom G-code placeholder parser. This is too broad for one slice because Orca's parser handles expressions, conditionals, vectors, object context, and many variables.
3. Implement full flushing or wipe-tower behavior. That would consume more of the same option family but requires tool-change planning and purge logic that is outside the current Ares slice boundary.

## Included Behavior

- Missing `filament_flush_volumetric_speed` renders `[flush_volumetric_speeds]` as `2`.
- Missing `filament_flush_temp` renders `[flush_temperatures]` as `240`.
- `filament_flush_volumetric_speed = [0, 4.5]` and `filament_max_volumetric_speed = [2, 8]` render `2,4.5`.
- `filament_flush_temp = [0, 245]` and `nozzle_temperature_range_high = [260, 270]` render `260,245`.
- Fallback vectors shorter than flush vectors reuse their first value, so `[0, 0]` with `filament_max_volumetric_speed = [3.5]` renders `3.5,3.5`, and `[0, 0]` with `nozzle_temperature_range_high = [255]` renders `255,255`.
- Supported input forms follow existing Ares vector parsing: numbers, numeric strings with comma or semicolon separators, and arrays of numeric values for flush speeds; non-negative integers, integer strings with comma or semicolon separators, and arrays of integer values for flush temperatures and nozzle range highs.
- Invalid values return `SliceError::InvalidInput` mentioning the offending option key. Invalid cases include empty lists, negative flush speeds or temperatures, non-finite numbers, out-of-range flush speeds above `200`, wrong JSON containers, fractional integer vectors, and non-numeric values.
- Rendered `M104 S[flush_temperatures]` participates in existing automatic nozzle startup suppression after placeholder replacement.

## Deferred Behavior

- `filament_cooling_before_tower` placeholder behavior from the adjacent Orca lines.
- Full Orca placeholder parser parity, including brace-form placeholders, vector indexing, expression evaluation, conditionals, object context, and unknown placeholder semantics beyond existing Ares behavior.
- Tool-change flushing, wipe tower generation, prime tower behavior, purge-volume computation, and G-code emitted during filament changes.
- Nullable `"nil"` element semantics for these nullable Orca option types. This slice consumes numeric runtime behavior only; `"nil"` handling remains deferred until Ares ports nullable runtime vectors consistently.
- UI behavior and printer/filament profile editing behavior.

## Acceptance Criteria

1. `machine_start_gcode` containing `[flush_volumetric_speeds]` renders `2` when flush speed and max volumetric speed are absent.
2. `machine_start_gcode` containing `[flush_temperatures]` renders `240` when flush temperature and nozzle high range are absent.
3. Zero flush speed and temperature entries are replaced from the corresponding fallback vectors while non-zero entries are preserved.
4. Fallback vector `get_at` behavior reuses the first fallback value for indices beyond the fallback vector length.
5. Invalid flush placeholder inputs fail slicing with `SliceError::InvalidInput` mentioning the offending option key.
6. A rendered nozzle command such as `M104 S[flush_temperatures]` suppresses Ares' automatic first-layer nozzle startup command and appears before `;LAYER_CHANGE`.
7. `crates/ares-core` remains platform-neutral and WASM-compatible.
8. Every touched Rust file remains at or below 400 LOC.

## Verification Plan

- RED: add focused runtime and G-code tests, then run `cargo nextest run -p ares-core flush_placeholders` and confirm the new behavior fails before implementation.
- GREEN: implement the accessor and placeholder rendering, then rerun the focused command.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core flush_placeholders`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Documentation

Update `docs/roadmap.md` with this source-cited consumed runtime slice after implementation review approves the diff.
