# End-gcode M400 dwell skip

## Upstream boundary

This slice ports the end-gcode delay state from
`src/libslic3r/GCode/GCodeProcessor.cpp` and its producer from
`src/libslic3r/GCode.cpp`.

The processor source defines the machine-end marker at
`GCodeProcessor.cpp:186`, resets its skip state in `GCodeProcessor::reset` at
`:3732`, recognizes the exact comment at `:4531–4536`, and handles M400 at
`:6947–6960`. The producer writes the marker after the Custom role tag and
before filament-specific and machine-end templates at `GCode.cpp:4813–4820`.

The Rust destination is the existing estimator command flow and the existing
end-template append boundary. This is a source-cited compatibility rewrite,
not a new timing pipeline.

## Included behavior

- Reset the end-gcode skip state at the start of every processor input.
- Recognize only the exact whole-line machine-end marker.
- After the marker, skip valid `M400 S` and `M400 P` delays before planner
  synchronization.
- Preserve source operand precedence: a valid `S` value takes priority over
  `P`.
- Preserve G4 seconds and milliseconds behavior, including precedence and
  conversion.
- Preserve zero-delay firmware-specific M400 synchronization behavior.
- Emit the marker only when `enable_pre_heating` is enabled, after Custom
  output and before both end-template classes.

## Deferred behavior

This slice does not change arc precision, other numeric parsing, scheduler or
lookahead arithmetic, timing-cache attribution, M73 formatting, template
contents, filament behavior, machine-start handling, or unrelated geometry and
routing behavior.

## Non-negotiable decisions

- The marker comparison is exact; substring and inferred boundary detection are
  not equivalent.
- The skip applies only to M400 commands carrying a valid S or P value.
- The skip occurs before synchronization, not after a delay has been queued.
- G4 is never suppressed by the marker.
- State is local to one processor run and cannot affect a later input.
- Existing arithmetic, ordering, parsing, and output comparison rules remain
  unchanged outside this source boundary.
