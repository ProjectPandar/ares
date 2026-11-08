# Lightning per-tree polyline walk

## Rewrite boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream | Behavior | Rust destination |
| --- | --- | --- |
| `Fill/Lightning/TreeNode.cpp:358-369` (`Node::convertToPolylines(Polylines&, coord_t)`) | every call on a tree root starts its own `std::mt19937_64`, seeded from `uint64_t(PointHash{}(m_p))` | `fill/lightning/tree_geom.rs` (`convert_to_polylines`) |
| `TreeNode.cpp:372-388` (recursive overload) | at every internal node, the first child is `rng() % m_children.size()`, and the other children start new polylines | `tree_geom.rs` (`convert_rec`) |
| `TreeNode.hpp:272` | the recursive overload takes `std::mt19937_64&` | `tree_geom.rs` |
| `Point.hpp:505-508` (`PointHash`) | `coord_t((89 * 31 + int64_t(x)) * 31 + y)` | `fill/lightning/distance_field.rs` (`point_hash`) |
| `Fill/Lightning/Layer.cpp:450-459` (`Layer::convertToLines`) | walks each tree root, then clips to the outline; takes no generator | `fill/lightning/layer.rs` (`convert_to_lines`) |

This is a port of the upstream polyline walk, not a new infill feature. The
process-wide `rand()` stream in the Rust port keeps the shape of the earlier
upstream baseline, and it is removed rather than kept beside the ported walk.

## Measured owner

Fixture: Tree Strong with `support_base_pattern = lightning` on the overhang model.
It differs from a passing Tree Strong case only by that setting. Same-layer captures
were taken at `TreeSupport.cpp:2708` (generator inputs) and `:1833` (polylines from
`convertToLines`, before the length filter), read from Native through the debugger on
the existing seeded binary.

- `contours` and `overhangs` match on every layer, so the support areas are not
  involved. `support_density` is optimized out in that binary and was not compared.
- The polylines have the same segments and the same count on every layer, and differ
  in how the segments are stitched on all but one. The lightning trees agree; only the
  walk through their junctions differs.

With only the walk changed, the polylines match on every layer before and after
chaining, and the whole G-code stream matches Native, including `M73`.

## Rust divergence

| Site | Rust today | Upstream |
| --- | --- | --- |
| `tree_geom.rs:13-22` | takes a shared `GlibcRand` | seeds one generator per tree root |
| `tree_geom.rs:38` | `rng.next()` is glibc `rand()` | `rng()` is the 64-bit Mersenne Twister |
| `layer.rs:165,172` | threads the shared generator through every tree | no generator parameter |
| `generator.rs:41,99,168,285` | one `RefCell<GlibcRand>` shared by all layers and callers | no generator state |

## Change

1. Seed `Mt19937_64` from `point_hash(root)` inside `convert_to_polylines`, and walk
   with it, as `TreeNode.cpp:366-367` does.
2. Remove the `rng` parameter from `Layer::convert_to_lines` and the `rng` field from
   `Generator`.
3. Delete `fill/lightning/glibc_rand.rs`. Nothing else uses it:
   `group_fills/params/projection/rotation/meta.rs` has its own `GlibcRandom`, which
   owns a different upstream `rand()` use and stays.
4. Reuse `fill/rectilinear/rng.rs::Mt19937_64` by widening its visibility and adding
   a seeded constructor. The rectilinear default seed does not change, and no second
   generator type is added.
5. Update the `TreeNode.cpp` and `TreeNode.hpp` rows of
   `2026-10-30-upstream-baseline-v2.5.0-beta-inventory-fill-support.md`: the status
   stays `M`, and the scope records the polyline walk as updated to the target while
   the rest of the file stays a bounded earlier slice.

The generator moves inside the walk, so all three callers of
`Generator::convert_to_lines` take the per-tree walk with no call-site change:

- `support_material/tree/non_organic/toolpaths/base.rs` (`TreeSupport.cpp:1832`),
- `project_slice/fill_entities/lightning.rs` (`FillLightning.cpp:26`),
- `prepare_infill/bridge_over_infill/sparse_anchoring/arms_tail/lightning.rs`
  (`FillLightning.cpp:26`).

The last one runs before emission and used to advance the shared stream, which moved
the picks of every later call. With no state crossing trees, the output of one tree no
longer depends on which layer, caller or thread walked another tree first, which is
why upstream made this change (`TreeNode.cpp:363-365`).

## Included and deferred

Included: the walk and its seed for all lightning output, sparse infill and Tree
support alike.

Deferred, each with no failing case in this slice:

- the rest of the `Fill/Lightning/*` rows of the inventory, which stay bounded earlier
  slices,
- `lightning_initial_overhang_polygons_a1`, which fails with the same first difference
  with and without this change, at a position before any lightning output.

## Failing cases and acceptance

No test code is added. Both failing cases already replay through the public 3MF path
and compare the complete stream with a Native reference produced at runtime, and
neither stores a coordinate or a count:

- `strong_tree_lightning_overhang_preserves_complete_output`, for the Tree caller.
  It needs the Tree roof-layer counter port, so this change stacks on it.
- `lightning_preserves_complete_ordered_output`, for the sparse and anchoring callers.
  A temporary panic in the anchoring caller confirmed that this case reaches it.

Acceptance, in official and seeded modes:

- both cases fail on the unchanged tree and pass with the change,
- both cases fail again when the seed is offset by one, so they pin the seed and not
  only the generator type. With a constant seed only the Tree case fails, so the Tree
  case is the one that separates per-tree seeding from a fixed seed,
- the `lightning | tree | support | ironing_pattern | coplanar_outer_wall` selection
  moves no test from pass to fail,
- the Tree and sparse fixtures produce byte-identical output at 1, 2, 7 and 16 cores.

The lightning path has no parallel code in Rust today, so the core-count check guards
future parallel work and does not by itself prove the layer-order property.

Implementation order: record both failures, apply the change, rerun both modes, then
run the full suite.
