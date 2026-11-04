# Green plan: concentric ironing ring order (`FillConcentric::_fill_surface_single`)

Status: PLAN. No production change in this document.
Boundary pinned by RED `944c70418` on main `d592d4145`.

## Upstream rules (source-cited, all verified in this tree)

`FillConcentric::_fill_surface_single` (`OrcaSlicer/src/libslic3r/Fill/FillConcentric.cpp:11-77`):

| line | rule |
| --- | --- |
| `:20-24` | `min_spacing = scale_(spacing) * multiline`; `distance = coord_t(min_spacing / density)` — integer domain, float division then truncation |
| `:26-29` | only when `density > 0.9999 && !dont_adjust`: `distance = _adjust_solid_spacing(bbox.x, distance)` and `this->spacing = unscale(distance)` |
| `:31-32` | contract `offset_ex(expolygon, -0.5*(multiline-1)*spacing)` (float mm → scaled inside) |
| `:34-38` | ring set: `loops = to_polygons(contracted)`, then repeat `last = offset2_ex(last, -(distance + min_spacing/2), +min_spacing/2)` and append `to_polygons(last)` until empty — outside-in |
| `:40` | **ring order**: `loops = union_pt_chained_outside_in(loops)` |
| `:43-48` | **ring start point**: `last_pos = (0,0)` initially; for each loop `polylines_out.emplace_back(loop.split_at_index(last_pos.nearest_point_index(loop.points)))`, then `last_pos = polylines_out.back().last_point()` |
| `:50-51` | `multiline_fill(polylines_out, params, spacing)` (mm spacing; offsets by multiline overlap) |
| `:54-66` | `clip_end(loop_clipping)`, drop non-valid paths, compact in place |

Ordering chain behind `:40`:

- `ClipperUtils.cpp:1012-1017` `union_pt_chained_outside_in` = `traverse_pt_outside_in(union_pt(subject).Childs)`
- `ClipperUtils.cpp:992-1010` `traverse_pt_outside_in`: build `ordering_points` from each node's `Contour.front()`, order with `chain_clipper_polynodes(ordering_points, nodes)`, emit each `Contour`, `reverse()` holes (to CCW), then recurse into `Childs`
- `chain_clipper_polynodes` (endpoint greedy chaining over those points) is the same shortest-path family as `ShortestPath.cpp`'s greedy chain

Direction: closed rings are emitted as-is except holes, which are reversed to CCW (`:1005-1006`). There is no per-ring reversal for solid rings at this stage.

## Ares destination

- `crates/ares-core/src/fill/concentric_sparse.rs:48` `fill_concentric_sparse` — the whole body already mirrors the table above:
  `:61-63` min_spacing/distance, `:66-81` solid spacing adjust, `:83-94` contract, `:95-113` offset2_ex ring set, `:117` `union_contours(&loops)`, `:118-140` split at `nearest_point_index(last_pos)` and `last_pos` chaining, `:141-147` multiline, `:148-150` `clip_end` + validity retain.
- `crates/ares-core/src/geometry/clipper/polytree.rs:97-129` `outside_in` mirrors `traverse_pt_outside_in`: chain over contour first points (`chain_points`, `:100-108`), emit in that order, `reverse()` holes (`:121-124`), recurse children (`:126-127`).
- `crates/ares-core/src/geometry/chain_points.rs:51` `chain_points` is a KD-tree endpoint greedy chain, the analogue of `chain_clipper_polynodes`.
- Caller: `crates/ares-core/src/project_slice/fill_entities/ironing.rs:118-167` (Concentric arm) calls `fill_concentric_sparse` and stores results in order with `no_sort: true`.

## What the evidence already establishes

Ares is not missing a documented step: every row of the upstream table has a counterpart. Measured on the diverging layer (`Z_HEIGHT: 52`): one area, 182 closed rings in one `no_sort` collection, chain call `n = 1` (nothing reorderable at emission), both oracles 2537 moves / 1474 ironing segments / 10 gaps > 0.35 mm, but different maximum ring-to-ring link (Native 42.910 mm vs Ares 34.000 mm), and 23 paired 15 mm run-segmentation offsets per side. So the ring set is the same and the order differs inside the collection.

## Step 0 (must run first): locate the divergence inside the seam

Before changing any code, one instrumented run decides where the order first differs, so GREEN is not guesswork:

1. Dump, in `fill_concentric_sparse`, the ordered ring first points (a) as produced by `union_contours` and (b) after the `split_at_index` loop, for the `Z_HEIGHT: 52` layer only (temporary probe, reverted, `git diff` empty).
2. Compare (a) against the Native G-code ring order for that layer. If (a) already differs, the divergence is in `union_contours` / `outside_in` / `chain_points` (ordering of the union). If (a) matches and (b) differs, it is the `last_pos` / `nearest_point_index` split rule.

This is the same probe discipline used for discovery, and it is the only way to avoid a speculative rewrite of a 150-line geometry function.

## Included in the GREEN

- Whatever Step 0 identifies, ported faithfully from the cited line range: either the union ordering (`ClipperUtils.cpp:992-1017` + `chain_clipper_polynodes`) or the split/`last_pos` rule (`FillConcentric.cpp:43-48`).
- Keep all existing invariants: integer `coord_t` domain, `no_sort: true`, one collection per ironing area.
- Make the frozen RED `concentric_ironing_ring_order_matches_native_public_slicing` pass in both oracle modes, plus the whole control set green with no regressions.

## Deferred (explicitly not in this GREEN)

- The Arachne/`ThickPolylines` overload (`FillConcentric.cpp:79-150`) and `FillConcentricInternal.cpp` — not reachable from ironing.
- `is_infill_first` per-region semantics: already byte-identical to Native on `two-region-prism.3mf`; not a gap (measured this window).
- The separate unmerged per-region speed emission (`GCode.cpp:6136/6163`) must not be duplicated here.
- Any change to the clipper kernel itself (`geometry/clipper/*`) beyond what Step 0 proves necessary.
- 3mf fixture regeneration: the frozen input SHA is pinned by the RED.

## Risk note for sizing

If Step 0 points at `chain_points`/`outside_in`, the change is small but touches a shared clipper helper used by brims (`brims/ears/paths.rs:46`) — that would need its own regression consideration. If it points at the split rule, the change is local to `concentric_sparse.rs`. I will report Step 0's answer before editing anything.
