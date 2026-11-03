# Preview Actual Speed / Actual Flow — source-cited rewrite spec

Status: approved at `667d92a0f` (extend the existing dynamics probe; the
profile table/plot is a roadmap deferral; core RED/GREEN bit-exact per vertex;
zero G-code change proven by seeded full raw bytes on both trees). Base: `6ff32f927` (motion attributes merged).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths below are relative to `OrcaSlicer/` unless prefixed with a crate/web path.

## Why this slice

Orca's `view_type_items` (`slic3r/GUI/GCodeViewer.cpp:1077-1093`) lists, in
order: Summary, Line Type, Filament, Speed, **Actual Speed**, Acceleration,
Jerk, Layer Height, Line Width, Flow, **Actual Flow**, Layer Time, Layer Time
(log), Fan Speed, Temperature, Pressure Advance (`Tool` is commented out).
After the motion-attributes merge the web viewer offers every item except
Summary, Actual Speed and Actual Flow. Actual Speed / Actual Flow are the
planner-based views users read to see where a print really reaches its
target speed. Summary is a different legend layout and stays out of this slice.

Ares already inserts the internal actual-speed vertices
(`crates/ares-core/src/project_slice/gcode_emit/processor/moves/actual_speed.rs`,
port of `GCodeProcessor.cpp:458-547,5962-5997`) but never records
`MoveVertex::actual_feedrate`. `crates/ares-vgcode/src/convert.rs:64-71`
hard-codes `actual_feedrate: 0.0` ("deferred by M853"), and
`crates/ares-wasm/src/colors.rs` leaves both views out "because ares-core
moves do not record them yet". The ranges and colors are already ported in
`crates/ares-vgcode/src/viewer/colors.rs`; they are only fed zeros.

This is a rewrite of the upstream recording, the wrapper pass-through and the
GUI items. It does not design an Ares speed model.

## Source → destination

| Source owner | Included behavior | Rust / thin adapter destination |
| --- | --- | --- |
| `libslic3r/GCode/GCodeProcessor.hpp:194,217` `MoveVertex::actual_feedrate`, `actual_volumetric_rate()` | One f32 field per move (mm/s) | `crates/ares-core/src/gcode_processor.rs::MoveVertex` (+4 B/move); dummy move `0.0` |
| `GCodeProcessor.hpp:425-445` `FeedrateProfile`, `Trapezoid::{cruise_feedrate, deceleration_distance, is_cruise_only}` | f32 entry/exit and cruise feedrate of each planned block | `processor/motion/planner/trapezoid.rs::Trapezoid` gains `cruise_feedrate`, `entry`, `exit` (already computed there; not stored today) |
| `GCodeProcessor.cpp:480-497` (Normal mode only) | When `prev` is not Extrude/Travel/Wipe and of a different type, `prev.actual_feedrate = block.feedrate_profile.entry` | `moves/actual_speed.rs::apply` |
| `GCodeProcessor.cpp:499-558` | Each inserted acceleration-end / deceleration-start vertex carries `actual_feedrate = trapezoid.cruise_feedrate` | `actual_speed.rs::actual_vertex` |
| `GCodeProcessor.cpp:560-572,5986-5997` | The block's own move gets `(is_cruise_only \|\| !has_deceleration) ? cruise_feedrate : feedrate_profile.exit`; if the move right after it is a `Seam`, the seam gets the same value | `actual_speed.rs::apply` |
| `slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:215-260` `convert` | Phantom path-start vertex takes **`prev.actual_feedrate`**; normal vertices take `curr.actual_feedrate` | `crates/ares-vgcode/src/convert.rs` (drop the M853 deferral comment) |
| `libvgcode/src/ViewerImpl.cpp:1495,1526,1845-1862` | Actual-speed range over visible travel/wipe/extrusion without binning; actual-flow range over extrusions with `round_to_bin`, Custom gated by its role visibility | Already ported (`viewer/colors.rs`, `viewer/ranges.rs`); unchanged |
| `GCodeViewer.cpp:66-101,1077-1093` | Labels "Actual Speed" after Speed and "Actual Flow" after Flow | `crates/ares-wasm/src/colors.rs::VIEW_TYPES`, `web/app/src/preview/LegendRange.tsx::VIEW_TYPE_ITEMS` |
| `GCodeViewer.cpp:3930-3946,3986` | Actual Speed: `append_range` decimals 0 plus the Travel option row (as Speed); Actual Flow: `append_range` decimals 2, no option rows | `web/app/src/preview/Legend.tsx` (`TRAVEL_ROW_VIEWS` gains `ActualSpeed`) |

