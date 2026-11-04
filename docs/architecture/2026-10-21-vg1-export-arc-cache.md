# ARD: VG1-induced M73 arc-cache cursor parity

## Upstream owner, trigger and exact Rust destination

Upstream `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp`:
- `process_gcode_line:2890–2923` dispatches `;VG1` as a virtual command;
- `process_VG1:4278` increments `m_g1_line_id`; its time blocks do not
  save a move vertex (`:4544–4546`);
- `run_post_process:1458–1471` advances physical export counters for
  G0/G1/G2/G3, not for VG1. Estimate ids and export counters can drift.

Owning rewrite slice is **ExportLines::update:771–810** plus the
`process_line_move:1241–1252` lookup / caller counter at1458–1471:
1. Compute temporary physical counter+1 and advance a persistent update
   cache cursor while cached id is below that desired id.
2. Without an exact cache-id hit, return zero internal count.
3. For G2/G3 walk THAT current cache cursor while its stored
   remaining_internal_g1_lines is positive. Advance cursor/counter/ret.
4. Caller probes progress cache at physical counter+ret and advances by
   1+ret. Update and progress lookup retain separate upstream cursors.
This is not equivalent to reading the current source arc's segment count.

Exact Rust destination ONLY
`crates/ares-core/src/project_slice/gcode_emit/processor/estimate/elapsed.rs`
`elapsed_snapshots:35–113`. Current M73 lookup uses precomputed
arc_segment_counts[index]/arc_seam_excluded[index] tied to the physical
source line. Once VG1 shifts ids, that shortcut queries different entries.
`estimate.rs` existing call/signature wiring may change ONLY if necessary.
Any required edit outside these two files is out of scope.

Existing `estimate/export_cache.rs::snapshots` already ports the dynamic
ExportLines timestamp cursor; it is a source-aligned adjacent reference,
not permission to change timestamp/preheat/backtrace behavior. No new
helper/public API/dependency/fallback or independent Ares pipeline.

**Do not change VG1 arithmetic, planner or cache construction.** Initial
virtual time-block hypothesis was falsified: actual Native/Rust2184 cache
entries are identical in id, remaining count and float elapsed. Preserve
virtual motion admission/ids, time totals, material/vertex state, source
arc discretization, eligibility and progress formatting. This slice ports
lookup semantics, not a symptom-level timing adjustment.

## Main actual-exported GUI INPUT pair

Existing `tests/parity/validation/bbl-toolchange-boundary.3mf` supplies
active alternating two-filament Bambu X2D geometry. Geometry unchanged,
prime towerOFF/skirt0. GUI sole key **change_filament_gcode**:
- target: real X2D macro from the existing multi-filament fixture;
- control: SAME macro with only ;VG1 lines removed. All SYNC/gated/arc
  lift and other commands retained.

Actual official-exported INPUT3MFs copied unchanged, no post-export edit.
Same ZIP inventory; only Metadata/project_settings.config differs and
only change_filament_gcode differs. Freeze whole/per-entry SHA and input
provenance, not Native evaluated values or expected G-code snapshots.

| Input | SHA256 |
|---|---|
| control |61d36dff5dfb995fb3cd1f2bcb3ece0d9e79c5476137b05ddd495d4b901368fb|
| target |f6a1b0ab02e4be9f4d3c20e6dcf09d55b0ba37f4b47e3989053de03adefac169|

Provenance uses repository-relative paths only. The original macro source is
`tests/parity/validation/multi-filament-bridge-anchor-narrowed.3mf`.

## Honest causal evidence and strict acceptance

Discovery official1/official2/seeded stable: control completePASS,
target strictFAIL, first1524 premature M73 insertion.5101 raw mismatched
line slots are alignment fallout, NOT5101 physical path errors. Separate
alignment diagnostic finds all non-M73 bytes equal, but acceptance NEVER
masks M73. It is output, and a legitimate actual RED in this family.

Minimal two-VG1 real-macro subset independently triggers the same class;
SYNC/gated/arc-lift subsets pass. On the minimal witness read-only runtime
captured2184 equal cache entries and full Rust preprocessor arc arrays.
Source-counter replay using same actual cache and Native measured clock
reproduces all22 Native /34 Rust M73 transitions at motion anchors.
First arc divergence: source1213,physical65,counter85,Native ret0 versus
Rust source-arc ret21. Diagnostic replay is not a production patch or
whole-GREEN proof. Rust total was not captured; do not claim measured
total equality. Successful full debug outputs match their own CLI;
failed/pending/incomplete probes are excluded. Discovery evidence is not
embedded into regression expected outputs.

Formal RED uses TWO DIRECT native_stream_case::check calls. Existing
NativeCase double Native replay then complete validated API/bare CLI
comparison; no copied orchestration/normalizer/helper/Native constants,
private seam, ignored or expected-failure flags. Official then seeded. Require controlPASS and genuine targetFAIL with
valid public generation and both comparison attempts retained. No setup
or environment-disabled run counted. Only existing generated-by identity/
date normalization; NO M73/time/header/fan/E/XY/comment/config/trailing-line
mask, tolerance, numeric equivalence or sorting.

## Deferrals and phase exit

Defer all virtual math/block/planner/cache-build/material/vertex changes,
other export policies, ramming/cooling tube/tower geometry, skirt,
FanMover EPSILON/thresholds, earlier processor firmware chain, header/
placeholder/value rendering and all other families. Any later bounded GREEN residual requires STOP/report,
not expanding scope or weakening the strict comparator.

Commit SPEC then input-only pair/tests/manifest/README RED and run dual-mode
targeted checks. Freeze tests/inputs/helpers/docs thereafter.
