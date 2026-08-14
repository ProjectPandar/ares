# Task 22O.97 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the pure external-perimeter seam candidate topology from pinned OrcaSlicer
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/GCode/SeamPlacer.hpp:42-108`;
- `src/libslic3r/GCode/SeamPlacer.cpp:229-273,406-592,1014-1038`;
- `src/libslic3r/ExtrusionEntity.hpp:507-512`.

Operate directly on retained source-native perimeter entity collections. A loop
is an external-perimeter candidate polygon when any constituent path has the
external-perimeter role, including mixed external/overhang loops. Preserve
`collect_points` path order and repeated closing/join points, normalize the
candidate polygon counter-clockwise, retain the original winding as the angle
sign, and calculate vertex angles with the source 0.4 mm nozzle-diameter arm for
KSR. Candidate flow width comes from each polygon's corresponding region external-
perimeter flow, including source fallback polygons.

This milestone is pure and inactive in `slice_project`. Occlusion/visibility,
painted enforcer/blocker topology, overhang and embedding penalties, candidate
selection, cross-layer alignment, placement, loop clipping, the runtime cursor,
and O96 activation remain deferred. The legacy `perimeters/seams.rs` path is not
used and no fixture branch is permitted.

Five focused/KSR tests pass, including differing per-region external-flow
ownership. KSR freezes 3,272 external seam perimeters, 62,094 ordered
candidates, and FNV-1a checksum `11805973356074762675`. Strict core Clippy,
rustfmt, diff, macro, and sub-400-LOC gates pass; implementation/test shards
are 233/261 LOC.
