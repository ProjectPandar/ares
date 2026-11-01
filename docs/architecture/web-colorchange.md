# ColorChange / M600 preview — source-cited rewrite spec

Status: approved and fast-forwarded to main at `70323c9a7`.
Spec approval: `d5f91a6c0`. Full gate checkpoint: `96f91ff28`, based on
`43df7c95d`; the final stop-time unwrap removal in `70323c9a7` passed focused
M73 byte tests (3/3), fmt and workspace clippy under the approved review scope.

Implementation constraints from approval: core changes are processor parsing
and statistics only; slicing/emission behavior is out of scope. Tagged slice
fixtures use existing user G-code templates. Native cardinality inputs live
in `tests/parity/color-change/`, not `/tmp`. Removing caller
`color_print_colors` requires a breaking `!` commit and `web/README.md` notice.
The coordinator explicitly permits one data-initialization exception in
`finish/accounting.rs`: initialize the new color-volume vector to empty;
no algorithm, control-flow or G-code output change there.

Predecessor: Filament/ColorPrint `9b5db6d7a`, approved and fast-forwarded to
`main` and `pi/web-ui`. Upstream source revision:
`OrcaSlicer` `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All upstream paths below are relative to `OrcaSlicer/`.

## Objective and limits of the upstream behavior

Port the processor's ColorChange metadata and volume/time records, and
libvgcode's ColorPrint segment records, into Rust. Feed the existing lazy
Filament view with Rust-selected palettes and display records. This is a
`libslic3r` / `libvgcode` rewrite slice, not a new Ares slicing pipeline.

**ColorChange is not bare M600 detection.** The command registry in
`src/libslic3r/GCode/GCodeProcessor.cpp:1790-1843` has no M600 handler.
`process_gcode_line` (`:2881-2929`) dispatches standalone comment tags to
`process_tags`; its ColorChange branch (`:3237-3301`) creates the event.
Fixtures must exercise `;COLOR_CHANGE,T0,#RRGGBB` (or the BBL spaced tag)
and M600 on separate lines; bare M600 and inline-tag comments are negative
controls. Do not invent a color
for a firmware command lacking processor metadata.

Two important upstream limitations are explicit:

- `src/libslic3r/GCode.cpp:4203-4232` puts automatic ColorChange tag/G-code
  injection behind `#if 0`. This slice does not re-enable that emitter path
  or add a custom-height editor. Use existing user G-code template overrides
  to produce tagged fixtures through the real native and Rust slicers.
- `src/slic3r/GUI/GCodeViewer.cpp:4172-4287` computes partial-time display
  records, but `:4316-4342` comments out their table rendering. Color-range
  row counting is also commented out at `:4030-4033`. The requested web
  display exposes these source-defined records; it is not pixel parity with
  an upstream table that is disabled in this revision.

## Source → Rust destination

| Upstream owner / lines | Included behavior | Rust destination |
| --- | --- | --- |
| `GCode/GCodeProcessor.hpp:46-78, 260`; `CustomGCode.hpp:14-35` | `volumes_per_color_change`, normal-mode custom times, ordered `CustomGCode::Item` metadata | `ares-core/src/gcode_processor.rs` plus a small result-data module if needed |
| `GCode/GCodeProcessor.cpp:3237-3301` | Color token validation/default cycle, selected-filament color ids, active-filament events | `ares-core/src/project_slice/gcode_emit/processor/moves/tags.rs` and a focused ColorChange module |
| `GCode/GCodeProcessor.hpp:677-711`; `.cpp:3929` | `OptionsZCorrector` moves the pending marker and fixes event Z at the next extrusion | processor move recording; no post-hoc browser marker repair |
| `GCode/GCodeProcessor.cpp:1504-1555, 1641-1648, 5936-5949, 6031` | `UsedFilaments` color cache, nonzero flush at ColorChange and finalization, raw mm³ export | processor-owned `used_filaments` slice and `PrintEstimatedStatistics` |
| `GCode/GCodeProcessor.cpp:473, 596, 2702-2707, 2737-2750, 5917-5934, 6022` | Normal-mode custom-time cache, synchronize/flush, final record, duration/remaining pair | existing processor timing pass and result data |
| `src/libvgcode/include/ColorPrint.hpp:12-18`; `src/libvgcode/src/ViewerImpl.cpp:1017-1026, 1363-1382` | Sorted used extruders with ordered `{extruder_id, color_id, layer_id, times}` transitions | `ares-vgcode::ViewerImpl` and `ColorPrint` data |
| `src/slic3r/GUI/Plater.cpp:16923-16946`; `GUI_Preview.cpp:680-688`; `LibVGCode/LibVGCodeWrapper.cpp:48-61, 192-206` | Result tool colors plus ordered event colors, trailing gray when custom events exist, RGB conversion | `ares-vgcode` processor-result conversion; WASM forwards only |
| `src/libvgcode/src/ViewerImpl.cpp:1543-1547`; `src/libvgcode/src/Layers.cpp:15-40` | ColorPrint color-id lookup; Pause/Custom layers use dummy color | existing `ares-vgcode` colors/layers, preserving this behavior |
| `src/slic3r/GUI/GCodeViewer.cpp:3358-3388` | Per-extruder ColorChange layer brackets, epsilon lower-bound, adjacent duplicate suppression | `ares-vgcode` ColorPrint legend data |
| `src/slic3r/GUI/GCodeViewer.cpp:4172-4239, 4287` | Partial Print / ColorChange / Pause record selection and order | `ares-vgcode` legend data, retaining raw volume and time fields |
| `src/slic3r/GUI/GCodeViewer.cpp:3390-3409, 3443-3448, 4241-4285` | Text, swatches, metric conversion, duration/remaining display | lazy web legend component only |

