# Web preview: legend Custom G-code overview (slice B)

Status: spec for review. It depends on slice A
(`project-plate-custom-gcodes.md`). Code starts after both are approved and
A is merged.

## Upstream boundary

The overview is the block of `GCodeViewer::render_legend`
(`GCodeViewer.cpp:3111`) between the view-type sections and the time
estimation footer (`:4477-4546`, OrcaSlicer `v2.4.2`, `8500fcdcc`). It is
drawn for every view type whenever the list is not empty. The web app holds
the editor role (it loads and slices projects), so the list is the editor
branch, `wxGetApp().is_editor() ? plater()->model().get_curr_plate_custom_gcodes().gcodes`
(`:4478-4479`), which slice A ports.

| Upstream | What | Ares destination |
|---|---|---|
| `GCodeViewer.cpp:4478-4479`, `Model.hpp:1565-1570` | The item list: the current plate's custom G-codes; the block is skipped when it is empty | slice A's `Project::curr_plate_custom_gcodes()`, carried to the viewer by the wasm slice adapter (transport below) |
| `GCodeViewer.cpp:4483-4493`, `:4505-4511` | Header `Custom G-code` / `Layer` / `Time` | the lazy legend chunk |
| `GCodeViewer.cpp:4517-4527` | Label per item: `Pause`, `Template`, `Tool Change`, `Custom`. The switch has no `ColorChange` case, so a color change falls to the default, `Unknown` (ruled: port verbatim) | ares-vgcode, exhaustive `match` on `CustomGCodeType` |
| `GCodeViewer.cpp:4515`, `GCodeViewer.hpp:290-295`, `ViewerImpl.hpp:94` | Layer zs: libvgcode `Layers::get_zs` (f32) widened to f64 | existing `ares_vgcode::Layers::get_zs` |
| `GCodeViewer.cpp:108-125` `find_close_layer_idx`; `IMSlider.hpp:24` `epsilon() = 0.0011` | Layer index of `print_z`: `lower_bound`, then the nearer neighbour within eps (a tie goes to the upper one); `-1` if none | ares-vgcode, private fn in the new module |
| `GCodeViewer.cpp:4528-4532` | Layer cell `%d` of `layer + 1` (so `-1` prints `0`) | ares-vgcode emits `layer + 1` as i32; TS prints it |
| `GCodeViewer.cpp:4534-4544`, `ViewerImpl.hpp:148` | Time cell: f32 sum of `get_layers_estimated_times()` (current time mode) over layers `0..layer`, only when `layer > 0`; text `short_time(get_time_dhms(t))` | ares-vgcode sums with `Layers::get_times(TimeMode::Normal)`; TS formats with the existing `shortTime(timeDhms(t))` port (`toolPositionFormat.ts`, `Utils.hpp:494-562`) |

New Rust module: `crates/ares-vgcode/src/viewer/custom_gcode_overview.rs`,
with `custom_gcode_overview(items: &[CustomGCodeItem], layers: &Layers) ->
Vec<CustomGCodeOverviewRow>`, where each row is `{ label: &'static str,
layer: i32, time: f32 }`. It takes the items as an argument, so it does not
care which upstream branch supplied them.

**Transport.** The wasm slice adapter already loads the `Project`. It
passes `curr_plate_custom_gcodes().gcodes` into a thin JSON getter in a new
`crates/ares-wasm/src/custom_gcode_overview.rs` (`moves.rs` is at 370
lines). The exact hook point is cited in the review once A has merged.
`LegendCustomGCode.tsx` renders the rows before `EstimationRows` and does
no computation beyond the existing time formatter.

## Included and deferred

**Included:**
- The header and one row per item, in list order, for every view type.
- The label, the layer number (including `0` for an unmatched z) and the
  Normal-mode time.
- The block's absence when the list is empty.

**Deferred:**
- **The G-code-viewer branch** (`m_custom_gcode_per_print_z`, `:4480`,
  set from `GCodeProcessorResult` at `:1340`). It applies only in viewer
  mode, and the web has none: the import input accepts `.3mf,.stl` only
  (`web/app/src/shell/TopBar.tsx:26`), and no route opens a raw `.gcode`.
