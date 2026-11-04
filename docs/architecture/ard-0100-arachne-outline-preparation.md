# ARD-0100: Arachne outline preparation rewrite boundary

Status: analysis; not a completed parity milestone.

## Source ownership and destination

OrcaSlicer v2.4.2,8500fcdccaa10b5099ac20d252af3a7c560046f1 owns
`src/libslic3r/Arachne/WallToolPaths.cpp::WallToolPaths::generate` (484–513),
`simplify` (96–243), `fixSelfIntersections` (264–309),
`removeDegenerateVerts` (314–349), `removeColinearEdges` (408–481), and
`removeSmallAreas` (351–406). The Rust destination is
`crates/ares-core/src/arachne/wall_toolpaths/outline.rs` and focused child
modules if needed to keep every Rust file below400LOC.

Evidence-driven prerequisite: `PerimeterGenerator.cpp::process_arachne`
2140–2174 calls `ExPolygon.cpp::simplify_p`229–249 and
`ClipperUtils.cpp::offset_ex(Polygons)`409–410, not the ExPolygons overload.
Its Rust destination is `project_slice/perimeters/arachne/walls.rs`, using
existing geometry simplification and offset_paths_tree interfaces. Include
this bounded input-contract repair before outline cleaner changes if verified.
The same owner passes `coordf_t layer_height` through WallToolPaths constructor
80–95 and converts it directly to float for extrusion-width thresholds518–519.
Preserve this unscaled floating millimeter value through the Rust raw config
and top-surface regeneration; it is not an integer XY-coordinate quantity.
The shared config's concentric consumer (`FillConcentric.cpp:100`) converts
its existing scaled height to millimeters when constructing this config;
its earlier input truncation is deferred, not silently claimed repaired.
`WallToolPaths.cpp:490` / `Geometry.hpp:301–305` converts option degrees to
radians before BeadingStrategyFactory and SkeletalTrapezoidation. Preserve
this conversion at wall/top-surface config construction; shared raw config
already accepts radians from concentric and raw callers. No fallback unit
guessing or independent new transition algorithm.

## Decisions

- Preserve the upstream offset/simplify/near-intersection/degenerate/collinear/
  second-intersection/degenerate/small-area/union sequence. The current combined
  cleaner and single even-odd union are temporary incomplete rewrite scaffolds,
  not an independent Ares polygon-preparation contract.
- Diagnose the first divergent stage before changing raw Voronoi or beading.
  Source differences alone do not prove the missing native corner's cause.
- Keep integer truncation, floating precision, mutable vertex traversal and
  spatial-query ordering where they affect geometry. Do not substitute an
  all-pairs near-intersection scan for upstream SparseLineGrid.
- Keep core platform-neutral. Diagnostic debugger scripts remain external;
  remove any temporary core probes before commit.
- Acceptance stays at the existing public 3MF-to-complete-G-code
  seam. No private source-pinning regression, comparator widening, fixture
  modification or unsupported-success replacement.

## Included / deferred

Included: stage evidence and source-owned outline preparation corrections,
one verified vertical slice at a time, paired official and seeded outputs,
retained hole/overhang full-output regressions and finite previous GREEN controls.

Deferred until evidence points there: `SkeletalTrapezoidation` raw generation,
stitching, Adaptive PA, Arachne+raft coverage, adjacent G-code consumers and
unrelated Option domains. These are not waived or counted as completed.
