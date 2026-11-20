# Type2 planned ram-band restart

## Upstream boundary and decision

Port the boundary ram-band restart in OrcaSlicer2.5.0
`libslic3r/GCode/WipeTower2.cpp::toolchange_Unload`:

- `1489–1493`: when the old tool has a boundary wipe start and the plan iterator
  is valid, obtain its planned `ramming_depth` by matching `old_tool`;
- `1699–1707`: when that depth is positive, travel to the left edge of the fresh
  wipe row beyond the reserved ram band, then set `m_left_to_right=true`;
- otherwise retain the ordinary ramming and non-ramming restart branches.

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/unload.rs::Generator::unload`.
Use the existing per-tool ramming predicate, gap-wall selection and
`wipe_start_offset_after_ram`; do not invent a new planner or restart formula.
The per-tool reservation specified in `type2-per-tool-ramming-reservation.md`
is a prerequisite.

## Causal boundary

The first simulated toolchange has matching reserved ram depth1.5 but different
unload return positions. Native restarts at `(0.75,2.5)`, while Rust retains the
ordinary ram-end point `(52.49225616455078,1.75)`. After their common half-width
Y adjustment, the wipe inputs differ and their outgoing directions are already
opposite. The later first-layer finish consumes this retained direction state
and selects a different fill-box corner.

The earlier toolchange entry-offset gap is distinct. Its current-position
mismatch reaches the common ram start before this restart; its result-start
metadata is not the source of the simulated saved finish volume. Do not reset
final generation direction or choose a different finish corner to compensate.
Port the upstream transition at its owning unload boundary.

## Included behavior

1. Resolve planned depth only for non-SEMM, ramming-enabled gap-wall starts.
2. Preserve planless priming and no-matching-old-tool behavior as zero planned
   depth, corresponding to the upstream end-iterator/matching guard.
3. For positive planned depth, reuse the source expression:
   `box.lower.y + depth_traversed + wipe_start_offset_after_ram(depth, first_layer) + perimeter`.
   The X coordinate is the existing ram-start left edge.
4. Set the outgoing wipe direction to left-to-right only in that positive-depth
   restart branch. Preserve existing ordinary restart behavior otherwise.

This remains a private, platform-neutral rewrite of `WipeTower2`, with no
filesystem, renderer, terminal, public API or adapter behavior changes.

## Test-first verification

Use the existing public TiQ2 whole-file and retained-role-fan E2E witnesses.
Commit this specification and confirm that both tests still fail on the
prerequisite base before porting the restart. Keep fixture bytes,
generated-by-only normalization, complete ordered-line/line-count comparisons
and all existing assertions.

Run a paired wipe-tower/TiQ2/SEMM/Type2 scope, including the existing BBL boundary
controls. Retain identical selected identities and report every transition;
require zero passing-to-failing transitions. The two original tests may still
fail, but the first complete-stream mismatch must move later or the tests must
pass without weakening the comparator.

Read-only GDB must qualify complete streams and observe the restart/wipe handoff
before and after. Record all75 cooling handoff IDs as downstream evidence;
never change cooling coefficients, thresholds, fan policy or time precision to
make those numbers match. Run formatting, workspace all-targets clippy and the
core wasm32 check.

## Deferred behavior

- Non-SEMM offset in `WipeTower2.hpp::toolchange_entry_pos` and its wall-gap
  placement consumers.
- Boundary inward-entry travel and intermediate ram-continuation geometry.
- Other wipe, role/footer restoration and cooling behavior.

Those remain adjacent upstream rewrite slices. No legacy fallback, native-output
snapshot, new comparison rule or independent Ares pipeline is introduced.
