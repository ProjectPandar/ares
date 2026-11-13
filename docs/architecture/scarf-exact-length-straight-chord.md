# Exact scarf ramp length beside a straight vertex

## Upstream boundary

OrcaSlicer `40ecc66f7`, `src/libslic3r/ExtrusionEntity.cpp:444-446,487-536`,
`split_at_slope_end` and `ExtrusionLoopSloped`, implements the change from
`f2ad1c8536b8ff40273047431c52ad48088389f1`.
`src/libslic3r/Point.hpp:435-439` defines integer-coordinate interpolation.

The Rust destination is `ares-core`'s
`project_slice/gcode_emit/motion/scarf.rs::split_path`. Its existing sloped-loop
representation remains a temporary compatibility shell around
`ExtrusionLoopSloped`, not an independently designed slicing pipeline.

## Included behavior

When the exact ramp endpoint is less than half a slope step (capped at 1 mm)
before a vertex, examine the chord to the following vertex. If the ramp length
is at least the scaled epsilon and the intermediate vertex is within 0.05 mm
of that chord in three dimensions, keep the exact integer-interpolated ramp
endpoint and omit the intermediate vertex from the flat path. Otherwise extend
the ramp to the intermediate vertex as before. Both paths share the endpoint.

Use the current geometry coordinate scale for the epsilon and tolerance.
Preserve path metadata and invalidate arc fits on split paths as upstream does.
The surrounding seam trim, slope subdivision, speed selection and flow
calculation are unchanged. Other extrusion-entity operations and general
three-dimensional path-length changes are deferred.

## Failure modes and behavioral verification

An unconditional snap makes adjacent layers end their ramps at different
positions. An unconditional skip removes real corners or changes a Z contour.
Skipping the final vertex loses the end of a path; inconsistent interpolation
separates the slope and flat endpoints.

Exercise the existing public project-to-G-code API against runtime OrcaSlicer
output using the boundary, absolute-speed, percentage-speed and above-role
projects. Compare complete streams for all four speed cases and the complete
second-layer object for the endpoint case. The boundary project emits the exact
ramp step `G1 X107.5 Y105 Z.25` while retaining its distinct scarf and flat
feedrates. Keep the adjacent corner and seam-trimming public comparisons.
