# Scarf seam-neighbor trimming

## Boundary and status

This is a public RED qualification contract, not an authorized production port.
The upstream owner is OrcaSlicer beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/ExtrusionEntity.cpp:444-484::trim_seam_ends`, called by
`ExtrusionLoopSloped` at `516-521` before slope construction. The v2.4.2
`8500fcdccaa10b5099ac20d252af3a7c560046f1` constructor lacks this prepass.

The Rust destination is `project_slice::gcode_emit::motion::scarf::build`
and a new private `scarf/trim.rs` leaf alongside the other scarf helpers.
The existing scarf module is a temporary materialization shell around upstream
`ExtrusionLoopSloped`, not an independently designed slicing pipeline.

## Included source semantics

Preserve the seam point and remove the adjacent vertex while its distance is
strictly below the source's fixed `scale_(0.05)` threshold. Process front and
back paths, including two-point paths that cross a path boundary. Clear fitting
metadata only when a path is modified. Recompute trimmed total length and clamp
caller-measured slope minimum length to it; preserve the caller's original
maximum segment spacing. Shared cached perimeter data must not be mutated.

The approved ramp-end vertex snapping remains unchanged. General clipping,
arc-fitting algorithms, GCode caller, speed, flow, progress, configuration
admission and emission are deferred. Clearing stale fitting metadata as required
by vertex removal does not authorize a new arc-fitting implementation.

## One fixed derived input

`scarf_seam_neighbor_trim/rear_low_motion_peak_scarf.3mf` derives from the
approved `rear_low_motion_peak.3mf` with SHA256
`be46fa7a81ee17aef5019dc3e341b1264e7261c551a60cbce91849842521c02e`.
Only `seam_slope_type` changes from `none` to `external` and `seam_gap` from
`10%` to `0`. All other settings, whitespace and ZIP entry payloads are unchanged.
Derived SHA256:
`e28178fae28e3abbe4ab6403997e74e29570cb0b63f5c1f4c3c92ff9fbb4308a`.

The existing ordinary-loop witness at Z4.4 has a roughly 0.021mm neighbor next
to its fitted seam. This motivates, but does not establish, scarf qualification.
Zero gap isolates the prepass from still-deferred clip behavior; no threshold,
geometry, seed, step count, min-length or wipe tuning is permitted.

## Qualification and acceptance

Use two fresh ordinary official beta Native exports and one direct
`ares_core::slice_project_plate_validated` invocation. Native must replay stably,
retain the fitted seam, and omit its old micrometre neighbor. The untrimmed API
neighbor must remain observable for a discriminating RED. The comparison retains
the entire Z4.4 object: all travel, M73, XYZ/E, feeds, start/flat/end scarf moves,
wipe and terminal motion. No motion omission, sorting, rotation, normalization,
tolerance or precision tuning is allowed.

If the premise is absent, unstable or confounded, stop without another input.
A qualified RED requires separate review before implementation. A future GREEN
must match the complete object byte-for-byte, otherwise retain an open RED and
localize the residual to a cited owner. Existing whole-file ledger rows are not
closed by this focused qualification. Production diagnostics and private planner
tests are not part of this boundary.
