# Consume `gcode_label_objects` Design

## Purpose

Consume the existing `gcode_label_objects` print option in real G-code output instead of adding more option metadata. This slice ports the OrcaSlicer object-label comment behavior that brackets object printing moves with human-readable start and stop comments.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3831-3837` defines `gcode_label_objects` as a boolean print option with default `true`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5266-5270` emits `; printing object ... id:... copy ...` before object extrusion when `gcode_label_objects` is enabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5403-5408` emits `; stop printing object ... id:... copy ...` after object extrusion when `gcode_label_objects` is enabled.

Adjacent but deferred upstream behavior:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3839-3843` `exclude_object` is not part of this slice.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5272-5295`, `5410-5424`, and `8020-8044` object exclusion commands and object definitions are deferred because Ares does not yet have multi-object instance IDs, object polygons, or firmware-specific cancel-object command semantics.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:111-116` and `GCodeWriter.cpp:1167-1179` pending object-label buffers are not ported in this slice because Ares currently emits a single flattened move stream and can insert comments directly at the object print span.

## Current Ares Boundary

- `crates/ares-core/src/print.rs` currently builds a `Print` with one `PrintObject` from the pipeline layer print paths.
- `crates/ares-core/src/gcode.rs` emits all flattened extrusion moves for the pipeline.
- `crates/ares-core/src/options.rs` is near the 400 LOC limit, so this slice must not grow it; a small G-code option parser module will read `SliceOptions::values()` and preserve the same boolean parsing behavior.

## Design

Add a small `gcode_object_labels` module that owns formatting for the scoped object label comments and boolean parsing for this G-code-only option. `format_gcode` will parse `gcode_label_objects` with Orca's default of `true`. When enabled and the pipeline has printable extrusion moves, it will emit:

```gcode
; printing object ares-object-0 id:0 copy 0
...
; stop printing object ares-object-0 id:0 copy 0
```

The label uses a deterministic Ares-local name because the current input model does not preserve Orca `ModelObject::name`. The object id and copy id are `0`, matching the single-object print domain currently produced by `build_print_domain`.

The comments must bracket the printable move stream, not metadata-only sections. Start is inserted immediately before the first emitted extrusion/travel print move in the layer move loop. Stop is inserted immediately after the last emitted move in that same loop. This keeps startup, layer headers, comments, temperature commands, and finish G-code outside the object span.

When `gcode_label_objects` is `false`, no object label comments are emitted.

## Acceptance Criteria

- `gcode_label_objects` is parsed as a boolean runtime option with default `true`.
- Default G-code for a normal rectangular test pipeline contains exactly one object start label and one object stop label.
- Setting `"gcode_label_objects": false` removes both labels while leaving existing move output intact.
- The start label appears before the first emitted motion command in the print move loop, and the stop label appears after the last emitted motion command but before final progress/finish G-code.
- The implementation does not add `exclude_object`, object definition commands, firmware-specific cancel-object commands, new dependencies, or a new object model.
- Rust source files remain at or below 400 LOC.

## Tests

Add focused `ares-core` tests that:

- Build a rectangular pipeline and assert default labels are present once.
- Build the same pipeline with `gcode_label_objects=false` and assert labels are absent while `;MOVE:` output remains.
- Assert label ordering relative to an emitted `;MOVE:` marker and stable finish output already present in Ares G-code, such as `M73 P100 R0` before `M2`.

## Documentation Impact

This spec is the documentation for the slice. No user-facing CLI or WASM docs change is required because the option already exists in the option registry; this change consumes it in generated G-code.

## Safety

The change is local to G-code text emission. It does not alter geometry, extrusion math, speed planning, temperature commands, filesystem access, or WASM boundaries. If needed, rollback is removing the new module, tests, and the `format_gcode` calls.
