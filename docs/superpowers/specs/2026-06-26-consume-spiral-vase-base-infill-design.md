# Consume Spiral Vase Base Infill Design

## Goal

Consume the next narrow OrcaSlicer `spiral_mode` slicing behavior in Ares: spiral vase mode must still generate the configured solid bottom base before the hollow vase body, instead of suppressing all infill when normalization forces sparse infill density to zero.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1560` declares `spiral_mode` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5678-5684` registers `spiral_mode` as a boolean option defaulting to false.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355-8369` normalizes spiral mode by forcing `wall_loops = 1`, `top_shell_layers = 0`, and `sparse_infill_density = 0`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1492-1514` limits solid-surface detection to `bottom_shell_layers` in spiral mode and reserves the last bottom-base layer for a top surface when the base has more than one layer.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1690-1695` marks the last bottom-base layer as `stTop` and marks layers after the base as `stInternal` in spiral mode.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:899-919` keeps top surfaces in spiral mode, keeps bottom surfaces when bottom shell layers are positive, and does not convert internal surfaces to dense solid solely because sparse density is 100.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:81-97` starts spiral perimeter generation only after the configured bottom shell layer/thickness boundary.

## Ares Boundary

- Keep the current Ares deterministic rectangle/scanline infill boundary in `crates/ares-core/src/infills.rs`; do not introduce a new pipeline.
- Extend `crates/ares-core/src/options/infill.rs` and a focused helper module under `crates/ares-core/src/options/infill/` so `InfillOptions` carries the minimal spiral-base state:
  - `spiral_mode` boolean parsed from `SliceOptions`.
  - The already parsed `ShellLayerOptions`, including normalized `bottom_shell_layers` and `top_shell_layers`.
- Extend infill path role metadata just enough to preserve spiral-base surface classification into print-path generation:
  - Add explicit `InfillRole::BottomSurface` and `InfillRole::TopSurface` solid routing roles.
  - These roles are generated only for spiral-base solid paths in this slice; existing non-spiral solid infill role behavior remains unchanged.
  - `InfillRole::as_str()` returns the existing `solid` label for these surface-solid paths; the required user-visible role distinction is the downstream `;PRINT_PATH:bottom_surface:` / `;PRINT_PATH:top_solid_infill:` classification.
- Update `crates/ares-core/src/print_paths/generate.rs` so spiral-base bottom/top surface infill routes directly to `PrintPathRole::BottomSurface` and `PrintPathRole::TopSolidInfill`, instead of being reclassified through `solid_infill_role(shell_layers, ...)`. This is required because spiral normalization sets `top_shell_layers = 0`, so shell-count-only classification would otherwise turn the last base layer into `bottom_surface`.
- Add local role logic under `crates/ares-core/src/options/infill/layer_role.rs`:
  - When `spiral_mode` is false, preserve all existing layer role behavior.
  - When `spiral_mode` is true and effective sparse density is zero, layers with index `< bottom_shell_layers` are generated as solid base layers.
  - The base layer count is `min(bottom_shell_layers, layer_count)`, matching Orca's `min(bottom_shell_layers, m_layers.size())` boundary.
  - In a multi-layer base, the final existing base layer is treated as `TopSurface`; earlier base layers are `BottomSurface`. This includes short models where `bottom_shell_layers > layer_count`.
  - In a single-layer base, bottom-surface precedence is retained and the layer is `BottomSurface`.
  - Layers at or above the computed base layer count remain sparse-role/empty because effective sparse density is zero.
- Update `crates/ares-core/src/infills.rs` so the early zero-density return does not bypass spiral vase base layers:
  - If density is zero and there are no spiral base layers, preserve the existing all-empty fast path.
  - If density is zero and spiral base layers exist, skip non-base layers before sparse spacing/path generation, and use solid-surface spacing/patterns only for base layers. The implementation must not compute sparse spacing with `line_width / 0`.
- Keep `crates/ares-core/src/gap_fills/solid_surface.rs` behavior scoped to the shared `InfillLayerRole` classification: spiral-base `BottomSurface`/`TopSurface` layers may be eligible for existing top/bottom solid-surface gap-fill targets, but this slice does not add new gap-fill path roles or new gap-fill geometry.
- Update focused pipeline tests in `crates/ares-core/src/pipeline/tests/spiral_mode_normalization.rs`, replace the stale `crates/ares-core/src/pipeline/tests/internal_solid_infill.rs::spiral_mode_density_100_normalizes_to_no_infill` expectation, and add infill tests under `crates/ares-core/src/infills/tests/`.
- If a touched Rust file would exceed 400 LOC, split only the code needed for this slice into an existing focused module. Do not perform unrelated cleanup.

## Included Behavior

- `spiral_mode=true` still uses existing `normalize_fdm(0)` behavior in `run_slicing_pipeline`.
- After normalization, Ares emits solid base infill paths for the configured `bottom_shell_layers` instead of returning no infill solely because effective sparse density is zero.
- With `bottom_shell_layers = 2`, a three-layer rectangular test pipeline emits:
  - layer 0 solid base paths that downstream print paths classify as `bottom_surface`;
  - layer 1 solid base paths that downstream print paths classify as `top_solid_infill`;
  - layer 2 no infill paths.
- With `bottom_shell_layers = 1`, the single base layer is downstream-classified as `bottom_surface`.
- With `bottom_shell_layers = 0`, spiral mode emits no infill paths.
- With `bottom_shell_layers` greater than the available layer count, all existing layers are treated as the base, and the final existing layer is the top surface when there is more than one layer.
- Existing non-spiral zero-density behavior remains empty.
- Existing sparse and dense non-spiral top/bottom shell behavior remains unchanged.
- Existing solid-surface gap-fill target semantics remain unchanged aside from following the same spiral-base `BottomSurface`/`TopSurface` layer classification.

## Deferred Behavior

- True continuous-Z spiral vase G-code, XY/Z smoothing, `spiral_mode_smooth`, `spiral_mode_max_xy_smoothing`, `spiral_starting_flow_ratio`, and `spiral_finishing_flow_ratio`.
- Exact Orca `SurfaceCollection`, `LayerRegion`, `fill_surfaces`, perimeter-generator, and sloped-vase geometric propagation parity.
- Bottom shell thickness as a spiral start boundary beyond existing Ares shell-thickness classification.
- CLI/user-facing automatic correction behavior beyond existing normalization and validation slices.
- Multi-region, support, bridge, raft, ironing, Arachne, and full Orca E2E output parity.

## Acceptance Criteria

1. A focused infill test proves `SliceOptions` with `spiral_mode=true`, `bottom_shell_layers=2`, and raw positive sparse density, after explicitly invoking `normalize_fdm(0)` before `infill_options()`, generates non-empty solid paths on the first two layers and no paths above the base.
2. A focused infill/print-path test proves the last layer of a multi-layer spiral base uses top-surface pattern/role selection and maps to `PrintPathRole::TopSolidInfill`, while earlier base layers map to `PrintPathRole::BottomSurface`.
3. A focused infill test proves `bottom_shell_layers > layer_count` still treats the final existing base layer as top surface when there is more than one existing layer.
4. A focused infill test proves `spiral_mode=true` with `bottom_shell_layers=0` emits no infill.
5. The existing non-spiral `density_zero_preserves_empty_infill_layers` test remains true.
6. A pipeline/G-code test proves `run_slicing_pipeline` still normalizes `wall_loops` to `1`, `top_shell_layers` to `0`, and `sparse_infill_density` to `0`, while final G-code now contains bottom/top base infill comments and no sparse infill comments.
7. A pipeline/G-code test replaces `spiral_mode_density_100_normalizes_to_no_infill` and proves spiral mode with raw 100% density now emits the bottom base instead of suppressing all infill.
8. A pipeline/G-code test proves layers above the spiral base contain no `sparse_infill`, `solid_infill`, `top_solid_infill`, or `bottom_surface` infill paths.
9. A gap-fill regression test proves existing top/bottom solid-surface gap-fill targeting still follows `BottomSurface`/`TopSurface` roles and does not add gap-fill paths on sparse-role layers above the spiral base.
10. Touched Rust files remain at or below 400 LOC.

## Verification

- RED before implementation:
  - `cargo nextest run -p ares-core spiral_mode`
- GREEN after implementation:
  - `cargo nextest run -p ares-core spiral_mode`
- Adjacent regression:
  - `cargo nextest run -p ares-core internal_solid density_zero shell_thickness top_bottom_solid_surface`
- Full verification after implementation review:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and its implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with this spec, the reviewed plan, diff, and fresh verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

Update `docs/roadmap.md` after implementation review to record that the spiral-vase base-infill slice is now consumed. No CLI or WASM usage docs change is required because the public byte-in/options-to-byte-output API shape remains unchanged.
