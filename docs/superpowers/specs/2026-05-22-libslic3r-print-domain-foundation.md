# libslic3r Print Domain Foundation Spec

## Goal
Port the first Rust equivalents of OrcaSlicer's `Surface`, `ExtrusionRole`/extrusion-entity, and print/layer/region domain boundaries into `ares-core` while preserving the existing `ares_core::slice` API and CLI output behavior.

## Background
M20 decided that Ares keeps `ares-core` and `ares-cli` as the only active crates for now. M21 begins replacing Ares-owned path/pipeline vocabulary with `libslic3r` domain concepts in small behavior-preserving slices.

Relevant upstream references:
- `OrcaSlicer/src/libslic3r/Surface.hpp` defines `SurfaceType`, `Surface`, and helpers such as `is_top`, `is_bottom`, `is_bridge`, `is_external`, `is_internal`, and `is_solid`.
- `OrcaSlicer/src/libslic3r/ExtrusionEntity.hpp` defines `ExtrusionRole`, role helper predicates, and entity base concepts.
- `OrcaSlicer/src/libslic3r/ExtrusionEntityCollection.hpp` defines collections of extrusion entities.
- `OrcaSlicer/src/libslic3r/Layer.hpp` defines `LayerRegion` and `Layer` ownership of slices, perimeters, fills, and metadata.
- `OrcaSlicer/src/libslic3r/Print.hpp` defines `Print`, `PrintObject`, and `PrintRegion` boundaries.

Current Ares behavior already slices STL bytes through `ares-core`, generates layers/contours/perimeters/infills/skirt/brim/moves/extrusions/speeds, and emits G-code through `ares slice --options option.json -o output.gcode input.stl`. This milestone must not change generated G-code semantics; it adds `libslic3r`-aligned domain objects and exposes them through the in-memory pipeline for later ports.

## Requirements
- Add `ares-core` domain modules for:
  - `SurfaceType` and `Surface`, based on `Surface.hpp`, using existing Ares `Contour`/`Point2` data instead of full `ExPolygon` holes for this first slice.
  - `ExtrusionRole`, `ExtrusionPath`, and `ExtrusionEntityCollection`, based on `ExtrusionEntity.hpp` and `ExtrusionEntityCollection.hpp`.
  - `Print`, `PrintObject`, `PrintRegion`, `PrintLayer`, and `LayerRegion`, based on `Print.hpp` and `Layer.hpp`.
- `ExtrusionRole` must include the upstream role set represented in `ExtrusionEntity.hpp`: none, perimeter, external perimeter, overhang perimeter, internal infill, solid infill, top solid infill, bottom surface, ironing, bridge infill, internal bridge infill, gap fill, skirt, brim, support material, support material interface, support transition, wipe tower, custom, and mixed.
- `ExtrusionRole` must expose helper predicates equivalent to upstream intent for perimeter, internal perimeter, external perimeter, infill, top surface, solid infill, and bridge.
- Existing `PrintPathRole` must map to `ExtrusionRole` without changing existing G-code role strings in this milestone. Implement the mapping outside `print_paths.rs` if needed to keep `print_paths.rs` under 400 LOC.
- `SurfaceType` must expose helper predicates equivalent to upstream intent for top, bottom, bridge, internal bridge, external, internal, solid, and solid infill.
- Add a builder that creates a `Print` domain object from current `Layer`, `LayerContours`, and `LayerPrintPaths` artifacts:
  - one `PrintObject` for the current input model,
  - one default `PrintRegion`,
  - one `PrintLayer` per current layer,
  - one `LayerRegion` per layer,
  - contour slices represented as internal/perimeter surfaces for now,
  - current perimeter paths represented in the layer region `perimeters` collection,
  - current infill and bridge paths represented in the layer region `fills` collection,
  - current skirt and brim paths preserved in a separate `extras` extrusion entity collection so no current print-path role is dropped.
- `SlicingPipeline` must expose the built `Print` object through an accessor so future milestones can port away from custom pipeline artifacts incrementally. This adds an advanced in-memory Rust API while preserving the simple `ares_core::slice` byte API and CLI contract.
- The existing simple public async byte API remains `ares_core::slice(input, options) -> Result<Vec<u8>, SliceError>`.
- The existing CLI contract remains `ares slice --options option.json -o output.gcode input.stl`.
- No new crates or dependencies.
- No direct filesystem, UI, OpenGL, or terminal behavior in `ares-core`.
- Modified Rust files must stay under 400 LOC.
- Plan/spec review must receive independent APPROVE before implementation.
- Final implementation must receive independent spec-compliance APPROVE and code-quality APPROVE before commit.
- Verification must include targeted tests for new domain helpers/builders plus `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.

## Non-goals
- No exact polygon hole/`ExPolygon` port in this milestone.
- No Arachne, support, bridge detection, or G-code writer parity implementation.
- No generated G-code output changes; this milestone must include a no-change G-code regression check.
- No new options.
- No new workspace crates.
