# Task 22O.103 architecture decision record

## Status

Accepted as an active vertical lifecycle slice. Decision date: 2026-08-16.

## Source boundary

This slice is bounded by the pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/FillConcentricInternal.cpp:13-95` for dispatching
  concentric internal surfaces into wall-tool-path output;
- `src/libslic3r/GCode.cpp:3950-3995` for machine-envelope prologue
  serialization;
- `src/libslic3r/GCode.cpp:4539-6228` for ordered layer/entity emission;
- `src/libslic3r/GCodeWriter.cpp` for extrusion state and movement output.

## Decision

Activate the already typed project lifecycle through a crate-private,
in-memory emitter. `slice_project` now retains the prepared ordered entities
long enough to emit a project G-code byte stream, then disposes the ownership
chain. The emitter derives header/config/width/machine values from the resolved
3MF settings and emits perimeter, fill, and thin-wall entity geometry without
reading the reference G-code or recognizing the fixture.

The current concentric implementation is a temporary compatibility shell around
the cited `FillConcentricInternal` boundary: it uses the existing Clipper
offset kernel to materialize nested closed paths. Full Arachne `WallToolPaths`,
transition filtering, seam placement, motion planning, placeholder evaluation,
arc fitting, timing, and exact G-code formatting remain deferred source-cited
work and are not claimed as parity by this slice.
