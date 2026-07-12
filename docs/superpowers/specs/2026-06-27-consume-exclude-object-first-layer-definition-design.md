# Consume `exclude_object` first-layer object definition geometry

## Problem

Ares already consumes OrcaSlicer `exclude_object` enough to emit Klipper
`EXCLUDE_OBJECT_DEFINE` plus object start/end markers, but the Klipper
definition still uses a fixed diamond polygon centered at `0,0`. That keeps the
registered option partly scaffolded: the emitted object definition does not
describe the actual first-layer print footprint produced by the current slice.

This slice replaces that static Klipper definition geometry with a definition
derived from Ares' finalized first-layer print paths. It continues the existing
single synthetic object boundary and does not add more option metadata.

## Upstream source boundary

Line numbers are from the vendored `OrcaSlicer/` tree in this repository.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1624` declares
  `((ConfigOptionBool, exclude_object))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3839-3843` defines
  `exclude_object` as a bool option used to add object-exclusion commands.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7989-8000` formats object polygons for
  G-code, closing the polygon by repeating the first point.
- `OrcaSlicer/src/libslic3r/GCode.cpp:8004-8050` implements
  `GCode::set_object_info`, including Klipper
  `EXCLUDE_OBJECT_DEFINE NAME=... CENTER=... POLYGON=...` and Marlin/RRF
  `M486` definitions.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5266-5288` emits object labels and
  exclude-object start commands.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5404-5427` emits object labels and
  exclude-object end commands.

## Current Ares gap

- `crates/ares-core/src/gcode_object_labels.rs:7` defines Klipper object info
  as `EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0
  POLYGON=[[1,0],[0,1],[-1,0],[0,-1],[1,0]]`.
- `crates/ares-core/src/gcode_object_labels.rs:49-50` returns that definition
  as a static string whenever Klipper `exclude_object` is enabled.
- `crates/ares-core/src/gcode.rs:101` writes the object definition before the
  first progress line, but it does not pass finalized print geometry to the
  object-label module.
- `crates/ares-core/src/gcode_first_layer_print_placeholders.rs` already
  computes first-layer print bounds from `LayerPrintPaths`; this slice reuses
  that boundary instead of adding an independent geometry pass.

## Ares destination boundary

- `crates/ares-core/src/gcode_object_labels.rs`
  - Change the Klipper object definition path from a fixed `&'static str` to a
    formatted `String` derived from first-layer print bounds.
  - Format the single synthetic object as a closed rectangular polygon using
    bounds corners in print-space order:
    `[min_x,min_y]`, `[max_x,min_y]`, `[max_x,max_y]`, `[min_x,max_y]`,
    `[min_x,min_y]`.
  - Format center as the midpoint of those bounds.
  - Keep Marlin and RepRapFirmware definitions unchanged.
- `crates/ares-core/src/gcode.rs`
  - Pass finalized `layer_print_paths` into `ObjectLabelConfig::object_definition`
    without increasing the file's LOC count beyond the 400-line guard.
- `crates/ares-core/src/tests/gcode_label_objects.rs`
  - Update focused end-to-end tests so Klipper `exclude_object` proves the
    definition reflects actual first-layer print bounds.

## Included behavior

- `exclude_object` parsing, validation, default-off behavior, supported flavors,
  unsupported-flavor no-op behavior, object start/end markers, and
  `gcode_label_objects` interaction remain as currently implemented.
- With `exclude_object=true` and `gcode_flavor="klipper"`, Ares emits one
  `EXCLUDE_OBJECT_DEFINE` after startup G-code and before the first M73 line.
- The Klipper definition's `CENTER` and `POLYGON` are computed from finalized
  first-layer print paths. With the existing `square_pyramid_ascii_stl()` test
  model and default skirts, the line is:

  ```text
  EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0 POLYGON=[[-2.5,-2.5],[2.5,-2.5],[2.5,2.5],[-2.5,2.5],[-2.5,-2.5]]
  ```

- With the same model and `skirt_loops=0`, the line is:

  ```text
  EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0 POLYGON=[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5],[-0.5,-0.5]]
  ```

- If no first-layer print bounds exist, Klipper emits no object definition.
  This avoids preserving the old fake diamond as a fallback while leaving
  start/end markers tied to actual emitted print moves.
- Numeric formatting uses the existing Ares `gcode_format::format_decimal`
  convention used by first-layer print placeholders.
- The Rust destination stays platform-neutral and WASM-compatible.

## Deferred behavior

- True Orca parity for multiple `PrintObject` instances, object names, object
  IDs, per-instance unique IDs, and per-instance convex hulls remains deferred
  until Ares has the corresponding `PrintObject` / `PrintInstance` runtime
  boundary.
- Bambu-printer object label-id comments, `M624`, `M625`, calibration object
  definitions, support-object exclusion, wipe tower, object skip flush, and
  absolute-E reset semantics remain deferred.
- This slice does not change slicing geometry, object ordering, by-object print
  sequencing, model import, registry metadata, CLI behavior, WASM bindings, or
  public API shape.
- Marlin and RepRapFirmware definitions stay name/id based and do not receive
  geometry because Orca's cited `M486` path does not include a polygon.

## Acceptance criteria

- Add or update focused E2E tests proving Klipper `exclude_object` emits a
  first-layer-bounds definition for default output.
- Add or update focused E2E tests proving `skirt_loops=0` changes the Klipper
  definition polygon from the default skirt-expanded bounds to the model
  perimeter bounds.
- Add a focused unit or integration test proving Klipper `exclude_object=true`
  emits no `EXCLUDE_OBJECT_DEFINE` when first-layer print bounds are absent,
  while preserving the existing Klipper start/end marker strings for any object
  move span that is still bracketed.
- Preserve existing E2E coverage for:
  - one Klipper definition/start/end sequence in the correct order;
  - Marlin and RepRapFirmware `M486` definitions;
  - `gcode_label_objects=false` suppressing comments without suppressing
    exclusion commands;
  - unsupported `repetier` accepting `exclude_object=true` as a no-op;
  - non-bool `exclude_object` rejection.
- Focused RED/GREEN verification uses `cargo nextest run -p ares-core
  gcode_label_objects`.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files remain at or below 400 LOC.

## Docs impact

This spec and its implementation plan are the behavior-tracking docs for the
slice. No roadmap edit is required unless a live roadmap section still describes
Klipper `exclude_object` definitions as static or metadata-only.

## Safety and simplicity

This is a narrow G-code export slice. It reuses finalized `LayerPrintPaths`,
existing object-label state, existing `GCodeFlavor` behavior, and existing
decimal formatting. It must not add dependencies, new crates, filesystem access,
terminal behavior, UI code, OpenGL, feature flags, or legacy fallback behavior.
