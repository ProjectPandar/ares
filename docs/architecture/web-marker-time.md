# Tool-position Time row — source-cited thin-display spec

Implementation proceeds RED → GREEN.
The oracle additionally requires the source/adapted diff and hashes to be
committed, with only member/parameter adaptations. The tagged-Time ruling
is recorded in [ColorChange Known difference](web-colorchange.md#known-difference-tagged-marker-time).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths below are relative to `OrcaSlicer/` unless prefixed with a crate/web path.

## Proposed slice and source correction

Port `libvgcode::ViewerImpl::get_estimated_time_at` and expose the existing
`GCodeViewer::SequentialView::Marker::render_position_window` **Time** row.
This is a rendering-neutral libvgcode rewrite plus its thin GUI adapter, not
an Ares-owned time estimator or slicing pipeline.

The README's suggested moves-slider time/distance gap is not a valid target:
`GCodeViewer.cpp:1765-1821` puts G-code IDs in `alternate_values`, and
`IMSlider.hpp:82` merely stores that vector. It does not establish a
horizontal-slider elapsed-time/distance feature. Keep current slider labels,
stop grouping and navigation unchanged; do not invent this feature.

By contrast, `GCodeViewer.cpp:452-454` explicitly formats the Time property as
`get_time_dhms(estimated_time) + " (" + printf("%.3fs", vertex_time) + ")"`.
Ares already transports vertex times; the missing pieces are the source-owned
inclusive lookup and this row. No additional processor fields are required.

## Source → destination

| Source owner | Included behavior | Rust / thin adapter destination |
| --- | --- | --- |
| `libvgcode/include/Viewer.hpp:326-329`; `libvgcode/src/ViewerImpl.cpp:1461-1465` | Inclusive, ordered f32 sum of vertex times in the current mode | `crates/ares-vgcode/src/viewer.rs`, focused time-data module if needed |
| `libvgcode/src/ViewerImpl.cpp:985-1010` | Load/reset and per-mode running totals | The existing Rust viewer load pass; optional prefix storage is owned here, never in TS |
| `slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:192-268` | `MoveVertex::time` → `PathVertex::times`; inserted path-start vertices have zero times | Existing `ares-vgcode/src/convert.rs`, unchanged and covered by controls |
| `slic3r/GUI/GCodeViewer.cpp:345-355,452-454` | Use the selected property-window vertex, including non-FeatureType seam substitution; pair cumulative time with that vertex's own duration | Existing `ares-vgcode/src/sequential_view.rs::ToolMarker.window_vertex` plus a neutral timing record |
| `libslic3r/Utils.hpp:540-562` | `get_time_dhms` display formatting, including the <=1 second branch | Display-only formatter in lazy `web/app/src/preview/ToolPosition.tsx` or adjacent file |
| `slic3r/GUI/GCodeViewer.cpp:417-454` | Append Time to expanded properties, retain fold/layout and existing rows | Existing lazy ToolPosition surface; no new panel or view mode |

## Non-negotiable decisions

1. **No core change.** Do not touch timing arithmetic, stop registration, M73,
   `UsedFilaments`, G-code generation, slicing, or the native-marker correction.
   Existing processor times are inputs. If a native timing discrepancy blocks
   the display oracle, report it; do not fix it here.
2. **Three distinct terms:** a vertex duration is `PathVertex::times[mode]`;
   elapsed time is the inclusive prefix through the selected vertex; the
   property-window vertex is the existing Rust-selected `window_vertex`, not
   necessarily the arrow's vertex. Neither time is layer duration, remaining
   print time, a color-change interval, or a wall-clock measurement.
3. **Exact arithmetic:** preserve upstream sequential f32 addition from
   `0.0f`, including zero-duration vertices and every hidden/out-of-range
   vertex preceding the selected vertex. No f64 sum, parallel reduction,
   visible-only accumulation, G-code-ID grouping, interpolation or clamping.
4. **Rust-owned lookup:** reuse the existing load loop's ordered per-mode
   accumulation to store inclusive prefixes, making cursor queries O(1).
   Account for `TimeMode::COUNT * sizeof(f32)` additional bytes per vertex;
   rebuild with each accepted nonempty load and do not keep a second vertex
   copy. Preserve current empty-load behavior rather than adding reset rules.
   No global cache or independently accumulated frontend state.
5. **Thin transport:** expose a small typed timing record containing raw
   `elapsed_seconds` and `vertex_seconds` for the already-selected window
   vertex. A narrow WASM getter may accept that engine-provided index and
   validate it at the public boundary. The worker calls it synchronously
   alongside `toolMarker` in the same `movesHigher` response and adds the
   timing record to the typed marker payload. Preserve existing ID buffers,
   old marker selection and stale-response rejection. No `serde_json::Value`,
   dynamic-value allowlist additions, or JS summation.
6. **Normal-mode browser scope:** use the viewer's current Normal mode. Keep
   per-mode neutral data correct, but add no Stealth estimate or mode picker.
7. **Display only:** append `Time` after Temperature in the expanded table,
   leaving still-unavailable Acceleration/Jerk/Pressure Advance deferred.
   Restore f32 payload values before formatting (e.g. `Math.fround` after JSON
   transport). Match the source's duration format and f32 decomposition; zero and exactly
   one second use the six-decimal seconds branch, not a rounded integer.
   Render finite upstream values without new fallback values or N/A guesses.
8. **Preserve visual authority:** the existing dense Orca-like Operate surface,
   fold button, typography and dark/light tokens remain. No redesign, new
   timeline, animation or automatic expansion. Keep the row accessible in the
   semantic table, unoccluded by the slider, and inside the lazy preview chunk.
   Initial JS gzip remains **strictly <125,000 B** (current 124,953 B).

## RED-first failure inventory and oracle plan

Write native-backed failures before production changes. Prefer real slice →
processor → vgcode → WASM → browser checks; isolated prefix/format cases are
justified only for the following numeric and selection boundaries.

- First/last vertex, zero-duration phantom vertex, repeated G-code IDs and arc
  tessellation: inclusive endpoint and vertex identity must not drift.
- Hidden travel/roles, a narrowed layer range, forward/backward scrubbing:
  elapsed time still includes the complete prefix, not the displayed subset.
- Seam current vertex in FeatureType versus another existing view: both time
  fields follow `window_vertex`; changing the arrow position is out of scope.
- Reload A→B, no visible marker, and stale worker replies: no old timing row,
  no new marker latch behavior, no mismatched ID/timing response.
- f32 values that distinguish ordered f32 from f64/reassociated sums; zero,
  one second and its adjacent f32 values; minute/hour/day boundaries;
  zero and fractional per-vertex durations with native `%.3f` rounding.
- Existing ColorChange/Pause/Custom data: no altered intervals, palette,
  statistics, corrected positions, G-code or M73 bytes.

**Oracle provisioning is the first implementation gate.** Preserve raw native
processor CSV/JSON and source revision/binary hashes. Add a read-only probe
using upstream `libvgcode::convert(GCodeProcessorResult, ...)` from
`LibVGCodeWrapper.cpp` to obtain native PathVertex data.
The `ViewerImpl` getter is an ordered `std::accumulate` expression, but full
`ViewerImpl::load` requires GL initialization. A headless probe may therefore
compile a source-extracted copy of precisely `:1461-1465` with only member/
parameter adaptation, alongside the real `Utils.hpp::get_time_dhms` formatter.
Archive the extracted source and hash; label this honestly as a **native
source-kernel oracle**, not execution of the full native viewer or a screenshot
oracle. Do not use a Rust/JS-generated expected sum. If this diagnostic cannot
be built without changing arithmetic/conversion, stop and amend the spec;
there is no production or oracle fallback.

For each selected native vertex emit identity, duration/prefix **f32 bits**,
mode, and the exact final Time text. Assert Rust bits and browser text against
these records. Tagged geometry cases retain the raw/stable marker
pair and complete difference-set rule; the stable patch remains marker-only.
Stable source-kernel vertex IDs and bits are authoritative for Ares. Retain
both native streams; align raw/stable by G-code ID, explicitly accounting for
phantoms and bounded elapsed propagation as documented in the Known
difference. Differences outside that boundary still require an explicit ruling;
the existing XYZ exception is not a blanket timing waiver.
Generated G-code/M73 use the unmodified official slicer, never that patch.
Keep all RED inputs and source-kernel fixture outputs reproducible and committed.

## Implementation sequence

1. Provision source-kernel oracle and failure fixtures; commit tests-only RED.
2. Port the inclusive query/prefix data and selected-vertex timing record into
   vgcode; verify native f32 bits. No core or existing conversion changes.
3. Add typed WASM/worker transport, then a browser RED for the absent Time row;
   add the lazy display formatter/row and verify exact native text.
4. Check first/middle/last selection, seams, filters, reverse navigation,
   reload, both themes and expanded-table non-overlap. Preserve screenshots,
   native records, raw G-code hashes and bundle evidence.
5. Run fmt, all-target workspace clippy with
   `-D warnings`, workspace nextest (including dynamic-value audit), WASM/lib/
   app builds, typecheck/lint and fresh full base/candidate browser
   suites once each. Include Filament/ColorChange and `presetSlice`. Run the
   full native reference suite; do not
   inherit a waiver for code changes or erase a RED by retrying.

## Exit / deferred behavior

Exit requires source-cited numeric and formatting parity, unmodified G-code
and M73, no initial-bundle regression or budget change, green Rust/browser
checks, and explicit evidence for any accepted oracle ND.
Production files approaching 400 LOC must be split within the changed boundary.

Deferred: Acceleration/Jerk/Pressure Advance recording/display, actual-speed
profile, Stealth estimator/UI, slider time/distance labels, imperial units,
remaining-time prediction, new marker selection, new view modes, full native
OpenGL/UI parity and any pipeline/time-accounting redesign. Existing Rust
viewer, marker and worker structures remain the adapters around the cited
upstream concepts; replace only the missing getter and Time-row omission.
