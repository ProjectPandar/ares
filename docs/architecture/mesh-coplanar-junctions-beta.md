# Mesh slicing coplanar junctions and canonical line order

## Source boundary

Rewrite Orca beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`TriangleMeshSlicer.cpp:163-240` (`CoplanarEdges`, `coplanar_edges`),
`:1180-1287` (`chain_lines_by_triangle_connectivity`, including first-line
closure and next-line junction skipping), and `:623-635` (per-layer
`IntersectionLines` canonical tuple sorting).

Rust destinations are `mesh_slicer` topology/coplanarity, raw intersection
finalization, and connectivity chaining. Existing unconditional junction
insertion is temporary scaffolding replaced by the source's geometric-plane
edge classification. This is not generic collinear simplification.

## Evidence and public seam

Both retained lightning and raft witnesses have 34 vertices and 64 facets.
All 32 sliced lines agree with Native by facet, endpoint coordinates, and
vertex/edge references. Native canonically orders those lines and omits
triangulation-only coplanar junctions, yielding 16 polygon vertices; Rust
retains 32. The later union, closing, DP, materialization, and seam stages
are not the first divergence. Actual slicing Z is already f32 on both sides.
External evidence retained outside the repository.

Public byte-in/API-out tests compare the complete ordered raw Outer-wall
motion record sequences against qualified Native double replay for the two
retained projects. No coordinate normalization, collinear merging, unordered
acceptance, or Native output golden is permitted. Existing complete-output
comparators remain unchanged.

## Non-negotiable decisions

1. Construct coplanar edges from the same face-edge identities and the
   upstream transformed plane/opposite-vertex test, including orientation
   and one-coordinate-unit distance tolerance. Thread that provenance to
   chaining; never reconstruct it from emitted XY collinearity.
2. Port coplanar junction omission first, then canonical line sorting in a
   separate commit. Preserve the first-line closure handling as well as the
   next-line handling.
3. Preserve geometric shallow corners, open-chain behavior, platform-neutral
   byte APIs, and all existing complete-output acceptance rules.
4. No DP, seam, G-code loop emission, or independent pipeline redesign is
   included. Adjacent slab/copanar-facet behavior is changed only where the
   shared source-owned chaining contract requires its provenance.

## Verification

Observe both public witnesses RED before production edits. Record each
port's witness results, then compare the same mesh/geometry/walls/fill parity
inventory before and after. Report every FAIL-to-PASS and PASS-to-FAIL; no
unexplained regression is acceptable. Run rustfmt and all-target Clippy with
warnings denied. Keep changed modules below 400 lines.
