# Fuzzy skin `allwalls` seeded-RNG consumption order (SPEC)

Date: 2026-10-25

## Upstream boundary (OrcaSlicer 2.4.2)

- `src/libslic3r/PrintConfig.hpp:50` — `enum class FuzzySkinType`.
- `src/libslic3r/PrintConfig.cpp:195-200` — enum key map (`"allwalls"` →
  `FuzzySkinType::AllWalls`); `:3540-3559` — `fuzzy_skin` option definition.
- `src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:477-494` — `should_fuzzify`:
  `AllWalls` fuzzifies contours and holes at every `loop_idx`; `:485-487` the
  first-layer gate; `:560-578` `apply_fuzzy_skin` single-region fast path
  consumes the RAW loop polygon (`fuzzified = polygon; fuzzy_polyline(…, true,
  slice_z, config)`; no rotation/reversal).
- `src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:294-342` — `fuzzy_polyline`
  classic-displacement draw sequence over the global seeded stream: one
  `random_value()` for `dist_left_over`, then per jitter point a spacing draw
  (`min_dist + random_value()*range`) plus a noise draw.
- `src/libslic3r/PerimeterGenerator.cpp:1345-1352` — onion loop builds
  `contours[i]`/`holes[i]` with depth `i` (0 = outermost/external wall).
- **Ordering rule under port**: `PerimeterGenerator.cpp:1450` passes
  `contours.front()` (the same-level siblings, vector order) to
  `traverse_loops`; `:146-151` the FIRST flat pass applies
  `apply_fuzzy_skin(loop.polygon, …, loop.depth, loop.is_contour)` to every
  sibling of that level before any child is visited; only afterwards does the
  second pass (`:240-241` chain order, `:254-255`) recurse into each loop's
  children — which recursively applies the same rule. RNG consumption order is
  therefore: same-level siblings (loops vector order) first, then chain-ordered
  recursion into children, level by level.
- `PerimeterGenerator.cpp:274-278` — emission/print order is separately
  children-first for contours (`out.append(children); out.emplace_back(eloop)`);
  G-code order (inner→outer) is unchanged by the RNG order.
- `src/libslic3r/PrintObject.cpp:3707-3708` — subthreshold point
  distance/thickness disables fuzz (validation edge, not implicated).

## Ares boundary (current, main `28539c7fb`)

- `crates/ares-core/src/project_slice/perimeters/classic/materialize/tree.rs:26-33`
  — `materialize_surface` uses an explicit stack: `Frame::Visit(seed)` pushes
  `Frame::Finish(seed)` (the parent) and then the children's `Visit` frames, so
  every child `Finish` runs before its parent's.
- `tree.rs:34-52` — each `Finish` calls `materialize_seed` (fuzz happens here)
  and nests `RawPathNode { paths, children }`.
- `crates/ares-core/src/project_slice/perimeters/classic/materialize/path.rs:31`
  and `:75` — `fuzzy_polygon(record, seed, scale, context)` inside
  `materialize_seed` / `materialize_overhang_from_lower`; `:137-160` passes
  `loop_index = seed.depth`, `is_contour = seed.is_contour` and keeps the
  fuzzified scaled points verbatim.
- `crates/ares-core/src/project_slice/perimeters/classic/fuzzy_skin.rs:28-45`
  (`apply`) → `crates/ares-core/src/perimeters/fuzzy_skin.rs:165-177`
  (`fuzzified_scaled_points`) → `crates/ares-core/src/perimeters/fuzzy_skin_seeded.rs:10`
  (`fuzzify(…, context.rng())` — the shared seeded MT19937 stream).
- `crates/ares-core/src/perimeters/fuzzy_skin.rs:225-246` — `should_fuzzify`
  already mirrors `FuzzySkin.cpp:477-494` (decision parity confirmed).

## Confirmed divergence (discovery evidence)

