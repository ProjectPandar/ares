# Task 22O.2 implementation plan

## Boundary

Implement the bounded source rewrite specified in
`2026-08-01-ksr-fdmtest-v4-task22o2-dynamic-top-one-wall.md` against fixed
OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Task 22O.1 is the owned predecessor, not a fallback.

## Steps

1. Add RED vectors for `ClipperUtils.cpp:55-166` bbox prefilter and
   `ExPolygon.cpp:50-56` area-with-holes behavior.
2. Add the smallest geometry seams: bounds inflation/access, ordered ExPolygon
   flattening, production polygon offset, and polygon-clip difference with and
   without the existing 10-unit safety offset.
3. Define an owned, aligned post-split state that retains the full Task 22O.1
   predecessor and both first-offset width classes.
4. Preflight every populated record before geometry, resolving typed effective
   3MF values, percent bases, selected nozzle, automatic infill width, and gap
   enablement.
5. Port the non-thin-wall `i == 0` offset from
   `PerimeterGenerator.cpp:1235-1306`, including exact narrow/short selection
   and collapse.
6. Port `split_top_surfaces` from `PerimeterGenerator.cpp:574-660` in source
   operation order, including bbox-filtered upper/lower inputs, bridge checker,
   top growth, gap handling, and final infill clip.
7. Apply only the caller gates at `PerimeterGenerator.cpp:1377-1385`, after
   normal first-offset geometry becomes `last`.
8. Add separate caller, geometry, typed mutation, KSR repeatability, alignment,
   and public lifecycle tests without binary/source-text oracles.
9. Chain `prepare_post_classic_prelude -> finish_classic_top_split`; continue
   returning `ProjectSlicingIncomplete`.
10. Run focused Task 22O.2, Task 22O.1 and Task 22N nextest; workspace nextest,
    fmt, warning-denying Clippy, native all-feature checks, and core WASM check.
    Audit file lengths and forbidden patterns.

## Explicit deferrals

Do not implement `i >= 1`, loop extrusion entities, hierarchy/traversal,
thin-wall medial axes, multi-region support, later bridge kinds, gap masks,
overhang splitting, fill remainder, seams, infill, motion, writer,
post-processing, Arachne, or an old pipeline fallback. This increment does not
claim complete Task 22O or G-code parity.
