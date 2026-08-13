# Task 22O.80 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:1590-1629,1711-1931`, into
`fill::rectilinear::regions`.

Generate ordered monotonic regions from O79 linked vertical sections. Each
region owns left/right vertical boundaries and low/high intersection indices,
source flip parity, and later-populated neighbor/length fields. Seed scanning,
vertical-run extension, exclusive right/left overlap, consumed marking, and
termination order follow source.

Deferred: neighbor scattering/path lengths, ant path/chaining, polyline output,
entities, lifecycle, and G-code. No legacy fallback, fixture branch, or public
API.

Two focused tests pass for rectangular odd/even flip parity and immutable,
repeatable generation. Strict core Clippy, rustfmt, diff, and sub-400-LOC checks
pass; the region shard is 168 LOC.
