# M3: Layer planning and first typed options

## Goal
Use imported STL geometry and the first typed layer-height options to create deterministic FFF layer plans and layer-aware G-code metadata.

## Exit checklist
- `SliceOptions` preserves dynamic keys while exposing typed `layer_height` and `initial_layer_height` accessors.
- Invalid non-positive or non-finite layer heights return typed `SliceError::InvalidInput`.
- `Model` exposes finite Z bounds from imported triangles.
- `ares-core` exposes `plan_layers(model, options) -> Result<Vec<Layer>, SliceError>`.
- Layer planning starts at `z_min + initial_layer_height`, advances by `layer_height`, and clamps the final planned layer to `z_max`.
- `slice` emits `layer_height`, `initial_layer_height`, `layer_count`, `;LAYER_CHANGE`, `;LAYER:<id>`, `;Z:<print_z>`, and `G1 Z<print_z>` metadata/moves.
- `ares-cli` continues owning filesystem behavior and observes the new layer-aware core output.
- No new crates or dependencies are introduced.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No polygon slicing, perimeters, infill, supports, extrusion E values, or Orca G-code parity.
- No additional option families beyond `layer_height` and `initial_layer_height`.
- No 3MF geometry extraction.
