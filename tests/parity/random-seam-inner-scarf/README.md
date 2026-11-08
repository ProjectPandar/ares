# Random seam with inner-wall scarf

The positive and control projects were exported by OrcaSlicer 2.4.2.
`manifest.json` records their archive hashes, ZIP payload hashes and
settings. Only `Metadata/project_settings.config` differs, and its only
setting difference is `seam_slope_inner_walls`: `"1"` for positive,
`"0"` for control. Geometry and every other payload are identical.

Both use random seams, staggered inner seams, three walls, Inner/Outer
wall order and Contour scarf. Wipe is disabled with `wipe = ["0"]`.

## Native motion contract

The positive case must contain an inner-wall scarf ramp: an extruding
G1 move with XY motion and changing Z within a layer's Inner wall block.
The control must have no such activity. Layer transitions, other roles,
constant Z and travel moves do not satisfy this property. The test uses
live Native output, not a fixed motion line, coordinate, E value or count.
The `seam_slope_inner_walls` and `wipe` configuration premises also apply.

Both cases reuse `runner/native_api_cli.rs::NativeCase` to compare complete
Native streams with the validated public API and bare CLI. Only the
existing generator identity/date-line exception applies. The complete
comparison includes motion, M73 and footer data, with raw API/CLI equality.

The upstream boundary is `SeamPlacerImpl::pick_random_seam_point` in
`libslic3r/GCode/SeamPlacer.cpp`, normalized-edge selection at lines
1441–1443 of `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The [historical arithmetic analysis](../../../docs/architecture/2026-10-20-random-seam-inner-scarf-spec.md)
describes the original source port, not the current Native motion premise.
