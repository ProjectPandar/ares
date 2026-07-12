# Brim Path Emission Spec

## Goal
Add the next adhesion path stage by generating deterministic first-layer brim path artifacts from current contours and emitting them through the existing print path, move, extrusion, speed, and G-code pipeline.

## Background
M16 added skirt artifacts before ordered object print paths. OrcaSlicer also models brim behavior as first-layer adhesion geometry around or inside model islands. Ares needs a small brim milestone before broader support, bridge, and full G-code parity work.

Relevant OrcaSlicer references:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1594-1604` defines `brim_width` and `brim_type`, with `brim_width` defaulting to `0` and `brim_type` defaulting to `auto_brim`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1627-1635` defines `brim_object_gap`, defaulting to `0`.
- `OrcaSlicer/src/libslic3r/Brim.cpp:447-458` selects brim type, object gap, width, and outer/inner brim behavior.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2892` includes skirt and brim in total print extrusion scope.

## Requirements
- `ares-core` exposes `generate_brims`, `LayerBrims`, `BrimPath`, `BrimOptions`, and `BrimType`.
- `SliceOptions` parses these Orca brim options from JSON numbers or numeric strings where applicable:
  - `brim_width`: finite mm value in Orca's supported range `0..=100`, default `0`; negative, non-finite, and non-numeric values are rejected.
  - `brim_object_gap`: finite mm value in Orca's supported range `0..=2`, default `0`; negative, non-finite, and non-numeric values are rejected.
  - `brim_type`: string enum, default `auto_brim`; valid values are `auto_brim`, `brim_ears`, `painted`, `outer_only`, `inner_only`, `outer_and_inner`, and `no_brim`; non-string and unknown values are rejected.
- Option parsing changes keep `crates/ares-core/src/options.rs` under 400 LOC by moving existing numeric parsing helpers into a focused options submodule before adding brim parsing.
- Brim path generation is deterministic and filesystem-free:
  - emits no brim paths when `brim_width == 0` or `brim_type == no_brim`;
  - emits brim paths only for represented layer `0`;
  - preserves empty represented layers;
  - for the current simple geometry stage, emits rectangular closed outer brim loops around each layer contour bounding box;
  - loop spacing uses the effective brim line width already used for skirt/perimeter-style extrusion width;
  - the innermost loop is offset by `brim_object_gap + min(effective_line_width, brim_width)` and the outermost loop never exceeds `brim_object_gap + brim_width`.
- In M17, `auto_brim`, `outer_only`, and `outer_and_inner` generate the same deterministic outer-only brim scaffold. `inner_only`, `painted`, and `brim_ears` are parsed but generate no paths until their dedicated geometry milestones.
- `PrintPathRole` gains `Brim`; generated brim paths are ordered after skirts and before perimeter/infill paths, so `is_infill_first` still only controls the relative perimeter/infill order after adhesion paths.
- Extrusion and speed stages support brim print moves:
  - brim extrusion width uses the existing `line_width`/automatic width fallback semantics used for current perimeter-style paths;
  - brim speed uses the external perimeter speed until a later milestone types a dedicated Orca brim speed behavior.
- `SlicingPipeline` includes a `Brims` stage after `Skirts` and before `PrintPaths`, exposes `layer_brims`, and reports `total_brim_path_count`.
- `slice` and `ares slice` output include:
  - `; total_brim_path_count = ...` header metadata;
  - per-layer `; brim_count = ...` metadata;
  - `;BRIM:x,y -> ...` artifact lines;
  - `;PRINT_PATH:brim:...`, `;MOVE:...:brim:...`, `;EXTRUSION:...:brim:...`, `;SPEED:...:brim:...`, and `G1 ... E... F...` commands for brim paths.
- Existing default square-pyramid fixture emits no brim paths because `brim_width` defaults to `0`.
- A square-pyramid fixture with `brim_width = 1.2`, `brim_type = outer_only`, and `brim_object_gap = 0.2` emits deterministic closed brim loops on layer `0` and no brim loops on layer `1`.
- Existing path-following command invariants remain true: closed brim loops return to their first point, one command is emitted per `;MOVE` marker, and no standalone feedrate commands are emitted.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.

## Non-goals
- No exact Orca offset-polygon parity, inner brim geometry, mouse ears, painted brim geometry, automatic brim-width analysis, support brim, prime/wipe tower brim, bridge detection, or support generation.
- No new workspace crates.
