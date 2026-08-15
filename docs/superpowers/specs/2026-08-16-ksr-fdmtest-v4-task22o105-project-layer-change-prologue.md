# Spec: Task 22O.105 project layer-change prologue

## Source boundary

Port the initial filament-start and layer-loop emission from
`OrcaSlicer/src/libslic3r/GCode.cpp` and the machine preamble from
`OrcaSlicer/src/libslic3r/GCodeWriter.cpp::preamble` into
`crates/ares-core/src/project_slice/gcode_emit.rs`.

## Requirements

1. Emit typed-project layer height markers and evaluate the typed
   `layer_change_gcode` for each layer.
2. Emit the upstream writer preamble and initial filament-start framing without
   fixture identification or reference-G-code reads.
3. Keep output modules below 400 LOC and retain focused parser/layer tests.
4. Defer timing, exact motion feedrates, and convex-hull translation to later
   source-cited slices.
