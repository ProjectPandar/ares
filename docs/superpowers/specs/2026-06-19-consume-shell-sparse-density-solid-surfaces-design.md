# Consume Shell Sparse Density as Solid Surfaces Design

## Goal

Consume the already parsed `bottom_shell_layers`, `top_shell_layers`, bottom/top surface pattern options, and solid infill numeric options in actual generated infill for sparse-density prints. Ares should no longer emit sparse infill on configured top/bottom shell layers when `sparse_infill_density` is nonzero; those shell layers should generate solid infill paths that downstream print-path, extrusion, speed, and G-code stages classify as `bottom_surface` or `top_solid_infill`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/Surface.hpp:106-113`: `Surface::is_bottom`, `is_external`, and `is_solid` make `stBottom` / `stBottomBridge` / `stTop` solid surfaces independent of the sparse-infill density.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:548-589`: `prepare_infill()` calls `detect_surfaces_type()`, `prepare_fill_surfaces()`, and `discover_horizontal_shells()` to classify and extend top/bottom shell surfaces.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1474-1482`: `detect_surfaces_type()` classifies `stTop`, `stBottomBridge`, `stBottom`, and `stInternal` from layer support and coverage.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:887-924`: solid external surfaces select top/bottom patterns and emit top/bottom extrusion roles; sparse internal surfaces keep sparse density and sparse role.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:934-941`: non-sparse roles use solid infill direction and solid rotate templates.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:967-983`: solid and bridge infill use solid/bridge spacing and do not apply sparse open-anchor extension.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs`: add an internal layer role helper for the current simplified full-contour shell classification. This helper returns sparse, bottom surface, internal solid, or top surface for a layer.
- `crates/ares-core/src/infills.rs` and `crates/ares-core/src/infills/rotation.rs`: generate dense shell paths for nonzero sparse-density top/bottom shell layers, using solid direction/template and bottom/top surface patterns while keeping sparse interior behavior unchanged.
- Existing downstream stages stay the consumers: `generate_print_paths()` already maps solid infill on shell layers to `PrintPathRole::BottomSurface` / `TopSolidInfill`, and extrusion/speed/G-code already consume those roles.

## Included Behavior

1. When `sparse_infill_density > 0` and a layer index is inside `bottom_shell_layers`, `generate_infills()` emits `InfillRole::Solid` paths for that layer.
2. When `sparse_infill_density > 0` and a layer index is inside `top_shell_layers`, `generate_infills()` emits `InfillRole::Solid` paths for that layer.
3. Interior layers outside configured shell ranges continue to use `InfillRole::Sparse`, sparse density spacing, sparse pattern, sparse direction, sparse rotate template, and sparse anchor length.
4. Dense shell layers use 100% density spacing based on `line_width`, no sparse anchor extension, solid direction / solid rotate template, and the shell-specific pattern (`bottom_surface_pattern` or `top_surface_pattern`).
5. For overlapping shell classification on short prints, bottom shell wins before top shell, matching the existing `generate_print_paths()` bottom-before-top role precedence.
6. Pipeline and G-code output for a nonzero sparse-density multi-layer rectangle must include bottom/top solid-surface roles and comments without requiring `sparse_infill_density: 100`.

## Deferred Behavior

- Full `SurfaceCollection`, `detect_surfaces_type()`, `prepare_fill_surfaces()`, `discover_horizontal_shells()`, and per-polygon partial shell classification remain deferred.
- `top_surface_density` and `bottom_surface_density` runtime parsing and density-specific spacing remain deferred; shell density is fixed at 100% for this slice.
- Bridge-specific surface types (`stBottomBridge`, `stInternalBridge`, `stSecondInternalBridge`) and bridge angle/density/filter/extra-layer behavior remain deferred.
- Zero sparse-density shell infill is not changed in this slice. Ares currently uses `sparse_infill_density == 0` as a whole-stage no-infill guard, and changing that requires a separate source-cited decision about shell generation with no sparse interior.
- Spiral vase behavior remains unchanged.
- No new crates, dependencies, file I/O, UI behavior, OpenGL behavior, or Ares-owned slicing pipeline concepts are introduced.

## Acceptance Criteria

- A focused `ares-core` infill test proves a sparse-density print generates solid bottom and top shell infill roles while preserving sparse interior roles.
- A focused `ares-core` infill test proves bottom/top shell layers use solid spacing and no sparse anchor extension.
- A focused `ares-core` pipeline/G-code test proves `sparse_infill_density: 50` with shell layers emits `;PRINT_PATH:bottom_surface:`, `;PRINT_PATH:sparse_infill:`, and `;PRINT_PATH:top_solid_infill:` in one generated pipeline.
- Existing density-100 solid-surface tests keep passing.
- `cargo fmt --check`, targeted tests, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass.

## Safety

This is an in-memory `ares-core` behavior change only. It does not touch adapters, filesystem behavior, WASM bindings, model loading, profile loading, or external services. The change is reversible by restoring the prior `generate_infills()` sparse-density role selection and tests.
