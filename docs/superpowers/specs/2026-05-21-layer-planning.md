# Layer Planning Spec

## Goal
Advance Ares from STL import metadata to the first real FFF slicing stage: derive model Z bounds, read the first typed Orca-compatible layer-height options, plan print layers, and emit deterministic layer-aware G-code comments/moves through the existing `ares slice` command.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` defines `layer_height` and `initial_layer_height`/`first_layer_height` option surfaces, and validates `layer_height > 0` near the print-config validation path.
- `OrcaSlicer/src/libslic3r/Slicing.cpp` maps print/object config into `SlicingParameters`, including `layer_height`, first-layer height, min/max layer height, and layer-height profiles.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` creates `Layer` objects with height, `print_z`, and `slice_z`, then slices volumes at the planned Z positions.
- `OrcaSlicer/src/libslic3r/GCode.cpp` emits layer Z metadata such as `;Z:` and uses `layer_z` in placeholder/config substitution.

## Scope
Milestone 3 adds layer planning and layer-aware placeholder G-code only. It does not yet compute XY cross-section polygons, perimeters, infill, extrusion E values, support, or Orca G-code parity. The output remains deterministic metadata plus Z travel moves, but it is driven by model geometry and typed options.

## Functional requirements
1. `SliceOptions` continues preserving all dynamic Orca option keys and gains typed accessors for:
   - `layer_height`, default `0.2` mm.
   - `initial_layer_height`, defaulting to `layer_height` when absent.
2. Typed layer heights must be finite positive numbers. Invalid values return `SliceError::InvalidInput` from `slice`/layer planning.
3. `Model` exposes Z bounds from imported triangles. Models with no triangles, non-finite bounds, or no positive Z height cannot produce a layer plan.
4. `ares-core` exposes a public layer planning API:
   ```rust
   pub fn plan_layers(model: &Model, options: &SliceOptions) -> Result<Vec<Layer>, SliceError>
   ```
5. `Layer` contains deterministic `id`, `height`, and `print_z` fields. `id` is zero-based, `height` is the layer thickness, and `print_z` is the absolute Z position.
6. Layer planning starts at `z_min + initial_layer_height`, then advances by `layer_height`, and clamps the final layer to exactly `z_max` when needed. Planning must tolerate `f32` STL coordinate precision without creating duplicate near-zero final layers.
7. `slice` calls `load_model`, then `plan_layers`, and emits G-code bytes containing:
   - `; input_format = stl`
   - `; triangle_count = N`
   - `; layer_height = H`
   - `; initial_layer_height = H0`
   - `; layer_count = N`
   - One block per layer with `;LAYER_CHANGE`, `;LAYER:<id>`, `;Z:<print_z>`, and `G1 Z<print_z>`.
8. `ares-cli` behavior remains unchanged at the argument/filesystem boundary and observes the new core output through existing `ares slice --options option.json -o output.gcode input.stl`.
9. No new crates or dependencies are introduced.

## Non-goals
- No cross-section polygon generation or actual extrusion paths.
- No `min_layer_height`, `max_layer_height`, adaptive layer height, support layer height, or full Orca option typing in this milestone.
- No 3MF geometry extraction.
- No WASM binding crate.

## Acceptance criteria
- Core tests cover default layer-height accessors, custom `layer_height`, custom `initial_layer_height`, invalid layer-height rejection for both typed options, model Z bounds, flat/no-triangle/non-finite bound rejections, layer planning with final-layer clamp, no duplicate near-zero final layer from f32 bounds, and `slice` layer-aware G-code output.
- CLI tests prove `ares slice --options option.json -o output.gcode input.stl` writes layer-aware output for an STL with positive Z height.
- `docs/roadmap.md` and `docs/milestones/m3-layer-planning.md` describe the milestone and defer full path generation.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation review returns APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
