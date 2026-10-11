# Web ToolPosition actual speed profile (S2)

Upstream revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`. This slice is
a source-cited rewrite of the GCodeViewer actual speed profile. Upstream
compiles it into shipping builds: `libslic3r/Technologies.hpp:52` has
`#define ENABLE_ACTUAL_SPEED_DEBUG 1`, so the name does not mean debug-only.
`web-actual-speed.md` deferred it; the per-vertex `actual_feedrate` it reads
is already merged and parity-checked.

## Upstream boundary and destinations

| Upstream | Destination |
|---|---|
| `GCodeViewer.cpp:1681-1753` `update_sequential_view_current`, the `ENABLE_ACTUAL_SPEED_DEBUG` block: profile data of the current G-code line | `crates/ares-vgcode/src/sequential_view/actual_speed_profile.rs`, `SequentialView::update_actual_speed` |
| `GCodeViewer.hpp:47-59` `SequentialView::ActualSpeedImguiWidget` data (`y_range`, `levels`, `Item{pos,speed,internal}`); `Marker::set_actual_speed_*` (`:93-101`) | `ActualSpeedProfile` / `ActualSpeedItem`, stored in `SequentialView` |
| `libvgcode/src/ViewerImpl.cpp:1858-1862` `m_actual_speed_range` | Already ported in `crates/ares-vgcode/src/viewer/colors.rs`; reused |
| `GCodeViewer.cpp:470-478, 518-614`: Show/Hide button, "Actual speed profile" label, external window with plot and Position/Speed table | `web/app/src/preview/ActualSpeedProfile.tsx`, used from `ToolPosition.tsx` |
| `GCodeViewer.cpp:152-230` `ActualSpeedImguiWidget::plot` | SVG in `ActualSpeedProfile.tsx` (no Three.js, no canvas) |

The WASM binding is thin. `VertexBuffers.actualSpeedProfile()` in
`crates/ares-wasm/src/moves.rs` runs `update_actual_speed` after
`setMovesRange` and `vertexColors`. That is upstream's order: the
moves slider sets the visible range, then the block runs. It returns the
stored profile as JSON
`{yRange, levels: [{value, color: [r, g, b, a]}], data: [{pos, speed, internal}]}`.
The f32 values are widened exactly to f64, as `colorRange` does. The spec
proposed a Float32Array. JSON keeps the upstream struct shape, and a profile
has only a few rows. TS does not compute the profile; it only draws and
formats with `printf`.

## Rules ported (`GCodeViewer.cpp:1682-1752`)

- **Trigger:** `enabled_range[1] != visible_range[1]`, and the current vertex is an
  extrusion, travel, wipe or Seam. Otherwise the previous profile stays
  (stale data is kept, not cleared).
- **Group:** `start_id` walks back to the first vertex with a different
  `gcode_id`, or to 0, and is included. `end_id` walks forward to the first
  differing vertex, or to the last vertex, and is exclusive. So a group at the
  buffer end drops its last vertex. `end_id` is decremented when vertex
  `end_id - 1` is a Seam.
- **Lengths:** f32 `(a - b).norm()` of the positions, accumulated in f32. A point
  is kept when it is the first one or when `len > EPSILON`. `EPSILON` is
  `static constexpr double 1e-4` (`libslic3r.h:52`), so the f32 length is
  compared as f64. `internal` is `times[0] == 0.0f`, the Normal mode index.
- **Levels:** `ColorRange::get_values()` of the ActualSpeed range, each with
  `libvgcode::convert(get_color_at(value))` (`c * (1.0f / 255.0f)`) and alpha
  0.5. `y_range` is `get_range()`.
- **Compression:** `sameSpeed` compares `(int)roundf(speed * 10)`. The first point
  is kept; on a speed change the previous point is pushed unless
  `compressed.back()` already has its speed, then the point itself. The last
  point of a run is kept. Finally the last raw point is appended if its `pos`
  differs from `compressed.back().pos`.

## Web UI (`GCodeViewer.cpp:518-614`, `:152-230`)

- Below the properties table: a Show/Hide button and the "Actual speed
  profile" label. The button is disabled unless the window vertex is an
  extrusion, travel or wipe (`actual_speed_exist`). Upstream keeps
  `table_shown` as a process static; the web keeps it for the preview session,
  closed by default.
