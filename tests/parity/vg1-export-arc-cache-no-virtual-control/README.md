# VG1-induced M73 arc-cache INPUT pair

Source `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:771-810`,
ExportLines::update dynamic cache cursor/remaining-internal walk;
caller1458–1471 and process_VG14278 explain estimate/export id drift.
Exact future Rust destination `processor/estimate/elapsed.rs`, minimal
estimate.rs call wiring only if needed; see
`docs/architecture/2026-10-21-vg1-export-arc-cache-no-virtual-control.md`.

Actual official-exported INPUT3MFs copied unchanged. Existing two-active-
filament X2D geometry, prime tower0/skirt0, no geometry edits. GUI sole
change_filament_gcode: target real existing X2D macro; control SAME macro
with only ;VG1 lines removed, preserving SYNC/gated/arc-lift commands.
Manifest freezes whole/per-entry SHA, only-one-payload/key proof and
relative provenance/evidence ID, NOT evaluated Native/output constants.
No expected G-code snapshots or private Native oracle.

Two DIRECT native_stream_case::check calls reuse NativeCase double replay
then complete validated API/bare CLI comparison. No copied helper/replay/
normalization/orchestration, ignored/expected-failure tags or tolerance.
CPU11/thread1/job1, ARES_REQUIRE_ORCA=1, explicit official then seeded
Native environments and retained artifact roots:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(vg1_export_arc_cache_)'
```

Require whole controlPASS/targetactualFAIL with stable Native repeats and
valid public generation/both comparisons. Setup/environment-disabled runs
are NOT RED evidence. Only existing generated-by identity/date handling;
NO M73/header/time/E/XY/fan/comment/config/trailing-line masks/sorting.
M73 is output; first-M73 failure is legitimate strict RED, not permission
to ignore it or patch time totals. Source block caches already match.

External evidence ID `w5-multimat-vg1-export-arc-cache` retains exports/
launches/settings/diagnostics. Such source-counter diagnostics are NOT
formal public API/CLI acceptance evidence. Freeze tests/inputs/helpers/
docs after RED; any later bounded GREEN residual requires STOP/report.
Send SPEC+RED RR then STOP, no production/GREEN/FULL/rebase/integration.
Ninth chain is based on frozen eighth tip d0ebd29ad; all prior approved
tips remain unchanged. No new candidate crate/dependency/pipeline.
