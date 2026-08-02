# Task 22O.2: Classic dynamic top-one-wall split

## Goal

Port the next bounded OrcaSlicer v2.4.2 Classic perimeter slice into
`ares-core`: the KSR-reached first external onion offset and
`PerimeterGenerator::split_top_surfaces`. The public lifecycle executes this
state transition and continues to return `ProjectSlicingIncomplete`.

## Fixed upstream boundary

Baseline: OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Included source:

- `src/libslic3r/PerimeterGenerator.cpp:574-660`, complete
  `split_top_surfaces`.
- `PerimeterGenerator.cpp:1235-1306,1343-1385`, only the non-thin-wall
  `i == 0` external offset, normal/smaller-width selection, collapse, `last`
  transition, and exact dynamic-top caller predicates.
- `PerimeterGenerator.cpp:22-27`, the 10 mm narrow-loop threshold.
- `src/libslic3r/ClipperUtils.cpp:55-166,650-710`, bbox polygon prefilter and
  polygon-clip safety-difference semantics.
- `src/libslic3r/ExPolygon.cpp:50-56`, area with holes.
- `src/libslic3r/Flow.cpp:21-35`, zero-width infill fallback.
- `src/libslic3r/Surface.hpp:105-113` and `libslic3r.h:86-96`, bridge predicate,
  bridge margin, scaling, and `SCALED_EPSILON`.

The first offset is included only because upstream calls the split after
`last = offsets`; invoking it on Task 22O.1 surface input would move the source
seam.

## Rust destination and state

`project_slice::perimeters::classic::top_split` owns the transactional stage.
Its output owns the unchanged `PreparedPostClassicPrelude` predecessor and
aligned optional records. Each surface retains initial/effective loop counts,
normal and smaller first-offset geometry, post-caller remaining geometry,
`top_fills`, `fill_clip`, caller outcome, and selected upper source.

All configuration is resolved from effective typed 3MF options before geometry:
`wall_loops`, `only_one_wall_top`, `interface_shells`,
`min_width_top_surface`, `sparse_infill_line_width`, outer nozzle selection,
and gap enablement. Percent bases and zero-width automatic infill follow the
cited source.

## Required behavior

- Preserve source f32/f64/fixed-coordinate casts and operation order.
- Use the source three-neighbor bbox vertex filter, not rectangle clipping.
- Choose smaller external width only for a collapsed narrowness opening whose
  area is below the fixed width-times-10-mm threshold.
- Call the split only for `i == 0`, `i != loop_number`, enabled top-one-wall,
  nonbridge surfaces with an upper layer and no earlier collapse.
- Preserve record slots and source surface order.
- Map Clipper range failures to a deterministic `InvalidInput` without partial
  output.

## Deferred behavior

Deferred: `i >= 1` onion shells and gap-only iteration; loop entities,
hierarchy and traversal; thin-wall medial axes; active multi-region
`interface_shells`; bridge surface classification beyond the currently
reachable internal surface; gap-mask collection; sparse-density termination;
overhang splitting; fill remainder; seams; infill; motion; writer and
post-processing; Arachne; old rectangular pipeline fallback; complete Task 22O
and G-code parity.

## Exit criteria

Focused source-derived geometry, caller, typed-option, transaction, KSR, and
public-lifecycle tests pass; Task 22O.1 and Task 22N regressions pass; workspace
nextest, fmt, Clippy, native checks, and core WASM check pass; every Rust file is
under 400 physical lines and forbidden implementation patterns are absent.
