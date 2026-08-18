# Spec: KSR FDM Test V4 task213 sparse infill simplification resolution

## Observable contract

Arc fitting and Douglas-Peucker simplification use 0.04 mm tolerance for `InternalInfill` paths, independent of the configured general print resolution. Perimeters, solid/bottom/top fill, bridges, and gap paths continue to use the configured resolution. This role choice occurs before fitting and point replacement.

A focused role-selection test pins sparse versus configured tolerance. Fixture sparse-infill moves decrease from task211's 43,235 to 36,941 toward the reference 33,571; feature blocks remain 846 versus 1,123. Files remain below 400 LOC; path/arc/sparse tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `libslic3r.h:78-79` and `LayerRegion.cpp:1070-1124`, the `SCALED_SPARSE_INFILL_RESOLUTION` branch used by `simplify_by_fitting_arc`, into Ares path simplification. Other role geometry, arc candidate selection, exact E, timing, and remaining differences are deferred.