Upstream has no ToolPosition detail text or property row for actual speed
(`GCodeViewer.cpp:360-455` `default:` → empty detail). Ares matches that.

## Behavior to preserve exactly

- Only the Normal time machine records actual speed. Stealth is not touched.
- f32 arithmetic as upstream; exact f32 bits are the parity target, with no
  tolerance.
- The vertex count and order are unchanged; the inserted vertices already
  match the oracle.
- The `prev.actual_feedrate = entry` write happens while blocks are walked in
  order. A later block can overwrite it; this slice keeps upstream's order.
- G-code bytes, M73 and estimated times are unchanged. This slice only adds a
  preview field.

## Included / deferred

Included: the field, its recording, the vgcode pass-through, the two view
types, legend ranges and the Actual Speed Travel row.

Deferred to their own slices:
- The ToolPosition "actual speed profile" table and plot
  (`GCodeViewer.cpp:152-200,519-600,1686-1745`, `ActualSpeedImguiWidget`),
  now `web-actual-speed-profile.md`.
- The Summary view.
- The `Tool` view (commented out upstream).
- Stealth time mode.
- `process_M205` (located low-priority difference, roadmap).

## RED-first plan and acceptance evidence

1. **Oracle.** Extend `tools/chain-probe/procgcode-dynamics.patch` so
   `--dump-move-dynamics` appends an `actual_feedrate` column (`%.9g`, all
   `result.moves`). This is a new store path; the old one stays for history.
   The build is a multi-core heavy job and runs only in a coordinated window.
   Diffs and hashes are committed.
2. **Processor RED/GREEN** (`crates/ares-core/tests/processed_motion_attributes.rs`:
   a second per-case assertion over `<case>.actual-speed.rle`, sharing the
   same oracle run and fixtures, plus a new `marlin2_option_moves` case with
   `;PAUSE_PRINT` / `;CUSTOM_GCODE` option moves before travel). Every fixture
   has 50 Seam vertices after extrusions, and Retract/Unretract/dummy-Noop
   predecessors; the RED adds the field as a zero placeholder so the failure
   is an assertion, not a compile error.
   - RED: an assertion failure on the current zeros.
   - GREEN: every vertex, internal ones included, matches the oracle
     bit-exactly by index with `gcode_id`/type checks.
