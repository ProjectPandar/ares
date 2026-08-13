# Task 22O.66 — region bridge ensuring-area preparation

## Status

Implemented after approved behavioral RED; final independent implementation review pending.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3341-3343`, into private ordinary
module `prepare_infill/bridge_over_infill/region_bridge_ensuring_areas.rs`.
This operation prepares one region's near-perimeter ring and clips O65 ensuring
areas to it. It remains production-unwired and stops before line 3345. The
enclosing region loop at `3338` and empty `new_surfaces` allocation at `3339`
remain composer context.

Direct source closure: `Surface.hpp:119-157`, `ExPolygon.hpp:300-363`,
`LayerRegion.cpp:21-28`, `PrintRegion.cpp:8-53`, `Flow.cpp:129-145,200-205`,
`Flow.hpp:62-69`, `libslic3r.h:38-43,60-94`, and
`ClipperUtils.hpp:17-27,331-383,509-520` /
`ClipperUtils.cpp:264-410,642-679,738-739,788-810`.

## Exact contract

```rust
pub(in crate::project_slice) struct RegionBridgeEnsuringAreas {
    pub(in crate::project_slice) near_perimeters: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring: Vec<ExPolygon>,
}

pub(in crate::project_slice) fn prepare_region_bridge_ensuring_areas(
    fill_surfaces: &[RegionSurface],
    additional_ensuring_areas: &[Polygon],
    solid_infill_flow: Flow,
    scale: CoordinateScale,
) -> Result<RegionBridgeEnsuringAreas, ClipperError>;
```

## Behavior

1. Flatten all region surfaces, independent of type, in surface order and
   contour-before-holes order.
2. Run one safety union over the complete flat input and flatten its output
   contour-before-holes without sorting.
3. Resolve truncating integer scaled solid spacing through O53, cast to f32,
   shrink the whole flat safety-union output once by the negative delta using
   Miter/3, then run one default no-safety `original - shrunk` flat difference.
4. Run one default no-safety Polygon/Polygon-to-ExPolygon intersection with O65
   ensuring areas as subject and the resulting near-perimeter ring as clip.
5. Preserve exact operation/error order: safety union, shrink, difference,
   intersection. Return no partial result and mutate no borrowed value or
   allocation. Empty inputs take the same sequence.
6. Add no kind filter, validation, fallback, option lookup, per-surface union,
   batching, sorting, deduplication, surface rewrite, or lifecycle activation.

Trusted inputs are same-object region surfaces, O65 output, exact Task 22N
normal solid-infill Flow projected by the future composer, and object scale.
Spacing is finite positive and yields a strictly positive `i64`/`f32` scaled
delta; coordinates are normalized and Clipper-safe. No validation or Flow
resolution belongs here.

## Included and deferred

Included only: `PrintObject.cpp:3341-3343` and its direct dependency closure,
including new private geometry overload
`intersection_polygons_polygons_ex(&[Polygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>`
plus root re-exports and direct operand/topology/empty/range tests.

Deferred: region-loop/new-surface allocation `3338-3339`, layer/map/composer
projection; `stInternal` subtraction `3345-3350`; bridge conversion
`3352-3367`; solid recomposition `3368-3374`; surface replacement
`3385-3386`; second pass `3391+`; lifecycle, extrusion, motion,
G-code, CLI, and golden parity.

## Tests and acceptance

Behavioral RED must discriminate:

- representative available `RegionSurfaceKind` values without filtering,
  surface order, contour-before-hole flattening, and a
  hole-bearing multi-surface global safety union;
- exact Normal/LargeBed Flow truncation and f32 cast, negative Miter/3 shrink,
  complete-set flat original-minus-shrunk difference;
- ensuring-subject/near-perimeter-clip intersection and ExPolygon hole topology;
- empty inputs without an early return, operation call cardinality and order;
- natural and injected errors at every stage, first-error short-circuiting;
- output engine order, repeatability, and complete borrowed-input/allocation
  preservation.

Reversible compiling mutations must kill kind filtering, input/contour/hole
reordering, skipped/repeated/per-surface union, Flow/scale/cast/sign/join/miter,
per-polygon/repeated/skipped/reversed/safety difference,
skipped/reversed/safety intersection, early empty return, ignored errors, and
output sorting, then restore production byte-exact. Compiler failures are
invalid mutation evidence.

Final acceptance requires focused/dependency/workspace Nextest, strict Clippy,
rustfmt, wasm32, x86_64/aarch64 Windows and macOS, diff/LOC/static, clean pinned
Orca, no staged files, and independent six-axis review/repair/re-review until
unconditional approval. Every Rust source is at most 399 LOC, tests use ordinary
child modules, and include macros may not split source.
