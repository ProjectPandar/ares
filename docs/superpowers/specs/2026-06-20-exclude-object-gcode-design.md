# Exclude Object G-code Runtime Design

## Goal

Consume the already-registered `exclude_object` option as concrete G-code output behavior. Ares should keep the current single-object pipeline but emit OrcaSlicer-style object exclusion markers when `exclude_object` is enabled, instead of leaving the option as metadata only.

## Upstream Boundary

Line numbers are from the vendored `OrcaSlicer/` tree in this repository.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1624` declares `((ConfigOptionBool, exclude_object))` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3839-3843` defines `exclude_object` as a `coBool` option, default `false`, whose purpose is adding exclude-object commands to G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5486-5492` copies `print_config.exclude_object` into the G-code generator state.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2661-2663` emits object-info definitions when exclude-object support is enabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5266-5288` emits object start labels or exclude-object start commands when an object starts.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5403-5427` emits object stop labels or exclude-object end commands when an object finishes.
- `OrcaSlicer/src/libslic3r/GCode.cpp:8004-8045` formats Klipper `EXCLUDE_OBJECT_DEFINE` and Marlin/RepRapFirmware `M486` definitions for object instances.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1167-1184` flushes pending object start/end labels before movement and resets absolute E after object-end labels.

## Current Ares Gap

`crates/ares-core/src/gcode_object_labels.rs` already consumes `gcode_label_objects` and emits one synthetic object start/stop comment around the first and last print moves. The adjacent `exclude_object` option is present in the registry and metadata tests, but `rg -n 'exclude_object|EXCLUDE_OBJECT|M486' crates/ares-core/src` shows no runtime parser or emitted exclusion command outside metadata tests.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_object_labels.rs`: parse `exclude_object`, format the single-object exclude definition, and extend the object-label state to emit exclude start/end commands around existing print moves.
- `crates/ares-core/src/gcode.rs`: wire the parsed exclude-object state into G-code formatting with minimal calls so the file stays below 400 LOC.
- `crates/ares-core/src/tests/gcode_label_objects.rs`: add end-to-end G-code tests for default-off behavior, Klipper output, Marlin/RRF output, and invalid option values.
- No registry metadata, slicing geometry, object ordering, multi-object model import, Bambu label-id commands, object skip flush, wipe tower, or filesystem/UI behavior is introduced.

## Included Behavior

- Missing `exclude_object` defaults to `false`.
- Non-bool `exclude_object` values return `SliceError::InvalidInput`.
- With `exclude_object = false`, emitted G-code remains unchanged except for any pre-existing `gcode_label_objects` comments.
- With `exclude_object = true` and `gcode_flavor = "klipper"`, Ares emits:
  - one executable-block definition after `gcode_start_custom::start_gcode(options)?` and before `gcode_m73::first_progress_line(options)?`: `EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0 POLYGON=[[1,0],[0,1],[-1,0],[0,-1],[1,0]]`
  - one object start marker immediately before the first object print move: `EXCLUDE_OBJECT_START NAME=ares-object-0`
  - one object end marker after the last object print move and before final progress/finish G-code: `EXCLUDE_OBJECT_END NAME=ares-object-0`
- With `exclude_object = true` and Marlin-family flavors (`"marlin"`, `"marlin2"`) Ares emits:
  - one executable-block object definition sequence after `gcode_start_custom::start_gcode(options)?` and before `gcode_m73::first_progress_line(options)?`: `M486 S0`, `M486 Aares-object-0`, `M486 S-1`
  - one object start command before the first object print move: `M486 S0`
  - one object end command after the last object print move: `M486 S-1`
- With `exclude_object = true` and `gcode_flavor = "reprapfirmware"` Ares emits:
  - one executable-block definition after `gcode_start_custom::start_gcode(options)?` and before `gcode_m73::first_progress_line(options)?`: `M486 S0 A"ares-object-0"` followed by `M486 S-1`
  - object start/end commands `M486 S0` and `M486 S-1`.
- Unsupported active Ares flavors keep `exclude_object=true` as a no-op for exclusion commands. In current public parsing this means `repetier` accepts the bool but emits no `EXCLUDE_OBJECT`/`M486` commands.
- `gcode_label_objects` comments remain independently controlled. If both `gcode_label_objects` and `exclude_object` are true, both comment labels and exclusion commands are emitted.
- The hard-coded single-object name is `ares-object-0`, matching the existing synthetic label name. The hard-coded Klipper polygon is the current deterministic diamond footprint used by `square_pyramid_ascii_stl()` in the Ares E2E tests, centered at `0,0`; full object instance bounds remain deferred.
- When both `gcode_label_objects` and `exclude_object` are true, the first printable object move is preceded in this exact order by `; printing object ares-object-0 id:0 copy 0`, then the active flavor's exclude-object start command, then the first `;MOVE:` / G-code move pair. After the last printable object move, Ares emits `; stop printing object ares-object-0 id:0 copy 0`, then the active flavor's exclude-object end command, then final progress/finish G-code.

## Deferred Behavior

- Real ModelObject / PrintObject instance names, ids, centers, convex hulls, multiple objects, by-object scheduling, and per-instance unique ids beyond the single synthetic object.
- Bambu printer label-id comments and `M624` / `M625` commands.
- `support_object_skip_flush`, purge tower, support object exclusion, calibration modes, and object skip flush behavior.
- `GCodeWriter` pending-label buffering and absolute-E reset after skipped objects; Ares already writes relative E by default and this slice does not alter E-mode semantics.
- Arc fitting, power-loss recovery, first-layer scanning, printer-specific object metadata, and any independent Ares pipeline concept.

## Acceptance Criteria

- Unit tests prove `exclude_object` defaults to disabled, accepts booleans, and rejects non-bool values.
- E2E G-code tests prove default output contains no `EXCLUDE_OBJECT` or `M486` object-exclusion commands.
- E2E G-code tests prove Klipper output contains exactly one define/start/end sequence in the correct order relative to object comments and moves.
- E2E G-code tests prove Marlin and RepRapFirmware output use Orca-cited `M486` object definition and start/end commands.
- E2E G-code tests prove `gcode_label_objects=false` suppresses comments without suppressing `exclude_object` commands.
- E2E G-code tests prove `exclude_object=true` with unsupported active flavor `repetier` emits no exclusion commands.
- `cargo nextest run -p ares-core gcode_label_objects` passes.
- `cargo nextest run -p ares-core` passes.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass before completion.

## Docs Impact

This SDD spec and its implementation plan are the behavior-tracking docs for this slice. Update `docs/roadmap.md` only if a live roadmap section still says `exclude_object` is metadata-only after this implementation.

## Safety And Simplicity

This is a narrow G-code-export slice. It reuses existing `SliceOptions` storage, the existing `GCodeFlavor` parser, and the current single-object object-label state. It should not add dependencies, crates, public APIs, broad config hierarchy, or a general multi-object object-exclusion subsystem.
