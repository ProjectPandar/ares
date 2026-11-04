# M853: GCodeProcessor move vertices → libvgcode input data

Spec and plan. Rewrite gate: `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.
Upstream pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Consumer: ARD-0106 web Preview (toolpath view, layer slider, feature legend).

## Goal

Port upstream move recording, `GCodeProcessorResult::moves`, into the existing
Ares `GCodeProcessor` pass, and expose it as a public ares-core result. Then
port `LibVGCodeWrapper.cpp convert()` so the result maps onto the existing
`ares-vgcode::GCodeInputData` / `PathVertex` data. `ares-wasm` gets only a
thin typed-buffer binding over these public APIs.

This slice is a rewrite of upstream data structures and functions. It does not
add an Ares preview pipeline.

## Upstream source boundary

| Upstream | Concept | Ares destination |
| --- | --- | --- |
| `libslic3r/GCode/GCodeProcessor.hpp:30-44` `EMoveType` | move classification | `ares_core::gcode_processor::EMoveType` (public) |
| `GCodeProcessor.hpp:155-229` `GCodeProcessorResult`, `:184-218` `MoveVertex` | per-move record, `moves` vector | `ares_core::GCodeProcessorResult { moves: Vec<MoveVertex>, spiral_vase_mode }` |
| `GCodeProcessor.cpp:5630-5718` `store_move_vertex` | push a `MoveVertex` from processor state (`m_last_line_id` rule, position + extruder offset, `std::max(1, m_layer_id) - 1`) | `processor/moves.rs` `MoveRecorder::store` |
| `GCodeProcessor.cpp:3834-3850` `process_G1` `move_type` lambda | Wipe / Retract / Unretract / Travel / Extrude / Noop | same rule, `moves.rs` |
| `GCodeProcessor.cpp:3880-3925` `process_G1` | `m_travel_dist`, `m_mm3_per_mm` (filament area × ΔE / Δxyz), height/width from forced tags with `DEFAULT_TOOLPATH_HEIGHT` fallback | `moves.rs`, fed by the existing `MotionState` positions |
| `GCodeProcessor.cpp:3113-3330` `process_tags` | `;TYPE:` → `set_extrusion_role` (`ExtrusionEntity.cpp:611 string_to_role`), `;WIDTH:`, `;HEIGHT:`, `;WIPE_START/END`, `;LAYER_CHANGE` → `++m_layer_id`, color change / pause / custom tags | `moves.rs` tag handling, alongside the existing `estimate.rs` tag scan |
| `GCodeProcessor.cpp:5004/5081` `process_M104/M109`, `:5017/5062` `process_M106/M107`, `:5439-5520` `process_T` | temperature, fan speed, extruder id (`Tool_change` vertex) | `moves.rs` |
| `GCodeProcessor.cpp:4548` `process_G2_G3` | arc segments recorded as moves | reuse the existing arc subdivision in `motion.rs` / `arc_accounting.rs` |
| `TimeMachine::calculate_time` `GCodeProcessor.cpp:419-472` | `moves[block.move_id].time[mode] = block_time` | the existing `Estimate` blocks gain a `move_id`; normal-mode time is written back |
| `GCodeProcessor.cpp:2685-2696` `finalize` | wipe moves get `Wipe_Width` / `Wipe_Height` (0.05, `:113-114`) | `MoveRecorder::finish` |
| `GCodeProcessor.cpp:2142` `apply_config` | `spiral_vase_mode` | from `ProjectSettings` |
| `slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:191-268` `convert(GCodeProcessorResult, ...)` | moves → `libvgcode::GCodeInputData` (prev-position duplication for extrusion starts, tool/color palettes) | `ares-vgcode::convert::from_processor_result` (new module; `ares-vgcode` gains an `ares-core` dependency) |
| `LibVGCodeWrapper.cpp:70-145` role / move-type `convert` | `ExtrusionRole` → `EGCodeExtrusionRole`, `EMoveType` → `libvgcode::EMoveType` | same module |
| `libvgcode/include/GCodeInputData.hpp`, `PathVertex.hpp` | target data | existing `ares-vgcode::{GCodeInputData, PathVertex}` |

Upstream runs the processor on the emitted G-code (`GCode.cpp:2082` `initialize`,
`:6262` `process_buffer`, `:2174` `finalize(true)`). Ares already runs the
equivalent pass in `project_slice/gcode_emit/processor.rs::process`, so moves are
recorded in that same pass, over the same lines and the same motion state.

## Included

- `MoveVertex` fields: `gcode_id`, `type`, `extrusion_role`, `extruder_id`,
  `cp_color_id`, `position` (with extruder offset; plate offset is 0 for a
  single-plate export), `delta_extruder`, `feedrate`, `width`, `height`,
  `mm3_per_mm`, `travel_dist`, `fan_speed`, `temperature`,
  `time[Normal]`, `layer_id`, `internal_only`.
- Move types: Retract, Unretract, Seam, Tool_change, Color_change, Pause_Print,
  Custom_GCode, Travel, Wipe, Extrude.
- Actual-speed split vertex positions/interpolated attributes from
  `TimeMachine::calculate_time` and `GCodeProcessor::calculate_time`
  (`:458–547`, `:5962–5997`), including internal_only and zero split time.
  Preserve Eigen Vec3f reduction order x²+(y²+z²) for endpoint exclusion.
- `spiral_vase_mode`.
- A public entry point that returns the G-code bytes and the result:
  `slice_project_plate_processed(...) -> ProcessedGCode { gcode, result }`.
  `slice_project_plate` becomes `.gcode` of that call. The G-code bytes stay
  identical.
- `ares-vgcode` conversion, as in `LibVGCodeWrapper::convert`.
- `ares-wasm`: `sliceProjectPlate` returns a `SliceOutput` object exposing
  `gcode`, plus `takeVertices()`, which returns typed buffers in a fixed
  documented layout built from `ares-vgcode::PathVertex`: `Float32Array`
  positions/width/height/feedrate/fan/temperature/time, and `Uint8Array`
  role/type/extruder/color. JSON is used only for palettes and the layer-z
  list.

## Deferred (adjacent upstream behavior)

- The `actual_feedrate` value itself is deferred; split vertices are included
  and vertex counts are compared after upstream splitting.
- Stealth time mode (`time[1]`), `layer_duration`, `pressure_advance`,
  `acceleration` and `jerk` visualization fields. They stay at `0.0`.
- `object_label_id` and `print_z` are not exposed by this first move record.
  Seam vertices are included through the existing seam detector.
- Producer detection for foreign G-code, and `Plater::load_gcode` (previewing
  external `.gcode`). These need a config-from-G-code loader.
- Wipe tower / `m_wipe_tower` per-move flags and used-filament caches. The
  existing filament stats stay where they are.
- libvgcode `ViewerImpl` rendering, color ranges, and option markers. The web
  app renders the vertex buffers with Three.js using libvgcode role colors
  (`ViewerImpl.cpp:283 DEFAULT_EXTRUSION_ROLES_COLORS`).

## Existing Ares scaffolding

- `processor.rs` / `estimate.rs` stay the time estimator. The move recorder is
  a sibling consumer of the same per-line loop. Estimator outputs (`M73`,
  header times) must stay byte-identical.
- `seam.rs` already tracks `;TYPE:` for seam detection. Role parsing is shared,
  not duplicated.
- `ares-vgcode` types are already ported. This slice adds only the
  `convert` module.

## Plan

1. `processor/moves.rs`: `MoveVertex`, `EMoveType`, `MoveRecorder` (tags, M104/
   M106/M107/M109/T, `store`). Keep the file under 400 LOC; split the tags into
   `moves/tags.rs` if needed.
2. Thread `MoveRecorder` through `Estimate::from_lines`, with
   `block.move_id → time[Normal]`. Arcs go through the existing segment loop.
3. `process()` returns `(Vec<u8>, GCodeProcessorResult)`. Add a public
   `ProcessedGCode` and `slice_project_plate_processed`. Re-export the types
   from `lib.rs`.
4. `ares-vgcode/src/convert.rs`: `from_processor_result(&GCodeProcessorResult,
   tool_colors, color_print_colors) -> GCodeInputData`.
5. `ares-wasm`: `SliceOutput` with typed buffers. In the web library, the
   worker transfers buffers and `types.ts` gains `Toolpaths`.
6. Web Preview: Three.js toolpath layer (role colors, layer range slider,
   legend). TS only renders the buffers.

## Verification

- Oracle E2E: extend `tools/chain-probe/procgcode.patch` (`--process-gcode`) to
  dump `result.moves` as CSV (type, role, extruder, gcode_id, layer_id,
  position, width, height, mm3_per_mm, feedrate, fan, temp, time[0]). Feed the
  same Ares G-code to the oracle and to Ares. Assert equal vertex count and
  equal fields within f32 tolerance on `tests/parity/orca_cli_ender3.3mf`
  and `tests/ksr_fdmtest_v4`. Artifact: `target/gcode-moves-oracle/*.csv`
  plus a diff summary.
- G-code bytes unchanged: workspace nextest, official + seeded control set
  with no new FAIL.
- Web E2E: Preview canvas shows the toolpaths. The layer slider changes the
  visible layer count. Screenshot artifact in `target/web-e2e/`.
- Gates: fmt, workspace clippy with no new warnings, wasm32 build.

## First Rust slice

The first slice covers only move recording and its CLI CSV oracle adapter.
`ares-vgcode::convert`, thin WASM buffers and M854 remain separate slices.

The old official manual Slim failure is a native reference fluctuation: main
and candidate input and Ares bytes are identical, only native bytes differ.
Do not substitute its outputs into a fresh control run.

Moves oracle: Ender3 5564 moves, KSR635147 moves, zero out-of-tolerance fields.
The comparator checks integer IDs as integers rather than lossy f32. Floating
bounds: exact copied f32 fields,1ULP derived values,2ULP normal time. The
oracle is built from source8500fcdcc with the existing procgcode patch.

## Risk

- The oracle binary must be rebuilt with the dump patch, using the nix pattern
  in `tools/chain-probe/README.md`. If no prebuilt oracle exists, that build
  is the long pole.
- Memory: one `MoveVertex` (~88 B) per move. ksr_fdmtest_v4 is about 1–2 M
  moves, roughly 150 MB in WASM. Upstream holds the same. The web buffers are
  converted once and transferred, not copied.