Discovery localization on the K1 classic fixture at layer
`Z:0.4` (first layer past the first-layer gate; one island, outer depth-0 +
inner depth-1 contour walls):

- Upstream fuzzifies the OUTER wall first (same-level first pass), the INNER
  wall second — proven by the outer-wall extrusion multiset being identical for
  168/180 lines between the `allwalls` and `external` seeded outputs (the
  12-line residue is seam-split/wipe positions rotated via the previous-entity
  endpoint, itself downstream of the fuzzed inner wall).
- Ares fuzzifies the INNER wall first (child `Finish` before parent), shifting
  the stream — proven by its outer wall sharing 0/362 extrusion lines with its
  own `external`-mode outer wall.
- Both sides fuzzify the inner wall (decision identical) and the per-call draw
  sequence is byte-correct (`fuzzy_skin=external` and `fuzzy_skin=all` are
  full-stream NULL controls) ⇒ the divergence is exclusively the cross-loop RNG
  consumption order.

## RED (tests-only, this slice)

`crates/ares-cli/tests/orca_parity/fuzzy_allwalls_rng_order.rs` with fixtures
`tests/parity/fuzzy-allwalls-rng-order/{allwalls,external,all,stock}.3mf`:

1. `fuzzy_allwalls_seeded_oracle_ordered_bytes` (RED until GREEN): gated on
   `ARES_PARITY_SEEDED_REFERENCE=1`; double Native replay must be stable and the
   Ares API and CLI outputs must match the seeded Native ordered bytes
   (generator-line-normalized). Frozen input SHA
   `d455419d2782f950d4ffab379a973ca7ba16f1cbf7df871796a3d44b9eefe0e7`.
   Native sanity pinned: layer-1 inner wall unfuzzed (first-layer gate), layer-2
   inner wall fuzzified.
2. Controls that must stay PASS in the same seeded mode:
   `fuzzy_external_seeded_oracle_control`, `fuzzy_all_seeded_oracle_control`
   (loop-0-only fuzzification paths).
3. `fuzzy_stock_native_control`: ungated stock control; official and seeded
   oracles are both deterministic here.
4. `official_fuzzy_allwalls_oracle_is_explicitly_unstable`: official-mode
   handling. Official OrcaSlicer is nondeterministic for fuzzy skin (replays
   differ in real geometry — arc parameters — not just timestamps), so official
   can never gate fuzzy parity. This test runs only when the seeded reference
   flag is absent and a runner is configured, slices the `allwalls` witness
   twice through the configured runner, and ASSERTS the normalized replays
   differ, writing the comparison artifact. It fails if official ever becomes
   deterministic — at which point official ordered-bytes parity must be added
   there instead of re-enabling silent passes. No test in this file may treat
   an unstably-produced official stream as parity evidence.

RED state on main: test 1 fails with the first ordered-byte difference at the
layer-2 inner wall; tests 2-4 pass in their respective modes.

## GREEN constraints

- Change ONLY the classic-path fuzzification call order in
  `project_slice/perimeters/classic/*`: consume the shared fuzz RNG exactly as
  upstream — all same-level siblings (seed vector order) first, then
  chain-ordered recursion into children, per surface. Emission order and the
  `RawPathNode` structure must not change; no output masks; no consumer
  compensation; `should_fuzzify`/`fuzzy_skin_seeded` per-call behavior must not
  change.
- All four tests above must pass after GREEN (seeded RED turns green; controls
  unchanged).
- Arachne (`wall_generator=arachne`, `arachne/fuzzy.rs` +
  `PerimeterGenerator.cpp:387` Arachne application) is AUDIT-ONLY in this slice:
  if the same ordering divergence is confirmed there, propose it separately.

## Deferred

`fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, `fuzzy_skin_first_layer`
probes; `ironing_type` top/topmost/solid; `ironing_pattern`; painted `"none"`
type; ripple/coherent noise types; official-mode fuzzy determinism; arachne
fixtures; multi-island / multi-region fuzzy ordering stress.
