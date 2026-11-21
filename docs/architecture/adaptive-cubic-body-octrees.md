# Adaptive Cubic octrees for separated bodies

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) uses `PrintObject.cpp:755–785` and `ConnectedBodies.cpp:27–89` to group overlapping islands of adjacent layers into bodies. `PrintObject.cpp:1115–1166` maps a point to its island's body (or the nearest within 1 mm), and assigns each connected mesh component by sampled triangle votes. `PrintObject.cpp:1169–1257` prepares octrees from the resulting body meshes, with an object-wide entry when needed. Each tree uses its region's line spacing (`Fill/FillAdaptive.cpp:304–323`) and builds in the centered object frame. `Fill/FillAdaptive.hpp:43–60` selects the body tree before the object-wide entry and then falls back to the first available body tree. `Fill/Fill.cpp:1353–1559,1580–1685` uses the selected tree for both printed fill and sparse bridge anchors.

An octree built over the whole object gives disconnected rings a different lattice phase. In the compound coupon, the Native first broad ring at Z0.8 retains all 32 undirected sparse segments from the same ring sliced alone. Both implementations agree on the isolated ring, but the whole-object tree's output contains none of those 32 segments. The first divergent input is the mesh selected during octree preparation, not path reversal, connection or the spacing formula.

## Rust destination

Port island connectivity, mesh component assignment, and per-body Adaptive Cubic tree selection to `project_slice/fill_entities/adaptive` and its use from the sparse anchoring and fill generation call sites. Reuse the existing Adaptive Cubic per-region spacing and octree primitives. The object-wide cache entry remains the upstream path for single-body objects and unmatched surfaces; the object-only cache is a temporary compatibility shell for the Support Cubic path. Keep coordinates in the same centered object frame as the call sites, and route bridge overhang triangles at their source layers to the corresponding trees.

Do not change Support Cubic preparation or spacing in this slice: it has its own overhang-only triangle selection and bridge payload path. Also defer the other consumers of `ConnectedBodies` and changes to line generation, multiline routing, hooks or G-code ordering.

## Public behavior

For the same synthetic compound project on the Bambu Lab X1 Carbon, Creality K1 and Prusa MK4, slice the stock and Adaptive Cubic configurations with Native and the Rust public API and CLI. The Native sparse moves must change when Adaptive Cubic is selected, and the complete files must agree with the pinned Native release after normalizing only the generator line. The stock configuration must retain complete-file parity. Native replay must be stable in the official and seeded modes.
