# Consume 100 Percent Sparse Infill As Internal Solid Design

## Goal

Consume OrcaSlicer's concrete `sparse_infill_density == 100%` behavior in Ares. When the current Ares internal infill region is configured at 100% density, it must be generated and emitted as internal solid infill, using the existing internal-solid pattern and solid-infill rotation options, instead of continuing to behave as ordinary sparse infill.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1092` declares `internal_solid_infill_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1095-1101` declares `infill_direction`, `solid_infill_direction`, `solid_infill_rotate_template`, `symmetric_infill_y_axis`, `infill_shift_step`, `sparse_infill_rotate_template`, and `sparse_infill_density`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2018-2025` registers `internal_solid_infill_pattern` and defaults it to `ipMonotonic`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2871-2879` registers `solid_infill_direction` and defaults it to 45 degrees.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2881-2885` documents that 100% sparse density turns all sparse infill into solid infill and uses the internal solid infill pattern.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3887-3892` registers `solid_infill_rotate_template` as a comma-separated per-layer solid-infill rotation template.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355-8368` and `8428-8441` normalize `spiral_mode` by forcing `sparse_infill_density` to 0.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:915-919` converts `stInternal` surfaces to `stInternalSolid` when `sparse_infill_density` is 100% and spiral mode is not active.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:887-899` chooses `internal_solid_infill_pattern` and density 100 for `surface.is_solid_infill()`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:910-923` maps solid surfaces to `erSolidInfill`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:934-941` uses `infill_direction` / `sparse_infill_rotate_template` for `erInternalInfill`, but `solid_infill_direction` / `solid_infill_rotate_template` for non-internal-infill roles.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6425` and `6465` show the later internal-solid-specific flow and speed hooks. This slice does not consume those hooks yet; it keeps Ares' existing sparse numeric plumbing while changing the role identity.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:50-55` and `72-73` show the supported rectilinear, aligned rectilinear, monotonic, line, grid, monotonic-line, and zigzag fill engines relevant to this Ares scaffold.

## Ares Boundary

- Extend `InfillOptions` in `crates/ares-core/src/options/infill.rs` so it can describe the effective infill role and effective pattern/rotation for the current Ares internal fill region. When `spiral_mode` is true, direct `SliceOptions::infill_options()` must expose effective sparse density 0. Otherwise the effective role is solid only when `sparse_infill_density == 100`.
- Parse `internal_solid_infill_pattern`, `solid_infill_direction`, and `solid_infill_rotate_template` for runtime use. This must consume already-registered Orca option keys; it must not add new option metadata-only milestones.
- In `crates/ares-core/src/infills.rs`, generate paths with `InfillRole::Solid` when the effective role is solid. Keep `sparse_infill_density == 100` plus `spiral_mode == true` sparse-disabled through the existing `SliceOptions::normalize_fdm()` path and direct `InfillOptions` gating.
- In `crates/ares-core/src/infills/rotation.rs`, use `solid_infill_direction` and `solid_infill_rotate_template` for the 100% internal-solid path, while preserving existing sparse direction/template behavior for densities below 100%.
- In `crates/ares-core/src/print_paths.rs`, map `InfillRole::Solid` to a new `PrintPathRole::SolidInfill`.
- In `crates/ares-core/src/extrusion_entity.rs` and `crates/ares-core/src/print.rs`, map `PrintPathRole::SolidInfill` to `ExtrusionRole::SolidInfill` and keep it inside the fills collection.
- In `crates/ares-core/src/moves.rs`, propagate `PrintPathRole::SolidInfill` as an open line path, like sparse infill.
- In `crates/ares-core/src/extrusions.rs`, temporarily use `sparse_infill_line_width`, `sparse_infill_flow_ratio`, and `initial_layer_line_width` / `first_layer_flow_ratio` for `PrintPathRole::SolidInfill`. This is a compatibility shell until the later source-cited `internal_solid_infill_line_width` and `internal_solid_infill_flow_ratio` slice.
- In `crates/ares-core/src/speeds/config.rs`, `crates/ares-core/src/speeds/slow_down_layers.rs`, and `crates/ares-core/src/speeds/kinematics.rs`, temporarily use sparse infill speed, first-layer infill speed, sparse infill acceleration, and infill jerk for `PrintPathRole::SolidInfill`. This is a compatibility shell until the later source-cited `internal_solid_infill_speed` and `internal_solid_infill_acceleration` slice.
- In `crates/ares-core/src/options/part_cooling_fan.rs`, keep `PrintPathRole::SolidInfill` out of bridge fan overrides.
- In `crates/ares-core/src/gcode*.rs` and custom role-change placeholder paths, rely on `PrintPathRole::SolidInfill::as_str()` returning `solid_infill` so speed/extrusion comments, role-change placeholders, and final G-code identify the path as solid infill.
- Keep the path geometry scaffold line-based and deterministic. `monotonic` / `monotonicline` are accepted as internal-solid pattern values and use the existing rectilinear line scaffold in this slice; full monotonic ordering is deferred.

## Included Behavior

- `sparse_infill_density < 100` preserves current sparse infill behavior, role names, sparse pattern parsing, sparse direction, sparse rotate template, sparse speed, and sparse flow ratio.
- `sparse_infill_density == 100` emits generated infill paths as `InfillRole::Solid`, `PrintPathRole::SolidInfill`, and `ExtrusionRole::SolidInfill`.
- `sparse_infill_density == 100` with `spiral_mode == true` does not emit any infill. Through `run_slicing_pipeline`, `SliceOptions::normalize_fdm()` forces the density to 0 and no infill is generated; through direct `SliceOptions::infill_options()` use, `InfillOptions::sparse_density_percent()` must also be 0 so `generate_infills()` returns no infill paths.
- The G-code comments for 100% density contain `;INFILL:solid:`, `;PRINT_PATH:solid_infill:`, `;SPEED:...:solid_infill:`, and `;EXTRUSION:print:solid_infill:`.
- The 100% internal-solid path uses `internal_solid_infill_pattern`; at minimum, `rectilinear`, `alignedrectilinear`, `line`, `grid`, `zigzag`, `monotonic`, and `monotonicline` are accepted for this effective internal-solid pattern.
- Missing `internal_solid_infill_pattern` defaults to Orca's `monotonic` and uses the current rectilinear-like scaffold.
- The 100% internal-solid path uses `solid_infill_direction` and `solid_infill_rotate_template`; sparse `infill_direction` and `sparse_infill_rotate_template` no longer control this 100% internal-solid path.
- `PrintPathRole::SolidInfill` uses the same numeric width, flow, speed, first-layer speed, acceleration, jerk, volumetric capping, slowdown, and fan behavior as sparse infill in this slice, but comments and print-domain roles must use `solid_infill` / `SolidInfill`.
- Invalid `internal_solid_infill_pattern`, `solid_infill_direction`, or `solid_infill_rotate_template` values fail during `SliceOptions::infill_options()` parsing with `SliceError::InvalidInput` naming the invalid key.
- The implementation remains platform-neutral inside `ares-core` and adds no dependencies, filesystem, UI, terminal, OpenGL, or viewer behavior.

## Deferred Behavior

- Full `stTop`, `stBottom`, `stBottomBridge`, `stInternalBridge`, and multi-surface `SurfaceCollection` classification. Ares currently feeds a single internal contour region into sparse infill generation.
- Full Orca monotonic and monotonic-line traversal semantics. In this slice those patterns are accepted for internal-solid compatibility and mapped to the current deterministic rectilinear line scaffold.
- `internal_solid_infill_line_width`, `internal_solid_infill_speed`, and `internal_solid_infill_flow_ratio` runtime consumption. This slice changes role and pattern/rotation first; per-role solid width/speed/flow can follow as a separate source-cited slice.
- `top_surface_pattern`, `bottom_surface_pattern`, top/bottom densities, top/bottom flow ratios, top-surface speed, bottom-surface role behavior, ironing, and narrow internal solid detection.
- `align_infill_direction_to_model`, because Ares' infill generator does not yet receive model transform data.
- `infill_shift_step`, because upstream applies it only to `ipCrossZag` / `ipLockedZag`, which Ares currently rejects.
- Full `FillMonotonic`, `FillMonotonicLines`, `FillZigZag`, connected-polyline stitching, bridge flow, travel optimization, multi-region batching, and object/region extruder routing parity.
- Any new crate, dependency, UI behavior, terminal behavior, filesystem behavior, OpenGL/viewer behavior, or independent Ares-owned slicing pipeline design.

## Acceptance Criteria

1. Options tests prove `internal_solid_infill_pattern` defaults to `monotonic`, accepts supported internal-solid patterns, rejects unknown values with an error mentioning `internal_solid_infill_pattern`, parses `solid_infill_direction`, and parses/rejects `solid_infill_rotate_template` with key-specific errors.
2. Options or pipeline tests prove `sparse_infill_density == 100` plus `spiral_mode == true` does not produce solid infill; the full pipeline path must show normalized density 0 and no infill paths.
3. Infill unit tests prove `sparse_infill_density == 100` changes generated paths from sparse to solid role while preserving non-empty line generation.
4. Options and infill unit tests prove `sparse_infill_density == 100` plus direct `spiral_mode == true` options exposes `InfillOptions::sparse_density_percent() == 0` and `generate_infills()` produces no infill paths.
5. Infill unit tests prove the 100% internal-solid path uses `internal_solid_infill_pattern = "grid"` to add the perpendicular pass that `sparse_infill_pattern = "rectilinear"` alone would not add.
6. Infill unit tests prove the 100% internal-solid path uses `solid_infill_rotate_template` rather than `sparse_infill_rotate_template`.
7. Print-path and print-domain tests prove `InfillRole::Solid` maps to `PrintPathRole::SolidInfill`, then to `ExtrusionRole::SolidInfill`, and remains in the fills collection.
8. Extrusion and speed tests prove `PrintPathRole::SolidInfill` temporarily reuses sparse infill line width, sparse infill flow ratio, initial-layer infill flow treatment, sparse infill speed, first-layer infill speed, sparse acceleration, infill jerk, and sparse slowdown behavior while emitting the `solid_infill` role string.
9. Pipeline/G-code tests prove 100% density reaches `LayerInfills`, `LayerPrintPaths`, `Print`, toolpath/extrusion/speed moves, and G-code comments as `solid_infill`.
10. Existing sparse density below 100%, sparse pattern, sparse rotate template, symmetric zigzag, infill combination, speed, extrusion, and role-change tests continue to pass.
11. All touched Rust files under `crates/` remain at or below 400 LOC.

## Verification

- Targeted RED/GREEN tests for the new parser and 100% internal-solid behavior.
- `cargo test -p ares-core --lib internal_solid`
- `cargo test -p ares-core --lib sparse_infill_pattern`
- `cargo test -p ares-core --lib infills`
- `cargo test -p ares-core --lib spiral_mode`
- `cargo test -p ares-core --lib solid_infill`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- Rust LOC gate: `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and the implementation plan are the documentation artifacts for the slice. No CLI or WASM docs are needed because this change consumes existing Orca option keys inside `ares-core` and does not change public adapter APIs.