## Non-negotiable boundaries

1. Core owns parsing, event ordering, corrected Z, color ids, raw volume
   caches and actual processor timing. Preserve upstream indexing:
   `CustomGCode::Item.extruder` is one-based; move/tool ids are zero-based.
   Neither WASM nor TS infers events from G-code strings or move positions.
2. Vgcode owns ColorPrint segment selection, layer brackets, palette assembly
   and partial-display record construction. A palette is not supplied by TS.
   Remove the remaining caller `color_print_colors` input from processed
   WASM/profile requests and worker calls, just as the preceding slice removed
   caller tool colors. No fallback to a stale caller palette.
3. WASM serializes owned Rust records. The JS package provides types and
   transport, without React or Three dependencies. React displays the selected
   records, formats text/time and converts raw mm³ to m/g using processor
   diameter/density metadata. It does not join arrays, allocate volumes to
   events, select columns, infer segment boundaries or accumulate timings.
4. Existing per-filament Model/Support/Flushed/Tower/Total behavior remains
   unchanged. A ColorChange volume is **not** a proportional share of that
   Total: only `increase_model_caches` adds to the color cache (`:1534-1540`);
   support, wipe-tower and flushing have different caches.
5. Keep the initial JS gzip gate strictly below 125,000 bytes. The previous
   slice leaves only 46 bytes headroom: all new view code/styles stay behind
   the existing lazy preview boundary. Do not weaken assertions/timeouts,
   enable retries, raise the budget or add platform-specific core behavior.

## Data semantics that must survive the port

- An active-filament tag updates the current color id, records a marker and
  `CustomGCode::Item`, arms Z correction, then processes time and volume.
  An inactive-filament tag still affects its color-id state/counter but does
  not append the active event. Port that distinction, including invalid tool
  ids, missing/invalid colors and the upstream default-color cycle.
- `volumes_per_color_change` is a chronological `Vec<f64>` of **nonzero**
  flushed model volumes. It is not dense by extruder and is not guaranteed
  one entry per tag. Ordinary tool changes do not flush this cache.
  Finalization flushes the trailing volume, including a no-change print.
- `ColorPrint` transitions are recorded only from Extrude vertices when that
  extruder's last color id changes. Record after the vertex time is added,
  exactly as `ViewerImpl::load` does. Travel/markers alone do not create
  a printed-color segment. Equal RGB values with distinct color ids must not
  be merged. Reload clears every extruder's segment vector.
- Do not equate three different upstream structures: printed-color segments,
  event-to-layer brackets, and time/volume partial-display records. A repeated
  tag at the same layer, a tag without subsequent extrusion, or a pause can
  make their cardinalities differ.
- A final synthetic ColorChange time record represents the remaining Print
  interval, not another physical color switch. `generate_partial_times`
  consumes matching custom items in order and has a no-item trailing Print
  branch. Preserve the normal-mode duration/remaining values from the
  processor rather than deriving them from wall time or TS move summation.
- Upstream indexes the volume vector directly while constructing partial
  rows (`GCodeViewer.cpp:4221, 4230`). The checked-in native probe confirms
  out-of-bounds reads for zero-before (1 volume / 2 time records), zero-between
  (2 / 3), zero-after (1 / 2), and zero-only/support-only (0 / 2); normal is
  3 / 3. See `tests/parity/color-change/oracle-cardinality.json`.
  **Approved Known difference:** preserve raw statistics and ColorPrint
  segments; if a required volume index is absent, Rust reports partial usage
  unavailable and TS displays that status instead of a guessed partial table.
  Do not zero-fill, truncate, partially return misleading usage rows, or invent
  event/volume associations. No-change prints with no time records need no
  volume reads and do not trigger this status.

