# Task 22O.108: First-layer island footprint

## Upstream boundary

This slice follows `OrcaSlicer/src/libslic3r/Print.cpp::finalize_first_layer_convex_hull`
and its fallback to `Print::first_layer_islands`. Ares obtains the footprint
from the first compensated layer slices retained by the classic perimeter
prelude and translates it into print space.

## Included and deferred

Included: model-part first-layer island bounds after project compensation.
Deferred: the Orca union with generated skirt, brim, support, and wipe-tower
polygons, plus the final point-set convex hull. The implementation is entirely
3MF/options/geometry driven and does not inspect fixture identity or reference
G-code.
