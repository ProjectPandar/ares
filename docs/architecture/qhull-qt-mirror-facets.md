# Mirror facets in Orca's 3D Qt hull

## Source boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) builds its ordered mesh in `src/libslic3r/TriangleMesh.cpp:1358–1434` using reentrant Qhull with `Qt` in dimension three. `deps/Qhull/Qhull.cmake:2–7` pins Qhull `v8.0.2.zip` (2020.2), SHA-256 `a378e9a39e718e289102c20d45632f873bfdc58a7a5f924246ea4b176e185f1e`. The following paths are relative to the verified archive's `src/libqhull_r/`:

- `io_r.c:1109` enters `poly2_r.c:3482–3637` (`qh_triangulate`). `poly2_r.c:3671–3732` and `poly_r.c:621–691,818–1032` create and link the ordered children of each nonsimplicial facet, including those with the apex repeated, before the deletion passes.
- `poly2_r.c:3514–3543,3816–3823` removes the null children: join their neighbors in slots zero and one. `poly2_r.c:3745–3768` replaces reciprocal neighbor references in place. An already mutual link schedules a mirror pair on the degeneracy stack (`merge_r.c:471–480`).
- `poly2_r.c:3532–3543,3781–3800` pops mirror pairs last-in-first-out. For each same-index neighbor slot, skip mutual links inside the pair and already deleted neighbors, otherwise reconnect the exposed neighbors. The reconnect can schedule another pair. `merge_r.c:5524–5541` (`qh_willdelete`) removes the deleted facets from the participating topology and clears their ridge and neighbor sets. Finally, `poly2_r.c:3551–3626` discards deleted facets, leaving the other facets in order.

## Rust boundary

In `ares-core/src/geometry/quickhull/qt`, extend the 3D Qt output pass to model the facet neighbors needed during null and mirror deletion, instead of the scaffold that drops null children immediately and detects mirrors only after packing the triangle buffer. Match and reconnect neighbor slots before collecting the mesh. Do not remove triangles by a geometric tolerance or unordered equality alone. The order of untouched simplicial facets and surviving children must not change; each child's plane is still its parent's plane. Rust does not need Qhull's pointer ownership transfer for the normal or centrum once planes are stored by value. Preserve first-encounter vertex order, the original winding calculation, and normal bits.

Cone `qh_matchdupridge` and `qh_forcedmerges` remain explicitly unsupported (`HullError::DuplicateRidge`), as do other Qhull dimensions and options. Do not introduce fallback hulls, new consumer behavior, or FFI. An invalid final topology remains a typed error rather than an unchecked mesh.

## Verification

Exercise `--inspect-mesh` with a synthetic STL that triggers the Native mirror branch and a control whose inspection is unchanged. Check replay, exit, output inventory, stderr and exact stdout in official and seeded modes via `crates/ares-cli/tests/orca_parity/case_support.rs`. Re-run the affected hull, orientation, and inspection cases on the starting and resulting states in both modes. In the ordered-point corpus compare all 20,000 records against the pinned Native Qhull, including the six successful inputs blocked by mirror rejection, without masking vertex order, triangle order, winding or normal bits. Retain rustfmt, workspace Clippy with all targets, and core/WASM compilation checks.
