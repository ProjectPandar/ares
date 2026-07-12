# Consume Spiral Mode Normalization Design

## Source Boundary

This slice ports the existing Orca configuration normalization boundary for `spiral_mode` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp`. In `DynamicPrintConfig::normalize_fdm` / `normalize_fdm_1`, Orca checks `spiral_mode` and forces the slicing inputs that make vase-mode geometry hollow: `wall_loops = 1`, `alternate_extra_wall = false`, `top_shell_layers = 0`, `sparse_infill_density = 0`, and layer-change retraction arrays are disabled. The related validation boundary in `PrintConfig.cpp` reports CLI errors when those values conflict with spiral vase mode, but the normalization branch is the concrete behavior this slice consumes.

## Ares Boundary

Ares already implements the source-cited normalization logic in `crates/ares-core/src/options/fdm_normalization.rs`, but the normal `slice(...)` / `run_slicing_pipeline(...)` path currently reads the original `SliceOptions` directly. That means `spiral_mode: true` does not yet affect generated perimeters, infill paths, print paths, or G-code unless a caller manually invokes `normalize_fdm`.

This slice connects the existing normalization to the core slicing boundary:

- `crates/ares-core/src/pipeline.rs` should normalize a cloned `SliceOptions` before deriving perimeters, infills, extrusion options, speeds, diagnostics, and G-code-facing artifacts.
- `crates/ares-core/src/lib.rs::slice` should format G-code with the same normalized options used to build the pipeline.
- `serde_json::from_value::<SliceOptions>(...)` must remain a pure option container load; deserialization should not normalize.

## Behavior

When a caller passes `spiral_mode: true` to `slice(...)`, Ares must use the existing `normalize_fdm(0)` behavior before generating geometry and G-code. As a result:

- requested `wall_loops` values greater than one are consumed as one perimeter loop in generated paths and G-code;
- requested `sparse_infill_density` values above zero are consumed as zero sparse infill paths and no sparse-infill G-code;
- the generated G-code header should report normalized option-derived diagnostics, including the changed print path and infill counts;
- invalid `spiral_mode` or invalid spiral retraction array values should surface through the slicing path as `SliceError::InvalidInput`.

When `spiral_mode` is absent or `false`, existing slicing behavior must remain unchanged.

This slice does not implement Orca's `SpiralVase` G-code post-processor, smooth spiral XY filtering, spiral starting/finishing flow transitions, true continuous-Z vase extrusion, top/bottom solid layer generation, support disabling, or CLI-only spiral validation. It only consumes the already-ported `normalize_fdm` option effects in the concrete Ares slicing/G-code path.

## Acceptance Criteria

- A G-code regression test with `spiral_mode: true`, `wall_loops: 3`, and positive `sparse_infill_density` shows exactly one perimeter path and zero sparse infill paths in output.
- The same test proves sparse-infill G-code comments/moves are absent while perimeter G-code still emits.
- A pipeline-level regression test proves `run_slicing_pipeline(...)` applies the same normalized options to diagnostics and generated artifacts.
- A regression test proves invalid `spiral_mode` reaches `SliceError::InvalidInput` through `slice(...)`.
- Existing `SliceOptions` deserialization behavior remains unchanged: raw `spiral_mode` options are not normalized until the slicing boundary.
- `cargo test -p ares-core --lib` passes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust file LOC gate pass.
