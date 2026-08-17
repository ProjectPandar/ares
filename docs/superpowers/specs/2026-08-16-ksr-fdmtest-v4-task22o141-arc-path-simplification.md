# Spec: Task 141 arc-fitting extrusion-path simplification

## Observable contract

When `enable_arc_fitting` is true, `slice_project` simplifies each generated extrusion path before dynamic overhang-speed processing. The KSR V4 inner-wall path therefore preserves the OrcaSlicer vertex sequence around `X132.523 Y100.347` through `X130.67 Y101.119` and omits the redundant intermediate vertices previously emitted by Ares.

The behavior is driven only by the loaded `enable_arc_fitting` and `resolution` options plus generated path geometry. It must not inspect fixture identity or reference G-code.

## Upstream boundary

This slice rewrites OrcaSlicer 2.4.2 `src/libslic3r/ArcFitter.cpp:9-150` (`do_arc_fitting` and `do_arc_fitting_and_simplify`), `src/libslic3r/LayerRegion.cpp:1055-1125` (wall path simplification), and `src/libslic3r/PrintObject.cpp:916-933` (simplification before downstream G-code processing).

Included behavior: fit arc and linear ranges first, apply Douglas-Peucker independently to every range, retain fitted arc data, and feed the simplified point sequence into dynamic overhang estimation. Deferred behavior: the remaining first-segment extrusion delta mismatch, full upstream arc-fit numerical parity outside this observed path, and non-arc-fitting path simplification.

## Acceptance

The focused `slice_project` test observes the exact source wall vertices after the final inner-wall feature marker before the first overhang feature. All touched Rust source files remain below 400 lines.
