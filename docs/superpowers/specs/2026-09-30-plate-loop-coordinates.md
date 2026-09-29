# Skirt and brim plate-coordinate emission

## Source boundary and destination

Port the observable coordinate conversion of `GCode::generate_skirt`
(`GCode.cpp:4411`, zero-offset callers `:4482,:5249`), `generate_object_brim` (`:4499`), and
`point_to_gcode` (`:8124-8128`) into
`ares-core/src/project_slice/gcode_emit/motion/path.rs`.
Both loop generators emit plate-space integer paths with origin zero.

## Failure

The Sovol SV06 High-Speed smoke emits Y98.189 instead of Y98.188.
The original Orca's rounding input is 98188.499999999985; Ares produces
98188.5 by unscaling local Y=-11811500 before adding origin 110.
Translating the integer point into plate coordinates before unscaling
preserves the upstream rounding result.

## Scope

Skirt/brim constant-path deposition and the initial travel destination
use integer plate translation before unscaling and extruder-offset
subtraction. Preserve local integer cursor/wipe paths, route boundaries,
and object-path coordinate conversion. This keeps the current local-frame
scaffold as a temporary representation of upstream paths; it does not
introduce a different slicing pipeline or change rounding tolerances.

Full upstream origin-state refactoring, unsupported skirt grouping, and
unrelated geometry differences are deferred.

## Acceptance

- Sovol real-Orca smoke and a plate-loop emitter regression fail before
  the fix and pass after it.
- Cover skirt/brim, nonzero extruder offset, initial travel, and retained local cursor/wipe points.
- Recheck other coordinate-residual printers with complete output.
- Remove temporary probes; run fmt, Clippy, workspace Nextest and
  independent six-aspect review before committing and pushing.
