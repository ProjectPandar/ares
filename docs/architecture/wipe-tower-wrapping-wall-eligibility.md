# Wipe-tower wrapping-wall eligibility

## Source and Rust boundaries

Port the constructor predicate for `m_enable_wrapping_detection` at
OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode.hpp:126`, into
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/integration.rs`.
The existing `Integration::extra_wall` is a temporary shell for the upstream
constructor's timelapse/wrapping state, not an independent tower policy.

A wrapping wall is enabled only when the raw wrapping option is true,
`wrapping_exclude_area.size() > 2`, and `slice_used_filaments.size() <= 1`.
Preserve the independent Smooth-timelapse alternative. Obtain the actual
used-filament slice from the existing `ToolOrdering::all_extruders` set at
integration construction; do not count configured slots or rebuild a second set.

This predicate is deliberately different from
`GCode/WipeTower.cpp:53–57`'s `wipe_tower_sparse_layers_skipped`. Sparse skipping
uses the raw wrapping option, even when no wrapping wall is eligible. Do not
substitute one predicate for the other or normalize the stored option.

The constructor currently treats any raw wrapping option as an extra wall.
On a two-filament project this incorrectly consumes the first result before
the normal toolchange dispatch, which can index a nonexistent second result.
Correct the source predicate, not the index access: no bounds fallback, skipped
case, synthetic result or extra error path is part of this port.

Passing the existing used-filament slice touches the constructor call in
`gcode_emit/orchestration.rs`. Keep that module below 400 lines by moving its
existing tower-priming overlap calculation into `orchestration/tower.rs`.
Read configuration and plate position from the existing traversal in
`Integration::new`, keeping each function within five arguments. Keep the
initialization calculations unchanged apart from passing the used set; this is
file-boundary maintenance, not a new slicing stage or public API.

## Native/public-API contract

First retain the existing failing Type1 wrapping scenario: the raw option is
on, the exclusion region is empty, and two logical filaments are used. Native
succeeds with a full-height tower; Ares currently panics in tower dispatch even
after correcting sparse-skipping eligibility.

Add a second same-input Native/public-byte-API scenario with a valid four-point
wrapping region and two used logical filaments. End-filament macros expose
`filament_extruder_id` so Native establishes both IDs are actually used, not
merely configured. The used-filament condition must still disable the extra
wrapping wall. Require successful slicing, no compacted-tower restoration lifts,
and tower extrusion at least at the highest previously printed object Z. Keep
the existing parser and all original eight restoration contracts unchanged.

Retain complete input, Native/public-API G-code, Native used-ID observations,
height/entry observations, child status receipts and the unchanged full-output
comparison result. Height success does not imply complete output equality.

The existing empty-region scenario observes the disabled area/used-set case;
the new valid-region scenario observes the used-set restriction independently
of region validity. The positive single-used-filament wrapping-wall path is not
independently asserted by these scenarios. It is not disabled by the source fix,
but no new parity claim is made for its wall geometry or detection macro.
Other wrapping geometry, detection scheduling, tower planning and clearance
validation remain deferred to their owning upstream slices.

Commit this specification, then the Native/public-API failing test, then the
constructor correction separately. Place this prerequisite group before the
independent layer-Z restoration implementation in the final stack. Preserve
prior failing runs and source bindings, and run both final-release modes plus
strict workspace/all-targets Clippy, rustfmt and ordinary browser WASM checks.
