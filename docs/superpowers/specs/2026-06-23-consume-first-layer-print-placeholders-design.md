# Consume First Layer Print Placeholder Design

## Goal

Consume OrcaSlicer's first-layer print bounds placeholders into concrete machine start G-code behavior by rendering `[first_layer_print_min]`, `[first_layer_print_max]`, and `[first_layer_print_size]`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2881-2904` computes the first-layer print convex hull path, builds a `BoundingBoxf bbox`, and registers:
  - `first_layer_print_min` as `{ bbox.min.x(), bbox.min.y() }`
  - `first_layer_print_max` as `{ bbox.max.x(), bbox.max.y() }`
  - `first_layer_print_size` as `{ bbox.size().x(), bbox.size().y() }`
- `OrcaSlicer/src/libslic3r/GCode.cpp:2887-2895` states the non-calibration upstream source is the convex hull of first-layer extrusions, including object extrusions, support extrusions, skirt, brim, and wipe tower, while excluding custom G-code purge lines and MMU/MMU2 starting areas.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11013-11025` defines the dimensions placeholders `first_layer_print_convex_hull`, `first_layer_print_min`, `first_layer_print_max`, and `first_layer_print_size`.

## Ares Destination Boundary

- Add a focused `crates/ares-core/src/gcode_first_layer_print_placeholders.rs` helper module.
- The helper computes an axis-aligned bounding box from Ares' existing first layer `LayerPrintPaths`, reusing the same rendered-print-path boundary already consumed by adaptive bed mesh.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` receives optional first-layer print placeholders and renders only `[first_layer_print_min]`, `[first_layer_print_max]`, and `[first_layer_print_size]` in `machine_start_gcode`.
- The existing start-G-code call path may be adjusted only as needed so machine start rendering receives both adaptive-bed-mesh and first-layer-print placeholder data derived from the same `LayerPrintPaths`.

## Included Behavior

- A machine start template containing `[first_layer_print_min]`, `[first_layer_print_max]`, or `[first_layer_print_size]` renders before the first `;LAYER_CHANGE`.
- The rendered values come from the min/max/size of all points in the first layer's existing `LayerPrintPaths`.
- The default square-pyramid fixture renders `-2.5,-2.5` for min, `2.5,2.5` for max, and `5,5` for size because the first layer contains the generated skirt path around the object.
- Disabling the skirt renders bounds from the first-layer external perimeter path only: `-0.5,-0.5`, `0.5,0.5`, and `1,1`.
- Decimal formatting uses existing compact G-code decimal formatting.
- If the first layer has no print path points, the three placeholders render as empty strings instead of inventing a bed-size fallback.
- The placeholders remain literal outside machine start G-code, including `layer_change_gcode`.

## Deferred Behavior

- Do not port Orca's `ConfigOptionPoints` first-layer convex hull placeholder in this slice.
- Do not port Orca's calibration PA line/pattern branch that offsets the bed bounding box by `-25.0`.
- Do not add head-wrap detection, `in_head_wrap_detect_zone`, object-projection intersection, `first_layer_center_no_wipe_tower`, `max_print_z`, wipe tower geometry, support generation, custom purge-line bounds, MMU/MMU2 start-area bounds, or convex-hull geometry.
- Do not change adaptive bed mesh bounds, print bed placeholders, model placement, travel bounds, path generation, clipping, or runtime G-code outside the three machine-start placeholder replacements.
- Do not add option metadata, candidate crates, dependencies, filesystem behavior, terminal behavior, UI, OpenGL, or native-only behavior.

## Acceptance Criteria

1. Focused RED tests demonstrate that `[first_layer_print_min]`, `[first_layer_print_max]`, and `[first_layer_print_size]` are not rendered before implementation.
2. Focused GREEN tests prove the three placeholders render from the default first-layer print paths, including skirt bounds.
3. Tests prove disabling skirt narrows the bounds to the first-layer external perimeter path.
4. Tests prove all three placeholders compose with existing machine-start placeholders and remain literal in `layer_change_gcode`.
5. Tests cover empty first-layer print path fallback through the helper without requiring a fabricated G-code model state.
6. Implementation touches only the focused core G-code/options test surface needed for these placeholders and keeps touched Rust files at or below 400 LOC.
7. Verification uses `cargo nextest run`, not `cargo test`, with focused tests, adjacent related tests, full workspace tests, clippy, wasm check, format check, diff checks, and LOC guard before commit.

## Test Strategy

- Add `crates/ares-core/src/tests/first_layer_print_placeholders_gcode.rs`.
- Register it from `crates/ares-core/src/tests/mod.rs` near other machine-start placeholder G-code modules.
- Use `slice(square_pyramid_ascii_stl(), options)` for rendered G-code behavior.
- Add a small unit test in `gcode_first_layer_print_placeholders.rs` for empty first-layer print paths.
- Run focused command `cargo nextest run -p ares-core first_layer_print_placeholders`.
- Run adjacent command `cargo nextest run -p ares-core adaptive_bed_mesh_gcode print_bed_placeholders`.

## Verification Commands

- `cargo fmt --check`
- `cargo nextest run -p ares-core first_layer_print_placeholders`
- `cargo nextest run -p ares-core adaptive_bed_mesh_gcode print_bed_placeholders`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- `for file in crates/ares-core/src/gcode_machine_start_placeholders.rs crates/ares-core/src/gcode_first_layer_print_placeholders.rs crates/ares-core/src/gcode_adaptive_bed_mesh.rs crates/ares-core/src/lib.rs crates/ares-core/src/tests/mod.rs crates/ares-core/src/tests/first_layer_print_placeholders_gcode.rs; do test "$(wc -l < "$file")" -le 400 || exit 1; done`

## Docs Impact

No user-facing documentation update is required because the repository does not currently have a dedicated placeholder reference document. This source-cited SDD spec, implementation plan, and focused regression tests document the behavior.

## Workflow Completion

After implementation acceptance, the active user objective still requires a Lore-protocol commit and push to `origin/codex/consume-slicing-options`. That repository side effect is part of the `$sdd-workflow` completion sequence, not a G-code behavior acceptance criterion.

## Safety

The change is platform-neutral Rust in `ares-core`, does not perform file I/O, terminal I/O, UI, OpenGL, networking, or native-only operations, and adds no dependencies.