3. **vgcode/WASM RED/GREEN.** `convert` carries `actual_feedrate`, the phantom
   vertex takes `prev`'s value, and `viewTypes()` / `colorRange("ActualSpeed" |
   "ActualVolumetricFlowRate")` match ranges derived from the oracle.
4. **Browser E2E (lazy UI only).**
   - Extend `web/e2e/colorRanges.ts`: the two views, legend title/rows
     (decimals 0/2), and the Travel row for Actual Speed checked with and
     without travels, all against oracle-derived ranges.
   - Assert that ToolPosition shows no new row.
   - Artifacts: dark/light screenshots and a CSV.
   - RED: the options are missing. GREEN: they render.
5. **Gates.** fmt, clippy `-D warnings`, `cargo nextest run --workspace`,
   CLI/WASM/app builds (`npm run build:wasm` before `npm run build`),
   typecheck, lint, the seeded full set and profile_entry groups (G-code bytes
   unchanged), the full browser run with BROWSER-START/END, and initial JS gzip
   strictly below 125,000 B.

## Open questions for review

1. Should the oracle extension replace the dynamics probe (new header, E2E
   updated in the RED commit) or be a separate `procgcode-actual-speed` probe?
   Proposal: extend the dynamics probe. It is one processor pass and one CSV,
   and the motion E2E keeps its columns by name.
2. Is the profile table/plot (deferred above) wanted as the immediate
   follow-up, or should the next web item come from elsewhere in the roadmap?

## Located planner divergence (ruled: fix in-slice, option A)

`marlin2_junction_deviation` is the only case that is not bit-exact after the
core GREEN (`0390a77ad`). It has 190 differing vertices. Every other case and
every PA/acceleration/jerk bit matches. The cause is the planner copy of the
JD jerk, `processor/motion/limits.rs::MotionState::effective_jerk`, compared
with upstream `GCodeProcessor::get_axis_max_jerk_with_jd`
(`GCodeProcessor.cpp:5774-5791`):

1. **f64 instead of f32.** Upstream computes `std::sqrt(jd * effective_acc * 2.5f)`
   in float; Ares computes it in f64. This gives 1-ulp junction/safe speeds
   (`1.80277562` vs `1.80277574`).
2. **`acceleration <= 0` fallback.** Upstream uses the axis maximum
   acceleration when the current acceleration is 0 (`M204 S0`, `:5781-5786`).
   Ares takes `min(0, max) = 0`, so the junction jerk becomes 0. The
   preview-attribute port (`moves/dynamics.rs`) already does this correctly.

With both fixes applied (`target/actual-speed/planner-jd-jerk.patch`, not
committed), all 5 cases are bit-exact. The planner feeds time estimation, so
the fix could change M73/estimated-time bytes for JD profiles. That conflicts
with this slice's zero-G-code-change condition, so it needs a ruling:
fix it in-slice as a separate RED/GREEN pair with seeded evidence, or flag it
for the owner of the planner.

Ruling A applied:

- **RED `dafca192b`.** `processed_motion_attributes.rs` asserts the per-vertex
  move time bits against native `time0` (`--dump-moves`, column 16). The four
  control cases are bit-exact; `marlin2_junction_deviation` fails.
- **GREEN `dd3950d14`.** `effective_jerk` ports `GCodeProcessor.cpp:5771-5791`
  line by line in f32. All 5 cases are bit-exact on time, motion and
  `actual_feedrate`. The `actual_feedrate` comparator is unchanged.
- **`0f514e6b2`.** The pre-existing unit test
  `envelope::tests::junction_deviation_only_affects_marlin_firmware_not_klipper`
  hard-coded the replaced f64 formula (`3.535533866420067`). It now expects
  the upstream f32 value (`3.535533905029297`).

## G-code byte changes (seeded full set, rebased on `5cde90c23`)

The seeded full set (control-set-v2, fresh caches, seeded native
`1k0p5nlb…-orca-slicer-seed-replay-2.4.2`) ran on main `5cde90c23` and on
branch `8703fe7d6`; both passed 597/597. For every case label, the multiset of
`ares.gcode` sha256s is identical between main and branch (517/517 case
manifests, 13 `DIVERGENT` generator-only manifests on each side).

**Per-case byte changes: none.** The JD port changes no seeded output, so the
native-match proof for the planner fix is the per-vertex time assertion above. Before the fix it mismatched native `time0`
on `marlin2_junction_deviation` (`target/actual-speed/planner-red.log`); after
the fix it is bit-exact (`planner-green.log`). profile_entry passed 26/26 on
both official and seeded.

Evidence: `${TMP_ROOT}/ares-actual-speed-rebased-8703fe7d6/` (`summary.txt`,
`seeded-{main,branch}/run.log`, `seeded-bytediff.json`, produced by
`${TMP_ROOT}/ares-actual-speed-bytediff.mjs`).
