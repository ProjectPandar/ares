# Consume Outer Wall Volumetric Speed Placeholder Design

## Goal

Consume the existing OrcaSlicer `outer_wall_line_width`, `outer_wall_speed`, `filament_max_volumetric_speed`, `layer_height`, `nozzle_diameter`, and `filament_diameter` option parsing into concrete machine start G-code behavior by rendering `[outer_wall_volumetric_speed]`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:508-522` defines `get_outer_wall_volumetric_speed`. It uses the default region `outer_wall_line_width`, falls back through `line_width` and nozzle/filament-derived defaults, builds an outer wall `Flow`, multiplies `outer_wall_speed` by `Flow::mm3_per_mm()`, then caps the result by `filament_max_volumetric_speed`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3028-3030` registers the computed value as the machine start placeholder `outer_wall_volumetric_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1093-1094` declares `outer_wall_line_width` and `outer_wall_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1333` declares `filament_max_volumetric_speed`.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` will render `[outer_wall_volumetric_speed]` only in `machine_start_gcode`, matching the existing machine-start-only placeholder model.
- The calculation will reuse `SliceOptions::extrusion_options()`, `SliceOptions::speed_options()`, `SliceOptions::layer_height()`, `ExtrusionOptions::extrusion_per_mm(PrintPathRole::ExternalPerimeter, layer_height)`, `SpeedOptions::external_perimeter_speed_mm_s()`, and `SpeedOptions::filament_max_volumetric_speed_mm3_s()`.
- Because `ExtrusionOptions::extrusion_per_mm` returns filament E distance per mm, the runtime placeholder must convert back to material volume per mm by multiplying by the parsed first filament cross-section area before multiplying by outer wall speed. This keeps the calculation aligned with Ares' current extrusion geometry and with Orca's `outer_wall_speed * outer_wall_flow.mm3_per_mm()` boundary.
- The result is capped with `filament_max_volumetric_speed` using the same first-filament value already parsed into `SpeedOptions`.

## Included Behavior

- A machine start template containing `[outer_wall_volumetric_speed]` renders the computed mm3/s value before the first `;LAYER_CHANGE`.
- The calculation reacts to `outer_wall_speed`, `outer_wall_line_width`, `line_width` fallback behavior already encoded by `ExtrusionOptions`, `layer_height`, `nozzle_diameter`, `filament_diameter`, `filament_flow_ratio`, `print_flow_ratio`, `set_other_flow_ratios`, `outer_wall_flow_ratio`, and `filament_max_volumetric_speed`.
- If the computed outer wall volumetric speed exceeds `filament_max_volumetric_speed`, the rendered value is the cap. If the cap is `0`, the rendered value is `0`.
- Invalid input already rejected by the reused option parsers continues to surface as `SliceError::InvalidInput`.
- `[outer_wall_volumetric_speed]` remains literal outside machine start G-code, including `layer_change_gcode`.

## Deferred Behavior

- Do not port the full Orca `Flow` type or change Ares extrusion geometry in this slice.
- Do not add per-object, per-region, support-extruder, multi-filament selection, or multi-extruder routing beyond Ares' current first-filament/default-region parsing.
- Do not add new option metadata or candidate crates.
- Do not change generated print move speeds, volumetric speed limiting, adaptive volumetric speed smoothing, or other runtime G-code beyond this placeholder.
- Do not render the placeholder in file start, layer change, filament start, or end G-code scopes.

## Acceptance Criteria

1. Focused RED tests demonstrate that `[outer_wall_volumetric_speed]` is not rendered before implementation.
2. Focused GREEN tests show machine start rendering uses `outer_wall_speed * external perimeter material mm3/mm`, capped by `filament_max_volumetric_speed`.
3. Tests cover an uncapped value, a capped value, the zero-cap case, invalid input propagation, and literal preservation in layer change scope.
4. Implementation touches only the focused core G-code/options test surface needed for the placeholder and keeps touched Rust files at or below 400 LOC.
5. Verification uses `cargo nextest run`, not `cargo test`, with focused tests, adjacent related tests, full workspace tests, clippy, wasm check, format check, diff checks, and LOC guard before commit.
6. The commit uses the repository Lore commit protocol and is pushed to the current branch upstream `origin/codex/consume-slicing-options`, matching the user's explicit commit/push objective for this active thread.

## Test Strategy

- Add `crates/ares-core/src/tests/outer_wall_volumetric_speed_placeholder_gcode.rs`.
- Register it from `crates/ares-core/src/tests/mod.rs`.
- Use `slice(square_pyramid_ascii_stl(), options)` to prove rendered G-code, not just parser output.
- Compute expected test values from Ares' current external perimeter area formula:
  - material area per mm: `layer_height * (outer_wall_line_width - layer_height * (1 - PI / 4))`
  - placeholder value: `min(outer_wall_speed * material_area_per_mm * flow ratios, filament_max_volumetric_speed)`
- Run focused command `cargo nextest run -p ares-core outer_wall_volumetric_speed_placeholder`.
- Run adjacent command `cargo nextest run -p ares-core filament_max_volumetric_speed speed_gcode flush_placeholders_gcode nozzle_temperature_gcode`.

## Verification Commands

- `cargo fmt --check`
- `cargo nextest run -p ares-core outer_wall_volumetric_speed_placeholder`
- `cargo nextest run -p ares-core filament_max_volumetric_speed speed_gcode flush_placeholders_gcode nozzle_temperature_gcode`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- `for file in crates/ares-core/src/gcode_machine_start_placeholders.rs crates/ares-core/src/tests/mod.rs crates/ares-core/src/tests/outer_wall_volumetric_speed_placeholder_gcode.rs; do test "$(wc -l < "$file")" -le 400 || exit 1; done`

## Docs Impact

No user-facing documentation update is required in this slice because the repository does not currently have a dedicated placeholder reference document. The behavior is documented by this SDD spec, the implementation plan, and focused G-code regression tests.

## Safety

The change is platform-neutral Rust in `ares-core`, does not perform file I/O, terminal I/O, UI, OpenGL, networking, or native-only operations, and adds no dependencies.
