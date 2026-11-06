# Spiral lifts inside the active physical extruder's printable area

## Decision and upstream boundary

Rewrite the printable-area admission owned by
`OrcaSlicer/src/libslic3r/GCodeWriter.cpp` at upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `apply_print_config:181–225`: capture the bed polygon and the per-extruder
  polygons, including intersection with the bed and largest-contour selection;
- `active_printable_area:229–239`: select the active filament's physical
  `extruder_id()`, then the bed polygon when no dedicated valid polygon exists;
- `spiral_lift_fits_printable_area:241–260`: admit only circles whose center is
  contained and whose radius does not exceed the distance to any polygon edge;
- `eager_lift:950–965` and `travel_to_xyz:1023–1036`: check admission before
  emitting a spiral, otherwise emit the upstream normal Z lift.

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/motion/travel/lift.rs`, with a
small sibling `lift/printable_area.rs` for the stored polygons and predicate.
Initialize the writer-owned area data once from the applied printer settings
through emission orchestration/`motion::EmitState`; choose the active physical
extruder from existing applied writer state. Do not rebuild clipped polygons
on every lift, copy them into each region's `MotionOptions`, or introduce a
second filament map. The existing `emit_pending` and `append_eager` functions
are temporary compatibility shells around these upstream writer operations:
replace their unconditional spiral admission, not the writer lifecycle.

This geometry is owned by core and must work on browser WASM, Windows, macOS,
and Linux. It requires no filesystem, catalog lookup, terminal, viewer or
platform runtime. The processor's printable-area result tracker is downstream
of motion emission and is not a substitute for the writer's admission check.

## Included semantics

Keep full polygons, not their bounding boxes. Match the upstream coordinate
scaling, orientation, polygon intersection, largest-area contour selection,
center containment and squared point-to-segment distances. An extruder area
with fewer than three points uses the bed polygon; when intersection produces
no contour, upstream retains the original extruder polygon. An unknown active
area admits the spiral, exactly as the source function does. These are upstream
branches, not newly invented validation or compatibility fallbacks.

Select dedicated areas by physical extruder, not logical filament, total head
count, configured first head or used-head maximum. The area remains selected
from the current writer filament when tool selection changes.

For eager lifts, subtract the writer's plate XY offset before constructing the
circle center `position + (radius, 0)`. For deferred lifts, use the source and
perpendicular `ij_offset` already calculated by `travel_to_xyz` in bed
coordinates. Do not subtract the nozzle offset again. A circle is rejected if
the center is outside the polygon or an edge is closer than its radius; do not
replace the strict upstream edge comparison with an epsilon or axis-aligned
clamp. Tangency follows the existing polygon containment and strict distance
comparison, not a separately designed boundary policy.

Both rejected-circle branches emit the existing normal lift to the same
raised Z, with existing feedrate/comment formatting and lifted/pending state
updates. Admitted circles retain `_spiral_travel_to_z:1200–1275` behavior,
including enabled-arc emission and disabled-arc discretization. Do not alter
radius arithmetic, segment counts, coordinate formatting or ordinary travel.

## Excluded behavior

No MAX_LIMIT reduction, machine-envelope output, processor configuration,
planner, time estimate, header or M73 correction; no comparator changes or
output overrides. A time/M73 change may follow changed motion, but is never
implemented by patching statistics. H2D cooling/feedrate residuals, J1 numeric
fidelity, tower/script transitions, filament-map recommendation, retraction
policy, lift-band/enforce decisions and axis-remap admission are deferred.

## Tests

The `tests/parity/front-edge-numeric-cursor` projects exercise public Native,
validated API and CLI slicing with complete ordered-byte comparison. Native
replay identity allows only the existing generator-line normalization. API and
CLI outputs must match as complete raw bytes.

- `control.3mf`, SHA-256
  `0f5ff07e6145155275ba82bbedfd608569117a3ee30a26309686efba2764f0c1`,
  asserts normal Z lifts where nine circles would cross the front bed edge.
- `target.3mf`, SHA-256
  `d70b49101439c4a040cad4bbba3baf728d0d7d39d97963e59718ed245a5087db`,
  asserts normal Z lifts where eighteen circles would cross that edge after
  the cursor offset. Neither case permits the outside-bed spiral points.

## Failure modes

1. Using a bounding box admits circles that cross a nonrectangular bed edge.
2. Selecting by logical filament uses the wrong physical extruder polygon.
3. Missing or double-applying the plate offset checks a different circle.
4. Gating deferred lifts but not eager lifts leaves the latter outside the bed.
5. Checking only arc G-code leaves disabled-arc spiral segments unrestricted.
6. Rejecting admitted interior circles changes valid motion unnecessarily.
7. Treating missing dedicated areas or empty intersections differently from
   upstream changes which bed or extruder polygon owns admission.
8. Changing the lifted/pending state when emitting the normal alternative
   changes subsequent travel or unlift behavior.