## Approved OptionsZCorrector diagnostic distinction

The operator approved retaining stable pending-marker identity rather than
reproducing an upstream stale-index defect. `GCodeProcessor.cpp:3291-3296`
calls `OptionsZCorrector::set` before custom-time synchronization inserts
actual-speed vertices (`:5962-6008`). The latter remaps pending time blocks,
but not `OptionsZCorrector::m_move_id` (`GCodeProcessor.hpp:687-705`). Its
later update can therefore relocate an unrelated internal extrusion instead
of the marker, producing spurious ColorPrint transitions.

Repro at pre-M73-C checkpoint `52678d625` (rebased from `f27867a3`)
(`two-changes`; input line IDs
advance after the C-line port): unmodified
native moves relocate the internal vertex for G-code line 188 from first-layer
geometry to index 569, XYZ `(114.346001,114.346001,0.600000024)`, after the
ColorChange at line 380. The actual marker remains at Z=0.4 while the custom
item is corrected to Z=0.6. The untagged exact-move control passes.

Keep the unmodified `ARES_PROCGCODE_BIN` and its raw CSV/JSON: beta
(`345efec64`) upstreamed the pending-marker remap as
`OptionsZCorrector::on_moves_inserted` (`GCodeProcessor.hpp:1105-1116`),
so raw Native beta is the complete authority for strict vertex/ColorPrint
parity — Ares must match it bit for bit, with no separately corrected
diagnostic. Metadata, volumes and custom times remain checked against the
unmodified native processor. Generated G-code (only existing
generator/timestamp normalization) and every M73 line remain byte-compared
with the unmodified official slicer. No tolerances are enlarged and no failing
assertion is dropped.

## Known difference: tagged marker Time (approved)

For the Time row, the **stable native source-kernel** (only the approved
marker-remap patch above) is authoritative. This is actual native conversion
plus the extracted `ViewerImpl::get_estimated_time_at` kernel, not execution
of the full GL viewer. Ares's real processed-slice → vgcode → WASM output
matches all **7,347 stable vertex IDs and G-code IDs**, both duration-mode
bits and every Normal-mode elapsed bit, including after display-buffer
transfer and reverse queries. No core timing arithmetic changed.

The tagged normal input SHA256 is
`55ec96a28d3ac035d94d0923c5f3be53cb9d897963fc2b8a561b7291ef935f89`.
Raw has 7,348 vertices. Aligning by **G-code ID and occurrence**, rather than
ordinal, leaves exactly these direct differences:

| G-code ID | Raw / stable vertex | Bounded effect |
| --- | --- | --- |
| 679 | 1654 / 1450 | Stale-index relocation changes the extrusion position and duration: raw `0.1304752230644226s`, stable `0.03975646197795868s` |
| 680 | 1450 / 1451 | Following phantom inherits the changed preceding position; duration stays zero |
| 790 | 1645 / 1653 | Intended ColorChange marker is relocated correctly; duration is raw zero versus stable `0.1304752230644226s` |

Raw's additional vertex **1653 / G-code 679** is a zero-duration phantom at
the relocated extrusion's start. These five field differences and that
phantom are bounded by `GCodeProcessor.cpp:3291-3296,5962-6008`,
`OptionsZCorrector::update` (`GCodeProcessor.hpp:693-706`) and the resulting
path-start insertion (`LibVGCodeWrapper.cpp:215-238`). Every other direct
identity/duration field and the order of unaffected nonzero durations match.

Elapsed first diverges at stable vertex 1450: the native increment is
`0.03975677490234375s`, the result of adding vertex 679's
`0.03975646197795868s` duration to the prior prefix in **f32**. It is not an
exact real-valued subtraction identity. The misplaced raw duration transfers
back to marker 790 in stable; the relocated marker's prefix also reflects
its changed place in the sequence. Subsequent ordered f32 additions propagate
the offset to the complete tail (`0.039794921875s`). Both native streams'
full prefix recurrences were checked; no tail was omitted. This bounded
propagation is explicitly approved, not a general timing tolerance or an XYZ
waiver. Any difference outside these boundaries must stop validation.

