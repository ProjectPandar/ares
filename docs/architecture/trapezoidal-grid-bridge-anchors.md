# Trapezoidal Grid bridge-anchor layer phase

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) dispatches Grid with `multiline > 1` to `FillRectilinear::fill_surface_trapezoidal` with pattern type 0 (`Fill/FillRectilinear.cpp:3525–3533`). The filler divides its layer ID by the surface's `thickness_layers` when positive (`:3134`). Odd effective IDs transpose the row coverage and the generated points about the grid center (`:3176–3177`, `:3230–3233`). The same parity selects the row axis along which closed outlines choose a starting point (`:3440–3446`). The `% 3` phase at `:3136–3137` is for the other pattern types, not Grid.

The two callers intentionally supply different IDs:

- `Layer::make_fills` sets `f->layer_id = this->id()` (`Fill/Fill.cpp:1378`). Printed sparse Grid retains the physical layer ID, including the raft offset. In Ares this is `PlannedLayer.id`, forwarded by `fill_entities/generate.rs` through `fill_entities/grid.rs`.
- `Layer::generate_sparse_infill_polylines_for_anchoring` subtracts the first object layer ID (`Fill/Fill.cpp:1611`) and then calls `fill_surface` (`:1678`). Ares assigns planned IDs as `index + raft_layers` (`project_slice/layers.rs:240`), so its object layer index is already this exact difference. The anchoring Grid arm must pass that index before thickness-layer division, not the absolute ID or a parity derived after division.

Three raft layers with uncombined sparse infill reverse the incoming phase if the anchoring call uses the physical ID. That changes the anchor segments consulted for the bridge direction, without changing the printed Grid phase. A Native-backed comparison of the same plate with normal raft, two-line Grid and arc fitting disabled checks the resulting bridge motion and full G-code. The printed Grid call, other patterns, and ordinary Grid's post-connection orientation remain unchanged.
