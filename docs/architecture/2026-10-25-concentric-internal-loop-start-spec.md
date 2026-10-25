# Concentric internal-solid loop-start parity SPEC

- **Date**: 2026-10-25
- **Base**: `5e8be001f3ac1e8cd2ba33e7ba5c7a95dbd4e87f`
- **Status**: SPEC (docs only). Approved discovery RR: `w3-geometry-discovery-5e8be001f`; localization evidence: `w3-concentric-internal-start-5e8be001f/LOCALIZATION.md`.

## 1. Witness

Stock-origin official-exported single-region cylinder
(`tests/parity/fan-mixed-timing/control.3mf` lineage) with exactly one GUI key
changed: `internal_solid_infill_pattern` `monotonic` → `concentric`.
Official and seeded Native double replay are stable and exit 0; validated API
and actual CLI exit 0 with raw API == CLI. Control matches Native byte-exact;
target differs in BOTH modes at the whole ordered-output level (first mismatch
line 743). Only the outermost loop of the internal-solid concentric island at
Z1 and Z6 differs: identical 7-junction closed cycle, identical orientation,
different starting junction — Native starts at the raw walk start, Ares at the
chained-nearest junction. Tail segments and three M73 progress-line positions
are consequences. Full inventory:
`w3-geometry-discovery-5e8be001f/internal-solid-concentric/difference-inventory.txt`.

## 2. Owning upstream boundary (pinned OrcaSlicer 8500fcd)

- `Fill/FillConcentric.cpp:105-145` — the standard `ipConcentric` Arachne
  variant: per-`no_overlap_expolygons` domain, `Arachne::WallToolPaths`,
  `all_extrusions` collected inset by inset, gate `if (extrusion->is_closed)`
  ONLY, then `start_at_index(last_pos.nearest_point_index(points))`, chained
  `last_pos` starting at `Point(0, 0)` (world coordinates), `clip_end(loop_clipping)`,
  `reorder_by_shortest_traverse`.
- `Arachne/WallToolPaths.cpp:573-670` `stitchToolPaths` +
  `Arachne/utils/PolylineStitcher.hpp` — where `is_closed` originates:
  fragments chain into `closed_polygons` (reconnected and marked
  `is_closed = true`) or stay open `stitched_polylines`.
  `stitch_distance = bead_width_x - 1`.
- `Arachne/utils/ExtrusionLine.hpp:205-222` `to_thick_polyline` —
  non-closing point/width flattening.
- `Fill/Fill.cpp:1152-1180,1289` — dispatch: `detect_narrow_internal_solid_infill`
  narrow-solid reclassification to `ipConcentricInternal` (a DIFFERENT variant,
  not this SPEC) and `use_arachne` for concentric patterns.
- `PrintConfig.cpp:2074-2095` — the GUI enum key.

## 3. Rust destination and proven divergence

Dispatch mirror: `project_slice/fill_entities/generate.rs`
(`Configured(ProcessInfillPattern::Concentric)` → `concentric::append_configured`,
full-density branch) → `generate_standard_thick_polylines` →
`fill_entities/concentric/finalize.rs::finalize_standard_polylines`
(gate `first == last && width.first == width.last` →
`concentric_sparse::nearest_point_index(last, points)` → `start_at_index`),
fed by `arachne/wall_toolpaths.rs::generate` +
`arachne/extrusion_line.rs::to_thick_polyline` +
`arachne/wall_toolpaths/stitch.rs`.

Localization verdict (two-sided GDB + exact-coordinate elimination,
`LOCALIZATION.md`):

1. **Raw walk starts are identical** — Rust's pre-rotation first junction of
   the Z1/Z6 rings equals the Native emitted start exactly
   ((154741,14962) / (154756,14815) local). Candidate "raw walk start
   divergence" is disproven.
2. **Upstream leaves the witnessed rings unrotated** — no rotation target
   (world-origin anchor or chained anchor) can produce the Native start; the
   upstream gate is `is_closed` only, so upstream `ExtrusionLine::is_closed`
   is false for these rings.
3. **Rust rotates them** — Rust's stitch chain closure presents the same ring
   closed (`first == last`, `width.first == width.last`, GDB-proven), and
   `finalize_standard_polylines` rotates all 50 rings (chosen index != 0 for
   every one), moving the witness rings off the raw start to the
   chained-nearest junction.

**Fix boundary (single proven site)**: `arachne/wall_toolpaths/stitch.rs`
closed-chain classification — the Rust chain walk closes this minimal ring
under the same `max_distance = inner_spacing - 1` / `0.01mm` snap inputs that
upstream uses, while upstream `PolylineStitcher` leaves the identical fragment
chain open. GREEN must reproduce upstream's open classification for this ring
so the raw walk start is emitted. No change to `concentric.rs` or
`extrusion_line.rs` unless the RED-phase C++ capture contradicts the
elimination argument.

**RED-phase obligation before GREEN**: directly capture
`all_extrusions[i]->is_closed` for the witness ring on the C++ side (GDB,
struct offsets on `WallToolPaths::generate` output) to confirm the
by-elimination conclusion; only then edit `stitch.rs`.

## 4. Arachne shared-surface disclosure

`stitch.rs` is shared by perimeter Arachne generation (wall toolpaths). Any
closure-classification change can alter perimeter loop classification. The
witnessed ring is a minimal concentric ring (feature below `min_bead_width`),
which perimeters can also produce. GREEN acceptance must therefore include
the directed pair PLUS the existing perimeter/Arachne parity surface (workspace
Clippy, directed concentric tests, and at minimum the arachne-related directed
cases) before any FULL request; a behavioral change in perimeter outputs would
be a boundary violation and must be reported instead of absorbed.

## 5. Included / deferred

Included: the witnessed standard-variant internal-solid concentric loop-start
behavior only; directed official+seeded control/target whole-output
acceptance with runtime-generated Native (no committed Native artifacts);
inputs `tests/parity/concentric-internal-start/{control,target}.3mf` derived
from the stock fixture by the single GUI key.

Deferred (no change in this chain):
- `FillConcentricInternal.cpp` narrow-variant path (`ipConcentricInternal`)
  and its pop/rotate/re-append gate — the layer-25 narrow ring currently
  matches and is untouched.
- Sparse concentric and the standard variant's sparse fallback.
- Latent divergence (a): Rust's standard gate adds a `width.first ==
  width.last` sub-condition upstream does not have (`FillConcentric.cpp:111`
  is `is_closed` only). Disclosed; not the witness root (witness rings have
  equal widths on both sub-conditions).
- Latent divergence (b): the first ring of each call anchors at `Point(0,0)`
  in the concentric-local frame (object center) in Rust vs world plate corner
  upstream; nearest-junction selection is translation-invariant for chained
  anchors, so this can only diverge on the first rotated ring per call.
  Currently passing rings coincide on near-circular geometry. Disclosed; not
  fixed here.
- All queued-window files (brim/*, motion/scan.rs, format.rs, jerk,
  seam_placement/alignment, config value.rs, fill_entities/grid.rs) — untouched.

## 6. Acceptance

RED (this chain): tests-only addition
`runner/concentric_internal_start.rs` via `native_stream_case::check` +
`native_stream_cases` group registration; control (monotonic) must PASS whole
output against runtime Native; target (concentric) must FAIL against current
code (real difference), failing on the witnessed loop-start divergence.
Without a Native oracle the case must error out, never silently pass.
GREEN (later chain): minimal `stitch.rs` change after the C++ `is_closed`
capture; target flips to PASS with control still PASS; no perimeter
output change.
