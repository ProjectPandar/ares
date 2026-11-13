# Arc extrusion delta storage

## Upstream boundary

Authority: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode/GCodeProcessor.cpp::process_G2_G3`.

- Lines6178–6181 store `end_position[E] - m_start_position[E]` into
  `std::optional<float> extrusion`, before the firmware-specific segmentation.
- The MarlinFirmware branch multiplies that float by its float inverse segment
  count and stores a float extrusion step.
- Lines6319–6322 promote the stored float for the non-Marlin branch's double
  `extruder_per_segment = *extrusion * inv_segment`.
- Both branches retain the original double `end_position` for their last
  segment. Rounding subdivision input does not round the commanded endpoint.

These numeric boundaries also exist at beta authority
`345efec64c7b95a05ce5cf96df0dde66a158835c`; the extrusion-admission and
non-Marlin step expressions are identical to the release expressions.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/processor/motion/arc.rs` owns
`targets`, `ArcGeometry`, `marlin_targets`, and `legacy_targets`, which implement
this upstream `process_G2_G3` slice.

The existing geometry shell carries the unrounded f64 E difference for the
exact last target. Add its upstream f32 extrusion value once, before firmware
branch selection. Marlin subdivisions multiply in f32; non-Marlin subdivisions
promote that f32 to f64 before multiplication. Keep original f64 endpoint
calculation, segment count, XY/Z geometry, feedrate and dispatch unchanged.
There is no alternate estimator or statistics-only correction.

## Shared estimation boundary

`MotionState::motions_indexed` consumes `arc::targets` for every discretized
motion, including its final double endpoint. `Estimate::record` consumes those
same motion blocks for both the planner and move recording. Their numeric
changes can therefore affect elapsed time, exported M73 progress, and estimated
time text as well as filament statistics. The double-position interpolation
slice introduced by Ares `a73d324dd38aa285aceb4d326e34ea86fec15644` remains in
place; only the upstream extrusion input store and multiplication are corrected.
No statistics-only implementation or time-byte exclusion is permitted.

## Straight G1 is outside this change

`GCodeProcessor.cpp:5206–5223` admits float reader coordinates promoted to
`optional<double>` for ordinary firmware, and precise decimal doubles for
Klipper. Its double-position overload at5226 retains double axis coordinates;
movement differences at5287–5291 are `AxisCoords` doubles. In particular,
there is no `optional<float>` store of the complete G1 E difference.

The later `float volume_extruded_filament = area_filament_cross_section *
delta_pos[E]` at5336 is a distinct, already-ported volume storage boundary.
Do not introduce a float E-difference store in straight G1, or change
`UsedFilaments`, its role-cache precision, flush order, diameter/density
handling, unit conversions or final role values.

## Included and deferred behavior

Included: the shared arc extrusion-difference f32 store, subsequent
firmware-specific multiplication types and unchanged double final endpoints.
The existing motion shell remains an implementation of the cited upstream
processor concept, not an independent Ares motion pipeline.

Deferred: unrelated parser, arc geometry, planner/estimator algorithms,
G-code-emission rules, role detection and statistics behavior. Numeric effects
of the shared corrected arc targets on time and progress remain included. The
correction does not reinterpret machine-generated G-code or alter accepted inputs.

## Verification contract

Before implementation, add an end-to-end processor regression using the real
Native role-map producer on exact retained slice output. Exercise extruding
arcs whose E difference is not exactly representable in f32; compare all role
meters/grams bits without tolerance. Keep the six existing role-map/wipe-tag
cases and complete processor-output comparisons. Include straight G1 and
firmware-branch controls where their existing processor tests provide them.
For both firmware branches, compare real Native and Rust non-internal arc-move
E deltas bit-for-bit, including the distinct last delta that carries the original
endpoint residual; retain both vectors as test artifacts.

Derive the wider test inventory from arc-bearing Native G-code retained in the
authoritative two-mode upstream run, then cross-reference registered tests.
Run those complete-output `orca_parity` cases and `gcode_moves_oracle` against
both unchanged base core and corrected core, in official and seeded modes.
Retain the inventory, Native stream hashes and per-case results. Use the
established generator-line-only normalization, with no additional G-code
exclusions, rounding or relaxed comparisons; M73 and estimated-time bytes are
part of the complete output.