Reproduction: `marker-time-stable-options.nix`, `marker-time-pair.mjs`, and
`marker-time-bounds.mjs` under `tools/chain-probe/`; genuine committed
`tests/parity/marker-time/tagged-overrides.json` input; integration test
`crates/ares-wasm/tests/marker_time_tagged.rs` now generates raw/stable Native
JSON and `stable.tsv` into a printed unique external artifact directory at runtime
(see `tests/parity/marker-time/README.md`). The former committed Native projections
and run logs were removed after live GREEN; they remain in git history, never a
fallback. Historical full records, correspondence,
first differences/counts and propagation evidence are retained under
`target/marker-time-final-9e/tagged-time/normal/` and
`target/marker-time-stable-proof/`. Diagnostics print bounded summaries, not
large assertion diffs. The initial ordinal comparison and exit 137 are
retained as failed diagnostic evidence, not silently retried to pass.

## M73 C stop-time postprocessing boundary

Operator-approved follow-up: port Normal-mode `GCodeProcessor` stop registration
(`GCodeProcessor.cpp:5715`), eligible cached-block elapsed updates (`:576-579`),
initial stop placeholder (`:1098-1101`), and next-stop / last-minute export
(`:1260-1293`). This only adds `stop_times` and M73 C output; it must not change
planner arithmetic, elapsed values, M73 P/R, slicing, or GCodeGenerator behavior.
Stealth-mode M73 D is outside the existing Normal-only estimate boundary.

The existing Schedule is a temporary representation of upstream interleaved
parsing/planning: observe how many blocks were already processed when a stop
was registered. Do not infer stop times from custom interval durations: SYNC
may already have flushed a stop's G1 ID, leaving its native stop time at zero.
Reuse the existing cache elapsed and its next entry for last-minute decisions;
never recalculate them. Preserve first-match updates for repeated G1 stop IDs.

Tests-only RED commit `4d686d5ba` (originally `057e0636`) contains three complete official G-code
oracles (SYNC, ordinary fractional last-minute, and pre-model stop), with exact
M73 and full-byte comparisons. Source-cited GREEN is `7f07fceb8`. Before
merge, run both official and seeded full reference suites including
`profile_entry`, plus all web gates. All color-change case changes must match
native. The later merger with lint's VG1 timing work must rebase and rerun.

## Existing scaffold replacement

- `moves/tags.rs::color_change` currently discards color text and only records
  marker/color ids. Replace that temporary shell with the cited processor
  event/default-palette/Z-correction behavior; do not add a second tag parser.
- `GCodeProcessorResult` currently lacks custom items and custom times;
  `PrintEstimatedStatistics` lacks the color-volume vector. Extend these
  existing public result boundaries rather than return frontend-only stats.
- `finish/accounting.rs::account_statistics` is a temporary post-export
  compatibility shell for existing filament buckets. The new color-cache
  state belongs to the processor's upstream `UsedFilaments` transitions.
  Do not grow a separate color-change accounting pipeline in that shell.
  Any necessary cache consolidation must stay within this source boundary.
- `ViewerImpl.used_extruders` is currently a `BTreeSet<u8>`. Replace it with
  the source-shaped sorted map of per-extruder ColorPrint vectors; derive
  used ids from that map, without keeping redundant sets.
- Keep the existing `from_processor_result` conversion and lazy legend as
  adapters around these upstream concepts. Retire the Known difference for
  custom ColorChange data only when its tests and browser evidence pass.

## Included / deferred

Included: both reserved-tag spellings; processor custom-item metadata needed
for colors and Pause/Custom ordering; ColorChange volume statistics;
normal-mode custom time records and Z correction; Rust ColorPrint segments,
color palette and display records; result-derived path recoloring; lazy
metric display; reload/no-change behavior.

Deferred: automatic M600 injection, editable custom-height events / 3MF
custom-G-code authoring, bare-M600 color inference, pre-slice object coloring,
Stealth time estimation, firmware wait-duration simulation, imperial units,
unrelated flush/nozzle accounting gaps and full native GUI behavior. The
existing Rust Tool-color lookup and web FeatureType/Filament totals must not regress.

## RED-first plan and acceptance evidence

1. **Oracle and failure inventory before production code.** Extend only the
   diagnostic `tools/chain-probe/procgcode.patch` output to dump upstream
   custom items, raw color volumes and normal custom times, alongside existing
   move CSV. Record source revision, patch hash, executable hash, fixture
   bytes/config and command. Make the probe's hard-coded single-tool config
   explicit per fixture before multi-tool comparisons; diameter, density,
   flavor and tool count must match. The current oracle only dumps moves and
   cannot prove the new statistics. Derive expected records from the cited source
   and the native dump, never from Ares/browser observed values.
