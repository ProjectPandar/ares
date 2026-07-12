# Skirt Path Emission Spec

## Goal
Add the first Orca-compatible adhesion path stage by generating deterministic skirt loops from current layer contours and emitting them through the existing move/extrusion/speed G-code pipeline.

## Background
M15 emits movement, extrusion, and feedrate data for object perimeter and sparse infill paths. OrcaSlicer treats skirt/brim/support as print-time path families around or below the object before full G-code parity. Ares needs a small, inspectable adhesion stage before broader support/brim/bridge work.

Relevant OrcaSlicer references:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5540-5624` defines `skirt_distance`, `skirt_height`, `skirt_loops`, `skirt_speed`, and `min_skirt_length` defaults.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4206-4236` handles skirt loop assignment during G-code generation.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2892` includes skirt and brim in the total print extrusion scope.

## Requirements
- `ares-core` exposes `generate_skirts`, `LayerSkirts`, `SkirtPath`, and `SkirtOptions`.
- `SliceOptions` parses these Orca skirt options from JSON numbers or numeric strings:
  - `skirt_loops`: non-negative integer, default `1`; fractional, negative, non-finite, and non-numeric values are rejected.
  - `skirt_distance`: non-negative finite mm value, default `2`; negative, non-finite, and non-numeric values are rejected.
  - `skirt_height`: non-negative integer, default `1`; fractional, negative, non-finite, and non-numeric values are rejected.
  - `skirt_speed`: non-negative finite mm/s value, default `50`; negative, non-finite, and non-numeric values are rejected; `0` means use the external perimeter speed for emitted speed artifacts.
- Skirt path generation is deterministic and filesystem-free:
  - emits no skirt paths when `skirt_loops == 0` or `skirt_height == 0`;
  - emits skirt paths only for represented layers with `layer_id < skirt_height`;
  - preserves empty represented layers;
  - for the current simple geometry stage, emits rectangular closed loops around each layer contour bounding box expanded by `skirt_distance + loop_index * effective_line_width`.
- `PrintPathRole` gains `Skirt`; skirt paths are prepended before the existing perimeter/infill ordering, so `is_infill_first` still controls the relative perimeter/infill order after skirts.
- Extrusion and speed stages support skirt print moves:
  - skirt extrusion width uses `line_width` fallback semantics already used for current roles;
  - skirt speed uses `skirt_speed` when positive, otherwise external perimeter speed.
- `SlicingPipeline` includes a `Skirts` stage before `PrintPaths`, exposes `layer_skirts`, and reports `total_skirt_path_count`.
- `slice` and `ares slice` output include:
  - `; total_skirt_path_count = ...` header metadata;
  - per-layer `; skirt_count = ...` metadata;
  - `;SKIRT:x,y -> ...` artifact lines;
  - `;PRINT_PATH:skirt:...`, `;MOVE:...:skirt:...`, `;EXTRUSION:...:skirt:...`, `;SPEED:...:skirt:...`, and `G1 ... E... F...` commands for skirt paths.
- Existing default square-pyramid fixture emits exactly one closed skirt loop on layer 0 and no skirt loop on layer 1.
- Existing path-following command invariants remain true: closed skirt loops return to their first point, one command is emitted per `;MOVE` marker, and no standalone feedrate commands are emitted.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.

## Non-goals
- No brim generation, support generation, bridge detection, draft shields, per-object skirts, `is_infill_first` override behavior, skirt minimum extrusion length compensation, multi-extruder skirt distribution, wipe/prime tower behavior, or exact offset-polygon parity.
- No new workspace crates.
