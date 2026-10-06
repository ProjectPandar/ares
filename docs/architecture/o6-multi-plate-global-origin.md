# O6: selected plate grid origin and plate-local G-code

## Source boundary and Rust destination

OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `slic3r/GUI/PartPlate.hpp:38-51::compute_colum_count`;
  `PartPlate.cpp:55,3914-3921,3952-3962,4838-4850`: grid origin/stride.
- `PartPlate.cpp:3179-3187::set_shape`, `OrcaSlicer.cpp:5710-5724`:
  global bed shape versus global instance print-volume containment.
- `PartPlate.cpp:2516`, `Print.cpp:2636-2638::export_gcode`,
  `GCode.hpp:218::set_gcode_offset`, `GCodeWriter.cpp:608,789,944-990`:
  subtract plate origin when formatting motion, not from input instances.
- `GCode.cpp:2924-2937` / `Print::translate_to_print_space`: first-layer
  template geometry is plate-local.

Destination: private selected-plate geometry in `ares-core::project::domain`,
`project_slice::state::print_volume`, and the existing neutral G-code emitter.
Upstream `PartPlate` lives under **slic3r/GUI**, but these are the plate geometry
operations actually used by the CLI. Port no GUI, filesystem, OpenGL, terminal
or viewer dependency into core; Tier 1 browser WASM remains supported.

## Non-negotiable contracts

1. Port `compute_colum_count` literally with **f32 sqrt, round, and comparison**;
   a mathematically similar f64 ceil is not the source contract.
2. Compute the selected origin before reducing the project to one plate.
   Columns are row-major; later rows have negative Y. Stride is the native
   plate size times `1 + 1/5`; plate identity remains unchanged.
3. Keep original instance transforms/global coordinates. Translate the bed
   shape for containment. The unconditional gate remains before validation;
   validation stays separate from pure processing.
4. Mirror native writer XY offset at emitted motion formatting. Do not rewrite
   arbitrary G-code text or subtract origin from user template commands.
   Internal motion coordinates and input geometry remain global. Template
   footprint conversion follows the cited native print-space boundary.
5. **Single-plate / origin-zero output is unchanged**, including diagnostics,
   configuration, tool ordering and complete G-code. No validation migration.
6. Do not apply the handed-off WIP as a patch: its gate-only sketch lacks export
   handling. Existing Ares slicing/emission is a temporary compatibility shell
   around these cited upstream boundaries, not a new Ares pipeline design.

## Included / deferred

Included: plate selection origin snapshot, geometric containment on the global
bed, native plate-local generated G-code and corresponding footprint values.
Existing bed compatibility diagnostics retain the caller-selected plate number.
Deferred: GUI arrangement/rendering, auto-arrange/bed resize/remapping, unrelated
Brim discrepancies, and any new viewer runtime or independent slicing design.

## Public verification seam

Approved seam: `slice_project_plate` with the original two-plate fixture.
Its plate 2 reference has SHA256
`752c208e5801b7b0d8aaf4d7097791c0c00e088c035050d97a3dde5f50c3920a`.
Retain genuine pre-implementation Empty RED and unchanged empty plate 1 control.
Compare the complete output using the existing strict generator-only byte
policy; do not rely solely on the transferred line-normalizing assertion.
Gate with workspace nextest, strict Clippy, fmt, WASM, reference-enabled
profile_entry, and independent current-base official/seeded controls.

## Implementation record (d450a6584)

- `Project::select_plate` stores the grid origin (`compute_colum_count`
  verbatim f32 sqrt/round, stride = bed bbox × 1.2). Gate:
  `PrintVolume::new` translates the bed polygon by the origin (global
  instances vs global shape).
- Emission offset mirrors the native writer XY offset
  (`GCode.hpp:218 set_xy_offset`, `GCodeWriter.cpp:608,789,944-990`):
  the per-object `set_origin` shift (`layers/tool_entry.rs`) and the
  initial model offset subtract the origin at the EmitState writer
  frame, so internal scaled coordinates stay global like upstream's
  `m_pos`. The exclude-object definitions (machine-generated, not user
  templates) subtract the origin at their formatting site
  (`footprint.rs klipper_body`) because they bypass the motion writer.
- Verified byte-exact against the native seeded `--slice 2` reference
  on the `-min` fixture (simple single-filament base); lint's original
  fixture additionally documents the later
  `bridge_over_infill_anchor_extruder_order` blocker (pi-arachne
  surface). Single-plate origin is (0,0) — output unchanged.
