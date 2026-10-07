# ColorChange / M600 preview — source-cited rewrite spec

Status: draft for coordinator review; no runtime implementation yet.
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
  rows (`GCodeViewer.cpp:4221, 4230`). Zero-volume intervals can expose a
  cardinality problem. Before implementation, probe these inputs against the
  native result and record the outcome. Do not silently zip/truncate vectors,
  invent zero volumes or copy an unchecked C++ access into Rust. If the native
  display contract is undefined for a case, report it to the coordinator and
  make that limitation explicit before approving the implementation contract.

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
existing Tool/FeatureType views and Filament totals must not regress.

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
   Print interval; switches to FeatureType/Tool and back; reloads a no-change
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
