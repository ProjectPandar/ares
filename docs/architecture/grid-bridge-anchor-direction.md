# Preserve Grid direction in internal bridge anchors

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r`:

- `PrintObject.cpp:3839` calls `Layer::generate_sparse_infill_polylines_for_anchoring`. In `Fill/Fill.cpp:1609–1617`, the function sets `Fill::layer_id` to the layer ID minus the first object layer's ID, excluding raft layers; at `1674–1679` it calls the pattern's `fill_surface` for each surface.
- `Fill/FillRectilinear.cpp:3525–3547`: ordinary Grid runs two sweeps and connects the results, then at `3543–3545` reverses each resulting polyline on odd `Fill::layer_id` without changing polyline order. Trapezoidal Grid (`multiline > 1`) does not take the reversal branch. Grid's sweep angle itself does not alternate (`FillRectilinear.hpp:84–86`).
- `PrintObject.cpp:3480–3547,3839,3942`: clipped directed lower-layer segments vote on the automatic bridge angle. Exactly horizontal segments oriented toward +X and −X may produce bridge angles separated by π. They define the same undirected axes but not the same monotonic traversal (`Fill/FillBase.cpp:446–453`, `Fill/FillRectilinear.cpp:2869–2879`).

## Rust destination

The bridge transaction's `transaction/candidate_expansion/layers.rs` requests polylines from the layer below each candidate. `sparse_anchoring/mod.rs` passes the zero-based object `layer_index` and the planned `layer_id` to `project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`. The plan (`project_slice/layers.rs`) assigns `layer_id = layer_index + base_raft_layers + interface_raft_layers`, so subtracting the first object layer's ID leaves exactly `layer_index`. Use that index for the ordinary Grid reversal, not the absolute planned ID or print height. The existing trapezoidal dispatch still receives the planned ID. Apply it before passing the polylines to the bridge preparation stage, not while clipping anchors, reducing angle votes, or ordering extrusion entities. The ordinary Grid extrusion path already applies the same upstream reversal. Preserve its behavior. Leave trapezoidal Grid, other fill patterns and G-code chaining unchanged.

## Observable behavior

Compare directed internal-bridge extrusion from a synthetic multi-feature plate with Grid at an infill direction of 90 degrees against the same Orca-sliced project. Reversing a bridge with the same undirected segments must not pass. Exercise the setting against the stock project and check the same setting with other machine profiles. Compare complete ordered output without changing the comparator. On the same X1 Carbon plate, print three raft layers to make the absolute and object-relative IDs have opposite parity. Use normal raft generation with arc fitting disabled so the comparison isolates Grid anchoring from tree raft generation and support-interface arc fitting. Require actual raft extrusion, directed bridge equality at the raised bridge height, and complete output equality.