2. **Boundary RED fixtures.** Exercise the real processor/slice API using
   cube/project inputs and existing layer-change G-code template overrides.
   Use the same tagged bytes/config for native and Rust processing. Cover:
   two color changes with known model volumes; no change; bare M600; missing
   and invalid color; inactive tool; return to a previous color; repeated Z;
   change before first/after last extrusion; zero-volume intervals; tool
   changes; support/tower exclusions; Pause/Custom adjacency; reload. Assert
   exact identities/order/palette and source-justified float tolerances.
   If an isolated vgcode fixture is necessary, document these failure modes
   before writing it, and create it before its implementation.
3. **Core → vgcode → thin bindings.** Implement only after the source/spec
   ambiguities above are resolved. Split growing modules before 400 LOC.
   No filesystem, terminal, UI or native viewer runtime in core/vgcode/WASM;
   maintain browser WASM, Windows, macOS and Linux portability.
4. **Browser RED → display.** Native-backed E2E checks ColorPrint segment
   colors, event row order, corrected layer/Z data, m/g cells and trailing
   Print interval; switches to the existing FeatureType/Speed picker entries
   and back (Tool is not exposed by this web picker); reloads a no-change
   model to prove stale palette/rows disappear. Preserve the existing 14
   browser specs and add focused coverage, rather than replace them. Save
   screenshots, native/Rust JSON comparison, generated G-code SHA256 and
   bundle JSON under `target/web-e2e/color-change/`.
5. **Exit and review.** On fetched `origin/main`, run fmt, workspace clippy
   with `-D warnings`, workspace nextest, native `test(profile_entry)`
   reference group, WASM/lib/app builds, typecheck/lint and the entire browser
   suite. Record exact source-cited tolerances and Known differences. No
   inherited browser waiver for this behavior change. For another suspected
   load failure, report evidence and seek the coordinator's control-run
   ruling; do not retry until green. Send REVIEW-REQUEST with commit and
   artifacts; merge only after APPROVED.

## Final verification checkpoint (2026-10-07)

- Core/vgcode/CLI reference checkpoint `06fc4be9d`; final code `96f91ff28`
  changes only WASM transport to typed `Serialize` views. No dynamic-value
  allowlist changes. The path-scoped reference-source diff is empty.
- `cargo fmt --check`, workspace/all-target clippy with `-D warnings`,
  workspace nextest: **7,935 passed, 9 skipped**. The raw/stable processor
  pair was enabled, including all nine BBL cardinality fixtures, tag edge
  cases and three complete original-native M73/G-code byte oracles.
- Both base and candidate used the coordinator's corrected
  `/tmp/ares-parity-control-set-v2.sh`, including `profile_entry`, with
  fresh empty caches. Seeded: **561/561 each**. Official: base **552/561**,
  candidate **553/561**; both retain the same eight reference-environment
  failures. These are not represented as a fully passing official suite.
- The extra base failure, `tree-support/auto/hybrid/overhang`, meets the
  operator's ND rule: base/candidate **raw Ares bytes are identical**, SHA256
  `6f9a9ab3bf60d7987356befe5106df6c6a0453e9baa21a409ac4fac3939ebe1a`.
  Native outputs differ beyond generator metadata: `M73 P3 R25` changes
  insertion point. No rerun-to-pass was used.
- WASM, library and app builds, typecheck and lint completed for both trees.
  Full low-load browser runs: **14/14 base**, **15/15 candidate**, one valid
  invocation each. Initial JS gzip: base **124,949 B**, candidate
  **124,953 B**, strictly below 125,000 B. Filament legend and `presetSlice`
  are included. ColorChange checks use three original-native byte oracles,
  source records, missing usage, view restoration, reload and non-overlap.
- Setup-only failures are retained, not counted as validation: candidate
  reference artifacts initially violated the external-directory rule;
  browser collection initially lacked three archived generated input
  fixtures; an audit filter initially selected zero tests. Corrected setup
  changed no assertions, timeouts, reference binaries or budgets. The real
  dynamic-value audit RED was fixed with typed transport before rerunning.

Evidence: `/tmp/ares-colorchange-reference-43df7c95d/comparison.json` and
`environment-failures.json`; repository-local
`target/color-change-final-43df7c95d/` (gates, setup failures and build logs),
`target/web-e2e/` (fresh browser artifacts), and
`target/color-change-slice-oracle/` (raw/stable metadata and exact XYZ-bit
sets). The base browser artifacts are under
`/tmp/ares-web-filament-control/target/web-e2e/`.

The eight official environment failures and approved diagnostic distinctions
remain explicit. This Linux/Chrome verification is not a claim of native
Windows/macOS execution. Lint's VG1/UsedFilaments integration remains owned by
lint; the later merger must rebase and rerun the required gates.