- **External window:** anchored left of ToolPosition at the canvas bottom
  (`set_next_window_pos(x - 5, canvas_h, pivot 1, 1)`). At ≤640px it stacks
  above ToolPosition at full width. Its height is clamped to the canvas, with
  the table scrolling under a sticky header (upstream `ScrollY` plus
  `TableSetupScrollFreeze`).
- **Plot:** level lines in the left 10px strip (10%..90% of it, 3px,
  `levels` colors); vertical grid lines per point except the last (teal at
  alpha 0.5 when internal, gray 0.5 otherwise) plus the right edge; profile
  segments (2px, light gray, the hovered one teal). The hover segment is `n`
  with `tp1.x < t < tp2.x`, where `t = clamp((mx - min - 10) / (w - 10), 0, 0.9999)`.
  Nothing is drawn with `size_x <= 0` or fewer than 2 points.
- **Table:** "Position (mm)" `%.3f`, "Speed (mm/s)" `%.1f`. Row background
  is teal 0.15 when internal, else gray 0.25. Rows `n` and `n + 1` of a
  hovered segment use the ORCA color.

## Deferred

- Window width from ImGui text metrics (`:536-551`). The web uses the 16:9
  aspect of the 135px plot height in CSS.
- Highlighting from table row hover (upstream highlights only from the plot).
- Stealth time mode.

## Ways it could fail (written before code)

1. The group bounds are off by one. Upstream's start includes the previous
   line's last vertex, and the end is exclusive, also at the buffer end.
2. The Seam `--end_id` is applied to the wrong vertex.
3. Lengths are in f64, the first point is skipped, or `EPSILON` is compared
   in f32.
4. Compression uses ties-to-even instead of `roundf`, or gets the
   `same_as_next` boundary, the `compressed.back()` guard or the final `pos`
   check wrong.
5. `internal` comes from another time mode, or uses `<=`.
6. The y range or levels come from another range, or ignore travel/wipe
   visibility.
7. The trigger uses the full range instead of the enabled range, or stale data
   is cleared.
8. TS `%.3f`/`%.1f` rounding of f32 values is wrong.
9. Hover is wrong: the offset strip, the 0.9999 clamp, or rows `n`/`n + 1`.
10. The component lands in the initial chunk (gzip budget 125000).

## Evidence

- **Native oracle (runtime only, never committed):**
  `tools/chain-probe/actual-speed-profile.nix` overrides the pinned
  `procgcode.nix` with `actual-speed-profile.patch` (`--dump-actual-speed-profile`)
  and `actual-speed-profile-oracle.hpp`. The oracle runs the real
  `LibVGCodeWrapper` `convert`, then the block extracted by
  `extract-actual-speed-profile.mjs`, which is checked with `--check` against
  `actual-speed-profile/extraction.json`. The adaptations are listed there and
  in `kernel.diff`. The ActualSpeed range is a real `libvgcode::ColorRange`,
  filled by the `ViewerImpl.cpp:1858-1862` rule for travels/wipes hidden and
  shown. The trigger is left to the consumer, so the dump has one profile (or
  "no update") per vertex id. `actual-speed-profile-fixtures.mjs` projects it to
  TSV outside the repository.
- **Rust:** `crates/ares-vgcode/tests/actual_speed_profile.rs` loads the Native
  vertices into `ViewerImpl`. For both visibilities it seeks every enabled
  vertex in reverse and then forward order, and compares the stored profile
  bitwise after each `update_actual_speed`. `ARES_REQUIRE_ORCA=1` without
  `ARES_ACTUAL_SPEED_PROFILE_BIN` fails. Fixture:
  `tests/parity/actual-speed-profile/profile.gcode`.
- **Web:** the preview E2E opens the profile at the marker, compares the
  table with the Native oracle run on the exported G-code, checks the disabled
  button and the plot hover, and writes `target/web-e2e/actual-speed-profile/`.

## Gates (ruled)

The exclusive full and workspace runs are waived, because only ares-wasm
depends on ares-vgcode. Required gates:

- fmt and clippy `-D warnings`;
- wasm32 check;
- `nextest -p ares-vgcode -p ares-wasm` with the Native producer;
- web build, lint and typecheck;
- browser base/cand windows and gzip.

RED and GREEN merge together.