- Stealth mode: the time uses `TimeMode::Normal`. Ares has no stealth
  machine yet (M853 deferred), the same rule as the estimation footer.
- ImGui column geometry (`max_len`, `max_len * 1.5`): the web uses a CSS
  grid with the same three columns.
- The adjacent `Settings` block (`:4430-4475`, viewer only) and the time
  mode button (`:4646-4670`).
- Tick emission (slice A's deferral). Until it lands, the times are those
  of an Ares G-code without M600/M601 lines. The overview is still exact
  for the G-code Ares produced, because its oracle reads those same bytes.

## Ways it could fail

- **Source list:** using the processor list or the ColorPrint ranges
  instead of the plate list, or using a plate other than the sliced one.
- **Label:** mapping ColorChange to a "Color change" text.
- **Layer match:**
  - comparing in f32 instead of widening the zs to f64;
  - using `<=` instead of `<` against eps;
  - resolving a tie to the lower layer;
  - not handling `z` above the last or below the first layer.
- **Unmatched z:** printing `-1` instead of `0`, or a nonzero time.
- **Time:** summing through layer `layer` inclusive, summing in f64,
  summing for `layer == 0`, or using the stealth times.
- **Order:** sorting by z instead of keeping list order.
- **Visibility:** showing an empty header when the list is empty, or
  showing the block only in ColorPrint.
- **Bundle:** the new code must stay in the lazy legend chunk, and the
  initial gzip must stay below 125,000 B.

## RED-first plan and evidence

1. **Native oracle, generated at runtime, never stored.** A new
   `tools/chain-probe/custom-gcode-overview.nix` reuses the S2 harness
   (real `LibVGCodeWrapper` conversion and a `libvgcode::Viewer`, as in
   `actual-speed-profile-oracle.hpp`). Its inputs are the Ares G-code bytes
   of the slice and the item list from slice A's Native probe JSON. The
   kernel header holds `:4515-4544` verbatim. For each item it writes
   `label`, `layer + 1`, the time as f32 bits and the `short_time` text.
   It is built with Nix (`--cores 4 --max-jobs 1`), pinned and gcrooted.
   The env var is `ARES_CUSTOM_GCODE_OVERVIEW_BIN`.
2. **Fixtures:** slice A's `plates` fixture, which covers every type. Its
   z values are chosen against the sliced layer zs to include:
   - an exact layer z;
   - a z within eps;
   - an equidistant tie;
   - a z below the first layer;
   - a z above the last layer;
   - a z on layer 0.

   A tick-free project gives the empty case. The oracle test asserts that
   all of these occur.
3. **Core RED** (`crates/ares-vgcode/tests/custom_gcode_overview.rs`): it
   compares the rows with the oracle (label, layer, and time f32 bits). The
   function exists and returns an empty `Vec`, so RED fails on an assertion.
   With `ARES_REQUIRE_ORCA=1` a missing producer fails.
4. **Core GREEN:** the port, plus a wasm test that round-trips the JSON.
5. **Web RED:** a preview E2E imports the `plates` fixture, slices it, and
   asserts the exact table text against the oracle rows in FeatureType and
   in one other view. It then slices the tick-free project and asserts the
   block is absent. RED fails because the block is missing. If the getter
   is part of the public worker API, `web/e2e/consumer/types.ts` gains the
   exact row shape, which is listed separately in the review.
6. **Web GREEN:** the table is implemented. The artifacts are the oracle
   JSON, the expected and actual rows, and dark, light and phone legend
   screenshots.
7. **Gates** (as in the S2 ruling; no ares-core change and no G-code
   bytes change):
   - Rust: `nextest -p ares-vgcode -p ares-wasm` with the producer, clippy
     `-D warnings`, fmt, and the wasm32 build.
   - Web: build, lint and typecheck, then browser base and candidate.
   - Bundle: gzip.
   - Screenshot diffs must stay in the known categories plus the new block.
