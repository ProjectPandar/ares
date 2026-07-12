# Hardware Option Typing Spec

## Goal
Advance option parity by typing the first OrcaSlicer machine/filament vector options needed by later extrusion and flow milestones while continuing to preserve unknown Orca keys dynamically.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` defines `layer_height` as `coFloat` with default `INITIAL_LAYER_HEIGHT`, `initial_layer_height` as `coFloat` with default `0.3`, `filament_diameter` as `coFloats` with default `[1.75]`, `min_layer_height` as `coFloats` with default `[0.07]`, `max_layer_height` as `coFloats` with default `[0.0]`, and `nozzle_diameter` as `coFloats` with default `[0.4]`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp::validate` rejects `layer_height <= 0`, `filament_diameter < 1`, and `nozzle_diameter < 0.005`.
- Orca profile JSON under `OrcaSlicer/resources/profiles/**` stores vector options both as JSON arrays of strings, for example `"nozzle_diameter": ["0.4"]`, and as semicolon-separated strings, for example `"nozzle_diameter": "0.4;0.2;0.6;0.8"`.
- Ares already preserves unknown options as `serde_json::Value`; M6 adds typed accessors for the hardware subset without adding profile inheritance or a new crate.

## Scope
Milestone 6 adds a focused hardware option boundary inside `ares-core`. It parses Orca-compatible numeric scalar/vector JSON forms for nozzle and filament diameters plus machine layer-height bounds. The existing `SliceOptions` dynamic map remains the source of truth for unknown and future options.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub struct HardwareOptions { ... }
   impl SliceOptions {
       pub fn hardware_options(&self) -> Result<HardwareOptions, SliceError>;
       pub fn nozzle_diameters(&self) -> Result<Vec<f64>, SliceError>;
       pub fn filament_diameters(&self) -> Result<Vec<f64>, SliceError>;
       pub fn min_layer_heights(&self) -> Result<Vec<f64>, SliceError>;
       pub fn max_layer_heights(&self) -> Result<Vec<f64>, SliceError>;
   }
   ```
2. Defaults match the researched Orca definitions for this subset:
   - `nozzle_diameter`: `[0.4]`
   - `filament_diameter`: `[1.75]`
   - `min_layer_height`: `[0.07]`
   - `max_layer_height`: `[0.0]`
3. Each vector accessor accepts:
   - missing key, returning the default vector;
   - JSON number, returning a one-element vector;
   - JSON string containing one number;
   - semicolon-separated numeric string such as `"0.4;0.2;0.6;0.8"`;
   - comma-separated numeric string such as `"0.4,0.6"`;
   - JSON array of numbers;
   - JSON array of numeric strings.
4. Each vector accessor rejects empty strings, empty arrays, `nil`, booleans, objects, nested arrays, non-numeric strings, non-finite numbers, and invalid threshold values with `SliceError::InvalidInput`.
5. Thresholds:
   - `nozzle_diameter` values must be at least `0.005`.
   - `filament_diameter` values must be at least `1.0`.
   - `min_layer_height` and `max_layer_height` values must be finite and non-negative.
6. Existing `layer_height()` and `initial_layer_height()` behavior remains unchanged in this milestone.
7. `slice()` emits top-level metadata for the typed hardware options:
   - `; nozzle_diameter = v1[,v2...]`
   - `; filament_diameter = v1[,v2...]`
   - `; min_layer_height = v1[,v2...]`
   - `; max_layer_height = v1[,v2...]`
8. `ares slice --options option.json -o output.gcode input.stl` accepts Orca-style array-of-string hardware options and writes the metadata above.
9. Unknown Orca keys continue to round-trip through `SliceOptions::values()` without typed parsing.
10. No new crates or dependencies are introduced.
11. `ares-core` stays platform-neutral and performs no filesystem I/O.
12. Modified Rust files remain under 400 LOC.

## Non-goals
- No profile inheritance/merge graph, preset discovery, or loading Orca resource directories.
- No extrusion E values, flow calculations, line width calculations, filament cost/weight, temperature, acceleration, or speed behavior.
- No change to `layer_height`/`initial_layer_height` defaults or semantics beyond relocating code if needed to keep files small.
- No new workspace crates.

## Acceptance criteria
- Core tests cover defaults, JSON numbers, numeric strings, semicolon strings, comma strings, arrays of numbers, arrays of strings, invalid thresholds, invalid non-numeric/nested values, unknown-key preservation, and `slice` hardware metadata output.
- CLI tests prove Orca-style array-of-string options for nozzle/filament/layer bounds are accepted and emitted.
- Docs include `docs/milestones/m6-hardware-option-typing.md`, this spec, an implementation plan, roadmap update, and an ARD for keeping option typing inside `ares-core` for now.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation review returns APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
