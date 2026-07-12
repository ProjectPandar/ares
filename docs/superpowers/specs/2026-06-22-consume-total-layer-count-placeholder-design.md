# Consume Total Layer Count Placeholder Design

## Goal

Consume OrcaSlicer's `total_layer_count` custom G-code placeholder as concrete Ares `machine_start_gcode` behavior. This slice makes an already planned slice result visible to custom start G-code instead of adding more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2855` sets `total_layer_count` on the main placeholder parser from `m_layer_count`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3082` sets reserved placeholders and then processes `machine_start_gcode` through the placeholder parser before automatic startup temperature suppression.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10927-10929` defines `total_layer_count` as a custom placeholder config value with tooltip "Number of layers in the entire print."
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11112-11113` lists `machine_start_gcode` as a custom G-code type with no type-specific placeholder allowlist, so globally registered placeholders such as `total_layer_count` are valid there.

## Destination Boundary

- `crates/ares-core/src/gcode.rs`: pass the actual planned layer count from `pipeline.layers().len()` into start G-code formatting.
- `crates/ares-core/src/gcode_start_custom.rs`: carry `total_layer_count` through the start-G-code command boundary.
- `crates/ares-core/src/gcode_adaptive_bed_mesh.rs`: forward `total_layer_count` beside existing adaptive bed mesh placeholders.
- `crates/ares-core/src/gcode_placeholders.rs`: replace `[total_layer_count]` in `machine_start_gcode`.
- `crates/ares-core/src/tests/custom_gcode_end.rs`: add focused G-code coverage if the file stays under 400 LOC; otherwise create a new focused test module.
- `docs/roadmap.md`: record this source-cited runtime slice after implementation review.

## Current State

Ares already builds all layers before formatting G-code and already routes `machine_start_gcode` through `gcode_placeholders::machine_start_gcode(...)`. That renderer consumes several Orca machine-start placeholders, including auxiliary fan values, flush placeholders, `filament_cooling_before_tower`, `min_vitrification_temperature`, and adaptive bed mesh values.

Ares does not currently replace `[total_layer_count]`. Existing layer-change custom G-code tests intentionally preserve `[total_layer_count]` as unknown in layer-change scope; this slice does not change layer-change, time-lapse, role-change, end-G-code, or file-start placeholder behavior.

## Design

`format_gcode(...)` will compute the value from `pipeline.layers().len()` and pass it into `gcode_start_custom::StartGCodeCommand`. This keeps the placeholder tied to the actual planned print result, not to user-provided option metadata or text parsing.

`gcode_start_custom::start_gcode(...)` will pass that count through `gcode_adaptive_bed_mesh::machine_start_gcode(...)`, which already owns the path from start G-code formatting into `gcode_placeholders`.

`gcode_placeholders::machine_start_gcode(...)` will replace only the bracket token `[total_layer_count]` with the decimal integer count. Unknown placeholders and brace expressions remain unchanged, matching the existing lightweight Ares placeholder policy for start G-code.

The rendered machine-start G-code remains the string used for existing startup temperature suppression, so a command that includes `[total_layer_count]` in a temperature value is evaluated after replacement before suppression checks run.

## Included Behavior

- A two-layer slice of the existing pyramid fixture renders `machine_start_gcode = ";LAYERS [total_layer_count]"` as `;LAYERS 2` before `;LAYER_CHANGE`.
- Bracket replacement composes with existing machine-start placeholders, for example `[total_layer_count] [min_vitrification_temperature]`.
- Unknown machine-start placeholders remain unchanged.
- Layer-change custom G-code still preserves `[total_layer_count]` as unknown because Orca's type-specific layer-change path in Ares has not been widened in this slice.
- No new option key, registry entry, dependency, crate, or public API is added.

## Deferred Behavior

- Full Orca placeholder parser parity, including brace-form placeholders, expression evaluation, conditionals, vector indexing, and custom config definition validation.
- Other nearby global placeholders from `GCode.cpp:2808-3037`, including `initial_tool`, `current_extruder`, `num_extruders`, `retract_length`, first-layer print bounds, print bed bounds, `max_print_z`, chamber aliases, and filament compatibility flags.
- `total_layer_count` in layer-change, time-lapse, role-change, end-G-code, file-start, filament-start, or filament-end scopes.
- Sequential object placeholder behavior, wipe tower/tool-change placeholder behavior, support-aware first-layer hull parity, and UI behavior.

## Acceptance Criteria

1. A focused G-code test proves `[total_layer_count]` in `machine_start_gcode` renders the actual planned layer count before the first `;LAYER_CHANGE`.
2. A focused test proves `[total_layer_count]` composes with an existing machine-start placeholder in the same template.
3. Existing tests continue to prove unknown placeholders outside this slice remain unchanged, including the existing layer-change `[total_layer_count]` preservation test.
4. The implementation updates `docs/roadmap.md` with the source-cited runtime slice and deferred behavior.
5. Verification uses `cargo nextest run`, not `cargo test`.
6. Touched Rust source files remain at or below 400 LOC.

## Verification Plan

- RED: add the focused G-code tests, then run `cargo nextest run -p ares-core total_layer_count` and confirm the new behavior fails before implementation.
- GREEN: implement the narrow data flow and placeholder replacement, then rerun the focused command.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core total_layer_count`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Safety

The change is additive and local to already rendered `machine_start_gcode`. It reads only in-memory pipeline state and keeps `ares-core` platform-neutral and WASM-compatible. It must not add filesystem access, terminal behavior, UI behavior, OpenGL behavior, feature flags, dependencies, or legacy fallback paths.
