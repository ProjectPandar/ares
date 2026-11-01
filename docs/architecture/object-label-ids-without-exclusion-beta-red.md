# Unconditional print object label IDs

Source authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`GCode.cpp:4336–4340` calls `assign_object_label_ids` before the exclusion
gate; `:11356–11370` numbers PrintObjects and their instances. `Print.hpp`
get_id/set_id and GCode.cpp:7608,7764,7842 consume those IDs in comments.
Rust destination: `project_slice/gcode_emit/object.rs::ObjectLabels::from_traversal`.
The exclusion-gated zero IDs are a stale compatibility shell; replace them
with the source print-object and per-object instance enumeration for every
printer. Exclude-object definitions remain separately gated.

Included public behavior: ordered raw object start/stop comments when
exclusion is disabled. The frozen overhang input has SHA-256
`71e2690d381d7698a94ce5c6d53eb2da5359fe00de18029c26c10a8b707a1695`.
The test obtains runtime Native double replay, then compares the public
`slice_project_plate_validated` result. No expected G-code is checked in.
The two-input qualification closes all 200 overhang and 20 shared-scale
object label records exactly. All nine refreshed candidate failures have
exclusion disabled and the same stale id:1-to-id:0 gate symptom; only two
inputs were dynamically qualified. This is not complete-output acceptance.

Deferred: model loaded-label IDs, exclusion outlines and commands, template
object indices, object naming, other unrelated G-code differences. Existing
strict complete-output comparators remain unchanged. External evidence is
retained outside the repository.
