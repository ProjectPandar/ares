# Minimum spanning tree ties broken by coordinates

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `MinimumSpanningTree::prim` (`MinimumSpanningTree.cpp:32-109`) grows the tree from the
  first vertex. Each step takes the vertex that is not yet in the tree and is closest to it
  (`:66-77`):
  - the distance is `vsize2_with_unscale` (`:16-25`), the squared length in millimetres as a
    `double`, with each coordinate unscaled by `SCALING_FACTOR` before squaring;
  - `std::min_element` compares the distances with `<`. When two distances are equal, it
    compares the points with `Point operator<` (`Point.hpp:272-275`), x first and then y.
  - The map holding the candidates is keyed by address. Without the coordinate tie-break,
    the first minimum would depend on where the vertices were allocated.
- `TreeSupport::drop_nodes` (`Support/TreeSupport.cpp:3034-3054`) builds one tree per group
  of nodes in a layer. A node moves towards the average of its tree neighbours
  (`:3248-3286`), so the edge chosen between two of the same length decides where the branch
  goes.

## Rust destination

- `crates/ares-core/src/project_slice/support_material/tree/non_organic/spanning_tree.rs`
  (`MinimumSpanningTree::new`): today, among equal distances, the vertex chosen is the last one
  inserted. This followed the hash-map order of the older source. The minimum is now taken by
  the key `(distance, x, y)`:
  - `distance` is the existing `distance_squared`, which already matches
    `vsize2_with_unscale`;
  - distances are compared with `total_cmp`. They are squares, so never negative zero or NaN,
    and `total_cmp` orders them as C++ `<` does.

## Included

- The coordinate tie-break of the closest-vertex search (`:66-77`).

## Deferred

- Nothing else in `prim`, `adjacent_nodes` or the node movement changes.

## Premises

1. `runner::tree_support::manual::vertical::manual_slim_nested_vertical_painting_preserves_complete_output`
   and `runner::tree_support::manual::manual_slim_painted_overhang_preserves_complete_output`
   fail in both modes.
2. In the nested vertical case, the vertical enforcer points and the contact nodes equal
   Native's. The first different node is dropped from layer 24 to layer 23:
   - the layer 24 nodes lie on a grid, so the tree has two candidate edges of the same length;
   - Native takes the edge whose vertex is smaller by x, then y. The node then moves towards
     one neighbour;
   - Ares takes the other edge. The node then moves towards the average of two neighbours.
   The branch, the support below it, the skirt and the time estimate follow from that
   position.
3. Before writing this spec, I tried the tie-break as a temporary probe, since reverted. In
   both modes, both tests went from FAIL to PASS. Across 204 tree support, support, painting,
   enforcer, blocker and organic tests, nothing else changed.

## Observable behavior

When support nodes in a layer are equally far apart, tree support branches merge along the
same edges as in Native.

## Failure modes

- Distance computed differently: it must stay the unscaled squared length in `f64`, not a
  squared length in scaled integers. Otherwise rounding changes which distances are equal.
- Wrong coordinate order: x is compared first, then y.
- Tie-break applied to the distance update: the update keeps a new distance only when it is
  strictly smaller (`:96-105`). That stays as is.

## Test plan

The witnesses already exist as whole-output parity tests, are named by behavior, and fail at
the base in both modes:

| Test | Before the port | After the port |
|---|---|---|
| `runner::tree_support::manual::vertical::manual_slim_nested_vertical_painting_preserves_complete_output` | F | P |
| `runner::tree_support::manual::manual_slim_painted_overhang_preserves_complete_output` | F | P |

This slice adds no new test commit. The full run in both modes must turn both tests into PASS
with no PASS turning into FAIL.
