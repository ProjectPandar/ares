# Raft-only footprint for non-Organic tree support

## OrcaSlicer 2.5.0 boundary

`PrintObject::generate_support_material` admits a nonempty object with a raft
independently of object support (`PrintObject.cpp:998–1002`). Its generator
dispatch sends tree support types to `TreeSupport::generate`
(`PrintObject.cpp:5088–5093`). Organic returns through the 3D generator;
Slim, Strong and Hybrid take the 2D path (`Support/TreeSupport.cpp:1901–1950`).
`create_tree_support_layers` creates the raft prefix even with object support
disabled (`Support/TreeSupport.cpp:1333–1378`).

For this 2D path, `TreeSupport::generate_toolpaths` offsets the first object
layer's `lslices` together with the floor, roof and base areas of the first
non-raft support layer, when present. It applies `raft_first_layer_expansion`
to the whole area, then applies the same expansion again for the first base
layer only. The first base wall follows that second offset; its fill is
inset by half the flow spacing (`Support/TreeSupport.cpp:1557–1603`). This is
not the normal support contact grid or the Organic raft base.

## Rust destination

`project_slice::support_material::project` admits tree raft-only objects to
its existing 2D or 3D generator according to the resolved style. The 2D
prefix is owned by `support_material::tree::non_organic::raft_layers` and
`support_material::tree::non_organic::toolpaths::raft`, not by the normal
raft-only fallback in `project_slice::raft`. That fallback is a compatibility
shell for the normal branch of `SupportMaterial`, not a source of raft geometry
for tree styles. Reuse the installed support layer pipeline; do not specialize
G-code bounding boxes or estimates to compensate for a footprint difference.

This slice covers raft-only dispatch and its first-layer footprint; leave the
Organic path, normal raft, support collision volumes, and arc fitting outside
its scope. Compare the entire G-code stream, normalizing only the generator
line, against Native for synthetic X1C and MK4 prisms. Each has three raft
layers, object support and arcs disabled, with Slim, Strong and Hybrid
selected separately.
