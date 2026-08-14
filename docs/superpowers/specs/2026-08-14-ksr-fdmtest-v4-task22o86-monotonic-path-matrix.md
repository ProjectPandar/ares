# Task 22O.86 — monotonic path matrix

Port pinned `FillRectilinear.cpp:1590-1709`. Add dense orientation-addressed
lazy edge costs with f32 endpoint distance, coordinate unscaling, source epsilon
visibility, cached length/visibility, and independently reset pheromone.

Focused tests cover all four orientation addresses, endpoint selection, lazy
cache identity, initial pheromone reset, exact f32 bits, both coordinate scales,
and immutable regions/slice. Separate modules, <400 LOC, no source-splitting
macros.

Deferred: ants/RNG/path selection, polylines/entities, lifecycle, G-code.
