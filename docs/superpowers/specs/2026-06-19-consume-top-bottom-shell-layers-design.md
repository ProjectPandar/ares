# Consume Top And Bottom Shell Layers Design

## Goal

Consume OrcaSlicer's `bottom_shell_layers` and `top_shell_layers` as concrete slicing behavior in the current Ares deterministic solid-infill scaffold. The existing Ares pipeline already turns `sparse_infill_density == 100` into solid infill and has bottom/top solid surface roles; this slice makes the configured shell layer counts decide how many generated solid layers use those roles instead of keeping the current hard-coded first-layer/last-layer behavior.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1079` declares `bottom_shell_layers` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1167` declares `top_shell_layers` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1119-1128` registers `bottom_shell_layers` as an integer option, minimum `0`, default `3`, with tooltip text stating it is the number of bottom solid shell layers including the bottom surface layer.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6564-6573` registers `top_shell_layers` as an integer option, minimum `0`, default `4`, with tooltip text stating it is the number of top solid shell layers including the top surface layer.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10123-10128` validates `top_shell_layers` and `bottom_shell_layers` as non-negative values.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10223-10225` rejects spiral-vase CLI slicing when top solid layers remain greater than zero.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:83-89` treats `bottom_shell_layers` as a layer-count boundary when deciding whether spiral mode can start after bottom shell layers.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:503-518` begins expanding top/bottom/bridge surfaces into shell-thickness solid infill. Full surface expansion remains deferred here.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:901-916` turns top or bottom surfaces into internal surfaces when their corresponding shell layer count is zero, then turns internal sparse infill into internal solid infill when density is 100%.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:41` documents `B = bottom_shell_layers` and `T = top_shell_layers` in fill rotation range counts.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:129-131` reads `bottom_shell_layers` and `top_shell_layers` as count tokens for infill angle ranges.

## Ares Boundary

- Add a small platform-neutral shell-layer-count type in `crates/ares-core/src/print_paths.rs` and export it from `crates/ares-core/src/lib.rs`.
- Add a focused `crates/ares-core/src/options/shell_layers.rs` parser that reads `bottom_shell_layers` and `top_shell_layers` with Orca defaults `3` and `4`, using the existing non-negative integer parser.
- Change `generate_print_paths` to accept shell-layer counts and classify only generated `InfillRole::Solid` paths:
  - A layer index less than `bottom_shell_layers` maps to `PrintPathRole::BottomSurface`.
  - Otherwise, a layer index inside the last `top_shell_layers` generated layers maps to `PrintPathRole::TopSolidInfill`.
  - Otherwise, solid infill maps to `PrintPathRole::SolidInfill`.
  - `bottom_shell_layers == 0` disables bottom-surface classification.
  - `top_shell_layers == 0` disables top-surface classification.
  - If bottom and top ranges overlap, bottom has precedence. This keeps the existing single-layer behavior when both counts are positive and gives a deterministic scaffold behavior until Ares ports Orca's full geometric surface partitioning.
- Update `run_slicing_pipeline` and `pipeline::test_support` to pass `options.shell_layer_options()?` into `generate_print_paths`.
- Keep sparse infill, internal solid generation, extrusion widths/flows/speeds, top/bottom solid numeric hooks, print-domain role mapping, G-code formatting, CLI adapters, WASM adapters, and option registry metadata unchanged except where their tests need explicit shell layer counts to preserve an intended top/bottom/interior role mix.

## Included Behavior

- The pipeline consumes already registered `bottom_shell_layers` and `top_shell_layers`; it does not add new option metadata.
- Missing options use Orca defaults: bottom `3`, top `4`.
- Explicit integer or numeric-string values are accepted when non-negative and integral.
- Invalid shell layer values return `SliceError::InvalidInput` through the existing parser path.
- With `sparse_infill_density == 100`, generated solid infill roles are classified by shell layer counts:
  - `bottom_shell_layers = 2`, `top_shell_layers = 1`, and 5 generated layers produce bottom, bottom, solid, solid, top.
  - `bottom_shell_layers = 0`, `top_shell_layers = 2`, and 4 generated layers produce solid, solid, top, top.
  - `bottom_shell_layers = 2`, `top_shell_layers = 0`, and 4 generated layers produce bottom, bottom, solid, solid.
  - `bottom_shell_layers = 3`, `top_shell_layers = 4`, and 5 generated layers produce bottom, bottom, bottom, top, top because bottom precedence resolves the overlap.
- Existing top/bottom solid numeric behavior remains role-based: top-surface width/flow/speed/acceleration/jerk hooks apply when paths are classified as `TopSolidInfill`, and bottom flow plus first-layer behavior apply when paths are classified as `BottomSurface`.

## Deferred Behavior

- Full Orca `LayerRegion` surface geometry: actual top/bottom exposed surface detection, bridge surface detection, shell-thickness expansion, sparse-infill-adjacent solid shell expansion, and `SurfaceCollection` partitioning.
- `top_shell_thickness` and `bottom_shell_thickness`; this slice consumes count options only.
- Top/bottom surface density, patterns, ironing, gap fill, support surfaces, bridge-specific bottom surfaces, and exact Orca E2E output parity.
- Changing infill rotation template `B`/`T` count token behavior. Ares already has a separate source-cited rotation-template scaffold; this slice is limited to print-path role classification in the current 100% solid-infill pipeline.

## Acceptance Criteria

1. Unit tests prove `SliceOptions::shell_layer_options()` uses defaults `bottom = 3`, `top = 4`.
2. Unit tests prove explicit `bottom_shell_layers` and `top_shell_layers` values are parsed and non-negative integer validation rejects invalid values through `SliceError::InvalidInput`.
3. Print-path tests prove shell counts classify multi-layer solid infill into bottom, interior solid, and top ranges by layer index.
4. Print-path tests prove `bottom_shell_layers = 0` disables bottom-surface classification.
5. Print-path tests prove `top_shell_layers = 0` disables top-surface classification.
6. Print-path tests prove overlapping ranges prefer bottom classification.
7. Pipeline/G-code tests prove changing shell layer counts changes emitted `bottom_surface`, `solid_infill`, and `top_solid_infill` role strings for `sparse_infill_density == 100`.
8. Existing top/bottom solid surface numeric tests are updated with explicit shell counts where they require a top/bottom/interior role mix, and they continue to prove role-specific width/flow/speed/acceleration/jerk behavior.
9. Existing spiral-mode normalization and validation tests for `top_shell_layers` and `bottom_shell_layers` continue to pass.
10. All touched Rust files under `crates/` remain at or below 400 LOC.

## Verification

- RED/GREEN targeted tests:
  - `cargo test -p ares-core --lib shell_layer_options`
  - `cargo test -p ares-core --lib print_paths::tests::solid_surface_roles`
  - `cargo test -p ares-core --lib pipeline::tests::top_bottom_solid_surface`
- Regression tests:
  - `cargo test -p ares-core --lib top_bottom_solid_surface`
  - `cargo test -p ares-core --lib fdm_normalization`
  - `cargo test -p ares-core --lib validation`
- Full verification:
  - `cargo test -p ares-core --lib`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and its implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with this spec, the reviewed plan, diff, and fresh verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and its implementation plan are the documentation artifacts for the slice. No CLI or WASM documentation changes are required because the public byte-in/options-to-byte-output API remains unchanged.
