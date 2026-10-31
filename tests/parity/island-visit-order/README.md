# Four-island default-tour input

`four-disjoint-prisms.3mf` is one model part and one instance containing four
separate closed 10 × 10 × 1 mm prisms. Their local XY centers are
`(-20, -20)`, `(20, -17)`, `(17, 20)`, and `(-17, -5)` mm. All four intersect every
object layer, with disjoint positive-area horizontal sections and no holes.
They therefore contribute four chainable islands to the default per-filament
tour, reaching the closed-cycle DP rather than the `n <= 3` identity shortcut.
The asymmetric layout avoids relying on equal-length tour choices.

The approved nested-hole-island input supplies archive packaging, configuration
and the instance transform only. The mesh is replaced with explicit outward-
wound prism surfaces. Settings remain classic/two walls, zero infill and shells,
no brim/skirt/support, aligned seams and default by-layer print order. No existing
fixture or generator is changed. Known named island controls inspected before
adding this input covered fewer islands or unrelated brim/seam behavior.

Regenerate from the repository root:

```sh
python3 tests/parity/island-visit-order/generate.py
```

`provenance.json` records both input hashes, mesh sizes, disjoint component data
and the closed directed-edge invariant. ZIP timestamps and member order are
fixed. The generator reads no oracle output.

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`GCode.cpp:7238–7362` and `:7680–7720`,
`ShortestPath.cpp:1624–1650::chain_points_with_postprocessing`, and
`GCode/OrderingStrategies.cpp:230–266::tsp_exact_cycle`.

The public `island_visit_order` case requires runtime official Native replay
stability and at least four inner/outer wall groups in the first complete object
block. The static input establishes four disjoint sections; the runtime assertion
confirms their perimeter emission. Expected G-code is fetched from Native each
run; no Native streams, goldens or output fragments are committed. Both full API
bytes and the complete compared blocks are retained as runtime artifacts.
