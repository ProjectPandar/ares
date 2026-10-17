# Diagnostic: crosshatch sparse-infill path ordering divergence (the "micro-island")

Status: diagnosis complete, spec for the fix slice follows evidence below.
Supersedes the M73 placement drift amendment in `50366ffa2` (rejected): the M73
drift was a downstream symptom of this divergence, not an export-walk bug.

## Reproduction

Standalone repro (all inputs derived from `tests/parity/per-object-skirt-brim-types/outer-only.3mf`):

- both build items scaled `5.5 5.5 10` (positions unchanged),
- project settings: `brim_type=no_brim`, `brim_width=0`, `brim_loops=0`,
  `skirt_loops=0`, `sparse_infill_density=24.5`, `sparse_infill_pattern=crosshatch`.

Native (`orca-official-242`) vs Ares (main `918ae452d`) first body diff at line
~1227, layer 3 (`Z .6`), immediately after the layer-change wipe:

```text
native: G1 X124.072 Y101.117 Z.6
ares:   G1 X120.374 Y104.072 Z.6
```

The same corridor of crosshatch loops is printed in a different order. In the
earlier `x24.5` probe (anchor defaults) the visible effect is the "micro-island":
ares emits one loop as a separate 13-line path that native folds into a
neighboring path (path start/end pairing shifted by one lane:
ares-only starts `(101.123,115.98)`, `(110.33,124.02)`;
native-only starts `(98.585,115.98)`, `(112.868,124.02)`).

## Evidence chain

1. **Polyline sets are identical.** Canonicalized (direction-normalized,
   coordinate-rounded) sparse-infill path sets of layer 3 are equal: 72 paths on
   each side, zero set difference. Pattern generation
   (`FillCrossHatch.cpp:28-176`), `intersection_pl` clipping, the
   `minlength` filter (`FillCrossHatch.cpp:210-218`) and `multiline_offset` are
   therefore faithful on this repro.
2. **The connect port is excluded.** With `infill_anchor=0/0.4/1` (upstream
   `FillParams::dont_connect()`, `FillBase.hpp:52`) the divergence persists
   unchanged (stripped bodydiff 664 for all three), and path sets remain equal.
   Under `dont_connect()` upstream runs `chain_polylines` only
   (`FillBase.cpp:1820-1828`).
3. **Conclusion:** the divergence is in polyline chaining order —
   `chain_polylines` / greedy chaining / two-exchange improvement — not in
   geometry generation or boundary connection.

## Upstream owner

- `OrcaSlicer/src/libslic3r/ShortestPath.cpp:1968-1994` — `chain_polylines`:
  `chain_segments_greedy2` greedy chaining, then
  `improve_ordering_by_two_exchanges_with_segment_flipping`
  (`ShortestPath.cpp:1918-1965`), which runs
  `reorder_by_two_exchanges_with_segment_flipping` over `FlipEdge`s
  (`ShortestPath.cpp`, 2-opt with segment flipping, earlier in the file).
- Call site: `Fill::chain_or_connect_infill` `FillBase.cpp:1820-1828`
  (`dont_connect()` branch), reached from
  `FillCrossHatch::_fill_surface_single` `FillCrossHatch.cpp:225-230`.

## Ares destination boundary

- `crates/ares-core/src/project_slice/perimeters/classic/shortest_path/polyline_chain.rs`
  (`chain_polylines`, claims port of `ShortestPath.cpp:1968-1994`):
  `advanced::chain` (greedy2) + `two_exchange::improve`.
- Reached for crosshatch via `crates/ares-core/src/fill/connect.rs:74`
  (`dont_connect` branch of the `chain_or_connect_infill` port).

## Hypotheses (RED will discriminate)

1. greedy2 port tie-breaking / epsilon: ares passes
   `1.0e-4 / scale.factor()` as epsilon where upstream compares scaled-integer
   `Point` distances exactly (`chain_segments_greedy2<Point>`); equal-distance
   ties may resolve differently.
2. `two_exchange::improve` port: divergent 2-opt acceptance, crossover
   selection, flip bookkeeping, or termination, reordering a tie cluster the
   upstream heuristic leaves in greedy order.
3. f64-vs-`coord_t` distance rounding in `FlipEdge` costs flipping the order of
   two near-equal candidate moves (upstream stores `Vec2d` endpoints but
   compares with doubles derived from exact integer points).

## Included / deferred

- Included: chaining order for open polylines (infill + brim lines), the exact
  upstream semantics of greedy2 and the two-exchange improvement, RED parity
  witness on the repro above.
- Deferred: `reorder_by_three_exchanges_with_segment_flipping{,2}` (disabled
  upstream behind `#if 0`), `chain_path*` item chaining, connect/anchor logic
  (already excluded as the cause).

## Relation to prior scaffolding

The M73 export walk (processor.rs) is a faithful port — decisive simulation of
the upstream walk over oracle per-block times reproduces both native and ares
placements exactly; total-time difference (+0.08 s, this divergence's extra
travel move) shifts every M73 R-boundary globally. No processor change belongs
in this slice; the M73 drift disappears once this ordering divergence is fixed.
