# Slab lines chained in canonical order

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `slice_mesh_slabs` (`TriangleMeshSlicer.cpp:2197`) builds the slab lines with
  `slice_slabs_make_lines` (`:1014-1095`), then chains them per slab with `make_slab_loops`
  (`:1668`) and `chain_lines_by_triangle_connectivity` (`:1180`).
- `slice_slabs_make_lines` fills the lines in a parallel loop over the facets, so their order
  inside a slab depends on thread scheduling. It then sorts every list canonically
  (`:1077-1093`):
  - the lists sorted are `at_slice` and `between_slices` of both `lines_top` and
    `lines_bottom`;
  - the key is `(edge_a_id, edge_b_id, a_id, b_id, a.x, a.y, b.x, b.y, edge_type, flags)`,
    where an endpoint carries either a vertex id or an edge id and the other is `-1`;
  - `make_slab_loops` takes its seed lines and its candidates for each vertex or edge in that
    order, so the order decides how loops that touch at one vertex are chained.
- `slice_make_lines` sorts the plain slice lines with the same key (`:622-633`). That sort is
  already ported in `crates/ares-core/src/mesh_slicer.rs` (`slice_mesh_on_planes`).
- Callers whose output changes:
  - `PrintObject::project_and_append_custom_facets` (`PrintObject.cpp:5491`) projects the
    painted support enforcers and blockers through `slice_mesh_slabs`. Tree support
    intersects the enforcers with each layer's new overhang, grows them by the tip diameter
    and subtracts the layer below (`Support/TreeSupport.cpp:1276-1281`). The contact points
    are then placed on that shape;
  - the MMU painted top and bottom projection (`MultiMaterialSegmentation.cpp:1170-1284`)
    also uses `slice_mesh_slabs`.

## Rust destination

- `crates/ares-core/src/mesh_slicer/slabs.rs`: after the facet loop, `slice_mesh_slabs`
  sorts each `at_slice` and `between` list of the top and bottom lines before `make_loops`.
  Today it chains them in facet order.
- The sort key now used by `slice_mesh_on_planes` (`mesh_slicer.rs`) moves into one function
  on `IntersectionLine` in `mesh_slicer/intersection.rs`. Both sorts call it; the key is not
  copied.
- The `edge_type` tie-break uses upstream's `FacetEdgeType` ordinals:
  - plane lines pass their own type;
  - `at_slice` slab lines pass their `General`, `Top`, `Bottom` or `TopBottom` kind;
  - `between` lines pass `Slab`.
  `flags` stays 0, because none are set before the sort.

## Included

- The canonical sort of the four slab line lists (`:1077-1093`).

## Deferred

- Nothing else in `slice_mesh_slabs`, `make_slab_loops` or the chaining changes.

## Premises

1. `runner::tree_support::manual::split_sides::manual_slim_one_edge_side_zero_preserves_complete_output`
   fails in both modes. Its sibling `side_one`, which has the same mesh and paints the other
   half of the split face, passes.
2. The side zero enforcer facets are two triangles of the overhang's bottom face. They share
   only one vertex.
   - In facet order, chaining starts on an edge that ends at the shared vertex, closes the
     first triangle, and then the second one: two loops.
   - In canonical order, the first line starts at the shared vertex, so the chain runs around
     one triangle, back through the shared vertex, and around the other: one loop through
     that vertex twice.
3. Tree support grows the enforced overhang by 0.8 mm:
   - from two separate pieces, each acute corner at the shared vertex grows into a miter
     spike;
   - from the single loop, the outline there is one right-angled corner.
   A contact point sits on that corner. Native's topmost support interface has its branch
   circle centred on the right-angled corner, while Ares's is centred on the spike tip. This
   moves the branch, the support below it and the first-layer skirt.
4. Before writing this spec, I tried the sort as a temporary probe, since reverted. In both
   modes, `side_zero` went from FAIL to PASS and `side_one` stayed PASS. Across 64
   tree-support, support-paint, enforcer, blocker and MMU-painted tests, nothing else
   changed.

## Observable behavior

Painted support enforcers whose facets touch at a single vertex project to the same loops as
in Native. The tree support contacts, branches and skirt that follow from them equal
Native's.

## Failure modes

- A list left unsorted: all four lists must be sorted, top and bottom, at-slice and between.
- Sorting too late: the sort must run before the top-plane edge ids are raised by the edge
  count in `make_loops`, as upstream sorts before `make_slab_loops`.
- A wrong endpoint key: an endpoint keyed by a vertex has edge id `-1`, and one keyed by an
  edge has vertex id `-1`. This matches the plane sort.
- MMU painted projections share the function. Their loops change only where the old order
  chained touching loops differently.

## Test plan

The witness already exists as a whole-output parity test, is named by behavior, and fails at
the base in both modes:

| Test | Before the port | After the port |
|---|---|---|
| `runner::tree_support::manual::split_sides::manual_slim_one_edge_side_zero_preserves_complete_output` | F | P |

This slice adds no new test commit. The full run in both modes must turn this test into PASS
with no PASS turning into FAIL. The results of the painted and MMU projection tests are
listed separately.
