# Preview Acceleration / Jerk / Pressure Advance — source-cited rewrite spec

Implementation proceeds RED → GREEN, with the rulings below. The ares-core
processor part is its own RED/GREEN pair, verified vertex-by-vertex
bit-exactly against the procgcode oracle, and must prove zero G-code output
change (full v2 references identical on both trees).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths below are relative to `OrcaSlicer/` unless prefixed with a crate/web path.

## Proposed slice

Port the three ORCA per-vertex preview attributes that
`GCodeProcessor::store_move_vertex` records and libvgcode already consumes:
`MoveVertex::pressure_advance`, `acceleration` and `jerk`. Today
`crates/ares-vgcode/src/convert.rs:64-87` hard-codes them to `0.0` ("deferred by
M853"), so the already-ported Rust `ColorRange`s and per-vertex colors for
`EViewType::{Acceleration, Jerk, PressureAdvance}` are unreachable and the
property window omits three upstream rows. This is a source rewrite of the
processor recording, the wrapper pass-through and the GUI items that read
them. It is not an Ares-owned estimator or a new view design.

The marker-time spec deferred exactly this: "Acceleration/Jerk/Pressure
Advance recording/display". The WASM move transport already has slots for
all three (`crates/ares-wasm/src/moves.rs:213-214`, stride-16 `PA,acceleration,jerk`).

## Source → destination

| Source owner | Included behavior | Rust / thin adapter destination |
| --- | --- | --- |
| `libslic3r/GCode/GCodeProcessor.hpp:194-217` `MoveVertex::{pressure_advance, acceleration, jerk}` | Three f32 fields per move | `crates/ares-core/src/gcode_processor.rs::MoveVertex` (+12 B/move); `Default` dummy move keeps `0.0` |
| `GCodeProcessor.cpp:5630-5664,5676-5706` `store_move_vertex` | `move_acceleration` by move type (Travel → travel, Retract/Unretract → retract, else print acceleration, Normal mode); `move_jerk` axis selection from per-axis jerks (E-only, Z-only, XY min, X, Y, Z, fallback XY min); `use_jd_jerk` = Marlin2 flavor and Normal JD > 0; recorded with `m_pressure_advance` | `crates/ares-core/src/project_slice/gcode_emit/processor/moves.rs::store` reading the Normal-mode machine state; a focused `moves/dynamics.rs` if `moves.rs` (369 LOC) would exceed 400 |
| `GCodeProcessor.cpp:5771-5812` `get_axis_max_jerk_with_jd`, `get_axis_max_jerk`; `:5842-5885` `get_{retract_,,travel_}acceleration` | Preview getters: JD jerk `sqrt(jd * acc * 2.5f)` in f32 with axis-max clamp and `acc <= 0` fallbacks; classic `machine_max_jerk_{x,y,z,e}`; raw machine accelerations (setter-clamped) | Same module, f32 arithmetic over the existing processor machine limits (`processor/motion/limits.rs` state is the data owner, not duplicated) |
| `GCodeProcessor.cpp:1832-1833,2899-2901,5031-5060` `process_M900`, `process_M572`, `process_SET_PRESSURE_ADVANCE` | M900 `K` / M572 `S` keep the previous value when absent, then `max(0, v)` (all flavors via the command map); `SET_PRESSURE_ADVANCE` only for Klipper flavor with case-insensitive command dispatch, then the case-sensitive raw-line regex `SET_PRESSURE_ADVANCE\s+(?:.*\s+)?ADVANCE\s*=\s*([\d.]+)`: no match keeps the value, a match `stof` cannot parse (e.g. `.`) sets `0` | Existing processor command dispatch (`processor/moves.rs` / `motion/config_commands.rs` neighbourhood); no new parser framework |
| `GCodeProcessor.cpp:5962-5997` actual-speed vertex clone | Inserted internal vertices clone the source move, so they inherit all three fields | Already satisfied by `moves/actual_speed.rs:69-80` (`..*curr`); covered by a control, unchanged |
| `slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:215-260` `convert` | `curr.pressure_advance/acceleration/jerk` → `PathVertex` for every emitted vertex | `crates/ares-vgcode/src/convert.rs::vertex` (replace the three zeros; update the M853 deferral comment) |
| `libvgcode/src/ViewerImpl.cpp:1472,1506-1518,1853-1866` | Range updates and colors (PA only for extrusions with `>= 0`; Accel/Jerk over visible travel/wipe/extrusion) | Already ported in `crates/ares-vgcode/src/viewer/colors.rs`; unchanged, now fed real data |
| `slic3r/GUI/GCodeViewer.cpp:66-102,1077-1093` | View-type labels "Acceleration", "Jerk", "Pressure Advance" in `view_type_items` order | `crates/ares-wasm/src/colors.rs::VIEW_TYPES` + `web/app/src/preview/LegendRange.tsx::VIEW_TYPE_ITEMS` (Acceleration, Jerk after Speed; Pressure Advance after Temperature; ActualSpeed/ActualVolumetricFlowRate/Summary still absent) |
| `GCodeViewer.cpp:3696-3709,3947-3982` | Legend titles and `append_range` decimals 0 / 1 / 3; Acceleration and Jerk add the Travel option row exactly like Speed | `web/app/src/preview/Legend.tsx` (lazy) |
| `GCodeViewer.cpp:404-412` | Header detail `Acceleration: %.0f`, `Jerk: %.1f`, `PA: %.4f` | `web/app/src/preview/ToolPosition.tsx::detail` |
| `GCodeViewer.cpp:438-451` | Property rows `Acceleration` `%.0f mm/s²` and `Jerk` `%.1f mm/s` after Speed; `Pressure Advance` `%.4f` after Temperature, before Time | `ToolPosition.tsx` rows (lazy chunk) |

## Behavior to preserve exactly

- All three values are recorded for **every** vertex type (including travel,
  retract, options, seams and custom markers), in Normal mode, at
  `store_move_vertex` time — i.e. after any preceding M204/M205/M201/M566/
  SET_VELOCITY_LIMIT/M900/M572 on earlier lines and before the move's own
  block is planned.
- Acceleration uses the setter-clamped machine value (`set_*_acceleration`
  clamps against `max_*`), not the planner's per-block effective value.
  The move jerk for JD mode is computed with **that move's** acceleration;
  classic mode ignores acceleration.
- Arithmetic is f32 as upstream (`std::min` on floats, `sqrtf`); exact f32 bits
  are the parity target.
- Ranges, colors, palette, visibility rules and G-code bytes / M73 are
  unchanged; this slice only fills data that was zero.

## Rulings

1. **Indeterminate initial Pressure Advance — UB-adjacent, not ported.** `GCodeProcessor.hpp:822`
   declares `float m_pressure_advance;` and neither `reset()` nor the
   constructor (`GCodeProcessor.cpp:1764-1774`) initialises it, so official
   2.4.2 records an unspecified value on every vertex before the first
   M900/M572/SET_PRESSURE_ADVANCE (same class as the accepted
   `initialized_object_id`). Proposal: Ares records `0.0`; the native oracle
   adds `m_pressure_advance = 0.0f` in `reset()` only, labelled an
   *initialized-native* oracle; pre-command PA values from official are not
   compared and are documented as a known difference.
   Ruling: accepted. Reading the uninitialized member is upstream UB and is
   not ported; Ares records `0.0`. Classified UB-adjacent, the same class as
   `initialized_object_id`.
2. **f64 machine state.** Ares' processor motion limits are held as f64.
   Proposal: round the stored machine values to f32 exactly where upstream
   stores `float` members and compute the preview getters in f32, with any
   bit mismatch treated as a failure (no tolerance).
   Ruling: accepted. f32 storage and arithmetic at upstream `float`
   members; bit-exact, no tolerance.

## RED-first plan and acceptance evidence

1. **Oracle.** New `tools/chain-probe/procgcode-dynamics.{nix,patch}` on top of
   the existing procgcode probe: `--dump-move-dynamics <csv>` writes
   `gcode_id,type,internal_only,pressure_advance,acceleration,jerk` with
   `%.9g` for all `result.moves`, plus the ruling-1 initialisation.
   Source/adapted diffs and hashes are committed.
2. **Processor fixtures** (`tests/parity/motion-attributes/*.gcode` + overrides),
   each processed by Ares and the oracle and compared by gcodeId/occurrence:
   - Marlin2 with JD > 0: M204 P/T/R and S changes above and below
     `machine_max_acceleration_*`, XY / X / Y / Z-only / E-only / XYZ /
     zero-length moves, travel, retract and unretract.
   - Marlin classic (JD = 0) and Marlin2 with JD = 0: M205 X/Y/Z/E jerks.
     The fixture writes `M205 X12 Y9` (both axes explicit) on purpose: Ares'
     M205 (`processor/motion/config_commands.rs`) does not yet port
     `process_M205` (`GCodeProcessor.cpp:5236-5265`), where `X` sets both the
     X and Y jerk and `J`/`S`/`T` set junction deviation / minimum rates.
     That timing divergence is a separate slice (ruled: next after this one);
     the fixture avoids it rather than masking it, and must not be read as
     coverage of X-only or `J` M205.
   - Klipper: `SET_VELOCITY_LIMIT`, `SET_PRESSURE_ADVANCE ADVANCE=…` with and
     without `EXTRUDER=`, lowercase command (dispatched, regex no-match keeps
     value), `ADVANCE=.` (→ 0); the same command under Marlin2 is ignored.
   - RepRap `M572 S…`, Marlin `M900 K…`, `M900` without K, negative K.
   RED: the Rust test fails on the current zeros. GREEN: all vertices,
   including internal actual-speed vertices, match bit-exactly.
3. **vgcode/WASM.** RED test that `convert` carries the fields and that
   `colorRange("Acceleration" | "Jerk" | "PressureAdvance")` and `viewTypes()`
   expose them, against the oracle-derived range values.
4. **Browser E2E** (lazy UI only): slice a project whose processor input
   exercises non-zero values; choose each new view type, assert legend title/
   rows (decimals 0/1/3, Travel row for Acceleration/Jerk) and ToolPosition
   rows/detail text against the oracle's vertex values for the selected
   window vertex; dark/light screenshots and a CSV artifact.
5. **Gates.** fmt, clippy `-D warnings`, `cargo nextest run --workspace`,
   CLI/WASM/app builds, typecheck, lint, full v2 seeded/official references
   (G-code bytes unchanged), full browser pair,
   initial JS gzip strictly below 125,000 B.

## Included / deferred

Included: the three `MoveVertex` fields and their recording, the three PA
commands, wrapper pass-through, the three view-type combo items, legend and
ToolPosition rows/detail.

Deferred: ActualSpeed / ActualVolumetricFlowRate view types (actual-feedrate
transport), Stealth mode values, the Summary view, the Tool view
(develop-mode only upstream), legend range property rows
(`GCodeViewer.cpp:2382-2387`, debug window), imperial units and any time or
planner changes. Existing processor motion state and Rust viewer ranges remain
the adapters around the cited upstream concepts; only the zero placeholders
and missing GUI items are replaced.
