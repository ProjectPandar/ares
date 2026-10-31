# Plane-path low-density top alignment beta RED

## Source boundary

Pinned beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/Fill/FillPlanePath.cpp::_fill_surface_single`, lines 93–113,
changes alignment from `density < 0.995` to
`extrusion_role == erInternalInfill`. A low-density top surface therefore uses
its snug surface bounds instead of whole-object bounds. The default
`center_of_surface_pattern` is each surface; nondefault centering is deferred.

Rust destination: `ares-core/src/fill/plane_path.rs::fill_component` and existing
fill-entity/ironing/bridge-anchoring parameter callers. The density heuristic is
temporary compatibility with the old upstream boundary, to be replaced by the
source role predicate, not a new Ares pipeline.

## Discriminating public RED

`tests/parity/plane-path-low-density-top/hilbert.3mf` retains existing
`plane-path-object-bounds/hilbert-compensation01.3mf` geometry and metadata.
Its process selects 60% Hilbert top surface, monotonic bottom surface,
rectilinear sparse fill, no prime tower and disabled layer-cooling slowdown.

The initial 40% candidate exposed adjacent connection and cooling behavior:
`FillPlanePath.cpp:150` calls `connect_infill` below 50%, whereas above 50% it
chains the clipped paths directly. Cooling also changed the emitted feedrate.
The revised input selects the direct-chain branch and disables cooling, so the
asserted extrusion records isolate the role-based alignment decision.
Qualification shows the source role predicate changes API top extrusion records
from 22 to 119; the latter are byte-equal to the runtime Native's 119 records.
No partial-match metric is the exit criterion.

The public test uses the existing runtime Native double-replay qualification
helper, then invokes `slice_project_plate_validated` directly. Its observation
follows `compatible_regions/top_pattern.rs::observe`: select Top surface G1
records with XY and E, stopping at wipe retraction. Compare every selected line
in order, byte-for-byte; no numerical rewriting, sorting or computed expectation.
M73, travel, wipe and footer are outside this extrusion-path boundary. The test
asserts Native pattern/density and nonempty activity. Native output is never
checked in. Registration remains inside the existing Native stream group.

Included is role-based alignment under default surface centering. Nondefault
centering, fill order, corner smoothing, connection behavior, shared Clipper2,
cooling/emission/processor and footer changes are deferred. RED contains no
production changes. The earlier full-file comparison was rejected because a
processor mismatch preceded the owning decision; it is not this slice's gate.
