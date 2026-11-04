# Thin-tip support fill connection specification

This slice follows ARD-0103's retained public RED, not its bounded closure.
Reference: OrcaSlicer v2.4.2, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Source and destination

- `src/libslic3r/Support/SupportCommon.cpp::fill_expolygons_with_sheath`
  and base support fill dispatch provide the unchanged caller context.
- `Fill/FillRectilinear.cpp::fill_surface_by_lines` dispatches sparse support
  connection to `Fill/FillBase.cpp::Fill::connect_base_support` (2247–2708).
- `FillBase.cpp::mark_boundary_segments_overlapping_infill` (1352–1429)
  and `line_rounded_thick_segment_collision` (741–845) own the proven fix.
  `base_support_extend_infill_lines` (1834–1948) and subsequent connection
  phases are downstream consumers, not changed by this slice.
- Rust destination: `crates/ares-core/src/fill/base_support/mark.rs`
  (`mark_boundary_segments_overlapping_infill`), with the existing
  `fill/connect/` graph used only as the upstream boundary graph representation.
  These modules must match upstream execution, not invent a new connection rule.

Included: diagnose the first support path divergence for tip diameter 0.1,
repair the proven source-semantic mismatch, preserve line/contour ordering and
complete output comparisons. The regression seam is the existing
public project-to-G-code comparison in CLI `option_coverage/tree_activation.rs`:
raw Grid normalized to Organic and explicit Organic, with support width 0.1
and layer/first-layer height 0.08. Existing assertions remain unchanged.

Deferred: unrelated option matrix cleanup, wipe tower/AdaptivePA, Arachne,
new Tree generator features, general connector redesign, and cross-worker
output equivalence. No guards, ignored tests, fallback or comparator changes.

## Acceptance

Reproduce retained RED before production edits; use cached full-output replay
for diagnosis, then execute fresh official OrcaSlicer 2.4.2 single-CPU and seeded
single-CPU/seed0 full activation comparisons (28 pairs each, previously 26 PASS).
The expected explicit Organic100 native rejection remains separately reported.
Require workspace Nextest, Clippy, rustfmt and core+adapter WASM compilation.
