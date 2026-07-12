# Consume Temperature Vitrification Placeholder Design

## Goal

Consume OrcaSlicer `temperature_vitrification` as concrete machine-start G-code placeholder behavior in Ares. This slice ports the narrow Orca path where filament softening temperatures are aggregated and exposed to custom start G-code as `min_vitrification_temperature`, instead of adding more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1332` declares `((ConfigOptionInts, temperature_vitrification))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2828-2835` registers `temperature_vitrification` as `coInts`, labels it "Softening temperature", and defaults it to `ConfigOptionInts{ 100 }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2982-2984` computes `min_temperature_vitrification` as the minimum `m_config.temperature_vitrification.get_at(extruder.id())` across writer extruders.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3004` registers that value with the placeholder parser as `min_vitrification_temperature`.

## Destination Boundary

- `crates/ares-core/src/options/temperature_vitrification.rs`: add a platform-neutral runtime accessor on `SliceOptions`.
- `crates/ares-core/src/options.rs`: register the new internal options module without growing the file past 400 LOC.
- `crates/ares-core/src/options/tests/temperature_vitrification_runtime.rs`: focused runtime parsing tests.
- `crates/ares-core/src/options/tests.rs`: register the focused test module without growing the file past 400 LOC.
- `crates/ares-core/src/gcode_placeholders.rs`: replace `[min_vitrification_temperature]` in `machine_start_gcode`.
- `crates/ares-core/src/tests/custom_gcode_end.rs`: add end-to-end machine-start G-code tests if the file remains under 400 LOC; otherwise create a new focused integration test module and register it in `crates/ares-core/src/tests/mod.rs`.
- `docs/roadmap.md`: add this consumed runtime slice to the current progress list.

## Design

`SliceOptions::temperature_vitrification()` will parse the existing dynamic option key `temperature_vitrification` as Orca-style non-negative integer vector data using the existing `temperature_vector::parse_integer_vector` helper. Missing values return the Orca default vector `[100]`. The accessor returns the minimum value in the parsed vector, matching Orca's minimum-over-extruders behavior for the currently represented Ares option values.

`gcode_placeholders::machine_start_gcode(...)` will render `[min_vitrification_temperature]` with the accessor result before the machine-start G-code participates in existing automatic startup command suppression. This keeps the behavior inside the existing core G-code API and avoids filesystem, UI, terminal, or native-only dependencies.

Only the bracket placeholder form is included in this slice because Ares machine-start placeholders already use bracket tokens for auxiliary fan and adaptive bed mesh variables. Brace-form placeholder parser parity remains deferred.

## Approaches Considered

1. **Chosen: add a small runtime accessor and render one machine-start placeholder.** This directly consumes the existing option into visible G-code behavior, reuses the established integer-vector parser, and keeps the diff narrow.
2. Add a generic full custom G-code placeholder parser. This is closer to long-term Orca parity but far too broad for one slice because Orca's parser handles expressions, conditionals, vectors, and many variables.
3. Implement `filament_cost` statistics instead. That is a valid future option-consumption slice, but it touches print statistics and end-of-file reporting rather than the existing start-G-code placeholder path.

## Included Behavior

- Missing `temperature_vitrification` renders `[min_vitrification_temperature]` as `100` in `machine_start_gcode`.
- Scalar integer, separated string integer list, and integer array forms are accepted consistently with existing Ares `ConfigOptionInts` temperature parsing.
- Multiple values render the minimum value.
- Empty lists, fractional values, negative values, non-numeric values, and wrong JSON container types return `SliceError::InvalidInput` mentioning `temperature_vitrification`.
- The rendered machine-start G-code is emitted before the first layer and still participates in existing suppression of automatic startup commands. For example, `machine_start_gcode = "M140 S[min_vitrification_temperature]"` suppresses the automatic `M190 S...` bed startup command and emits the rendered `M140`.

## Deferred Behavior

- Full Orca placeholder parser parity, including brace-form start placeholders, expression evaluation, conditionals, vector indexing, and unknown placeholder semantics beyond existing Ares behavior.
- Orca's full writer-extruder model and `ConfigOptionInts::get_at` last-value fallback semantics beyond taking the minimum of the provided Ares vector.
- Other nearby Orca placeholders in `GCode.cpp:2996-3010`, including bed temperature vectors, chamber temperature, high/low temp mix, first-layer temperature aliases, printable height, and z-offset.
- UI guidance about opening doors or removing glass.
- `filament_cost` statistics and any other filament metadata behavior.

## Acceptance Criteria

1. `machine_start_gcode` containing `[min_vitrification_temperature]` renders `100` when `temperature_vitrification` is absent.
2. `temperature_vitrification = [105, 95, 110]` renders `95`.
3. `temperature_vitrification = "102;98"` renders `98`.
4. Invalid `temperature_vitrification` inputs fail slicing with `SliceError::InvalidInput` mentioning the option key.
5. A rendered `M140 S[min_vitrification_temperature]` in `machine_start_gcode` suppresses Ares' automatic bed startup command and appears before `;LAYER_CHANGE`.
6. `crates/ares-core` remains platform-neutral and WASM-compatible.
7. Every touched Rust file remains at or below 400 LOC.

## Verification Plan

- RED: add the focused runtime and G-code tests, then run `cargo nextest run -p ares-core temperature_vitrification min_vitrification_temperature` and confirm the new behavior fails before implementation.
- GREEN: implement the accessor and placeholder rendering, then rerun the focused command.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core temperature_vitrification min_vitrification_temperature`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Documentation

Update `docs/roadmap.md` with this source-cited consumed runtime slice after implementation review approves the diff.
