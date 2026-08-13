# Task 22O.78 — rectilinear contour links

Port pinned `FillRectilinear.cpp:994-1214` over O77 segmentation. Add source
previous/next indices, horizontal/up/down link type, and valid/invalid/too-long
quality to each intersection.

Preserve candidate scan order, contour/kind matching, segment-distance direction,
same-line opposite-kind replacement, vertical inner-point invalidation,
both-same-side invalidation, don't-connect quality, maximum link-length
comparison, and symmetric invalid state. Do not invent hash lookup order or use
legacy scanline paths.

Focused tests cover a rectangle's adjacent horizontal links, a same-line case,
hole/inner invalidation, don't-connect, max length, symmetry, repeatability, and
input immutability. Separate modules, <400 LOC, no source-splitting macros.

Deferred: pinch handling, monotonic/traversal output, fillers, entities,
lifecycle, and G-code.
