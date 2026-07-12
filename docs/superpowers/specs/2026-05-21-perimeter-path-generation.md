# Perimeter Path Generation Spec

## Goal
Advance Ares from diagnostic contours to first perimeter path artifacts that later G-code and infill milestones can consume, while keeping geometry scope limited to current simple closed contours.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/PrintObject.cpp::PrintObject::make_perimeters` runs after object slicing and calls `Layer::make_perimeters` for each layer.
- `OrcaSlicer/src/libslic3r/Layer.cpp::Layer::make_perimeters` groups compatible region slices and delegates to `LayerRegion::make_perimeters`, then stores perimeter and fill-surface products on layer regions.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.hpp` defines `PerimeterGenerator`, whose outputs include perimeter loops, gap fills, and fill surfaces.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp::PerimeterGenerator::process_classic` creates external and internal perimeter loops by offsetting surfaces inward by line-width/spacing-derived distances, nests loops, and leaves fill surfaces for infill.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` defines perimeter-related options such as `wall_loops`, `wall_filament`, `inner_wall_line_width`, and `outer_wall_line_width`. Ares does not yet have polygon offsetting or a full typed option registry, so M10 ports only the first path artifact boundary from existing contours.

## Scope
Milestone 10 adds a `perimeters` module that converts each current closed contour into one external perimeter path with the same deterministic point order. The pipeline appends this stage and `slice` emits perimeter diagnostics and path metadata. Internal wall offsets, gap fills, fill surfaces, extrusion E values, and perimeter option families remain later milestones.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub fn generate_perimeters(layers: &[LayerContours]) -> Result<Vec<LayerPerimeters>, SliceError>;
   pub struct LayerPerimeters { ... }
   pub struct PerimeterPath { ... }
   pub enum PerimeterRole { External }
   ```
2. `generate_perimeters` returns one `LayerPerimeters` for every input `LayerContours`, preserving layer id and print Z.
3. Each non-empty `Contour` becomes exactly one `PerimeterPath` with role `PerimeterRole::External`.
4. `PerimeterPath::points()` returns the contour vertices in existing deterministic contour order. The closing point is not duplicated.
5. Empty contour layers produce empty perimeter path lists.
6. Contours with fewer than three points return `SliceError::InvalidInput`.
7. Output ordering remains deterministic by input layer order and contour order.
8. `SlicingPipeline` owns and exposes `layer_perimeters() -> &[LayerPerimeters]`.
9. `PipelineStage` adds `Perimeters` with string value `perimeters`; completed stages become `model,layers,segments,contours,perimeters`.
10. `PipelineDiagnostics` adds `total_perimeter_count() -> usize`, derived from the perimeter artifacts.
11. `slice` output adds:
    - header `; total_perimeter_count = N`
    - per-layer `; perimeter_count = N`
    - one `;PERIMETER:external:x1,y1 -> x2,y2 -> ...` line per path
12. Existing segment and contour metadata remains unchanged.
13. CLI output includes the new perimeter metadata through the existing `ares slice` command.
14. No new crates or dependencies are introduced.
15. `ares-core` remains platform-neutral and performs no filesystem I/O.
16. Modified Rust files remain under 400 LOC. Split modules if needed.

## Non-goals
- No polygon offsetting, internal wall loops, gap fill, fill surfaces, wall-loop option typing, extrusion E values, speed/flow roles, seam placement, overhang detection, Arachne, spiral vase, or Orca perimeter parity.
- No 3MF geometry extraction.
- No new workspace crates.

## Acceptance criteria
- Core tests cover generating one external perimeter path from a square contour, preserving empty layers, deterministic multi-contour order, and rejecting malformed contours.
- Pipeline tests cover the added perimeter stage, `layer_perimeters()` accessor, and `total_perimeter_count` diagnostics.
- Core `slice` tests cover the new header/per-layer perimeter metadata while preserving existing segment/contour metadata.
- CLI tests cover perimeter metadata from `ares slice --options option.json -o output.gcode input.stl`.
- Docs include `docs/milestones/m10-perimeter-path-generation.md`, this spec, an implementation plan, roadmap exit criteria update, and an ARD for external perimeter paths before polygon offsets.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation reviews return APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
