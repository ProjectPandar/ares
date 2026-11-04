# SeamPlacer occlusion object-affine precision

Status: source-cited rewrite slice; separate public RED and narrow GREEN.
Upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Owning source and exact destination

`src/libslic3r/GCode/SeamPlacer.cpp:630,657-671`,
`SeamPlacerImpl::compute_global_occlusion`, gets `po->trafo_centered()`, merges
positive and negative indexed meshes and calls
`its_transform(triangle_set, obj_transform, true)` before sampling.
`Print.hpp:325-328` returns `Transform3d`; `Point.hpp:85` defines it with double.
`deps_src/admesh/stl.h:362-369` evaluates:

```cpp
for (stl_vertex &v : its.vertices)
    v = (t * v.template cast<T>()).template cast<float>().eval();
```

Here T=double: promote the stored float vertex, evaluate the affine with
f64 coefficients/arithmetic, then narrow each completed coordinate to f32.
No FMA, source reassociation, epsilon or precision change elsewhere is included.

Rust destination is ONLY the merged-object transform call in
`crates/ares-core/src/project_slice/seam_placement/mesh.rs::TriangleMesh::from_object`.
Replace its slicing-specific `transform_point_f32` call with the EXISTING
f64 `Transform3d::transform_point`, followed by coordinate `as f32` stores.
The former call is a temporary compatibility shell at this upstream boundary,
not an Ares-owned occlusion or seam policy. No new transform API is needed.

## Non-negotiable boundaries

- Do not modify `transform_point_f32`, its signature or any other reference.
  Slicing's f32 rounding remains intentional (ARD-0076). On this base the
  identifier has16 source occurrences:14 calls (including2 tests), its
  definition and a const signature. Only the mesh occurrence may change;
  the other15 occurrences/files must remain byte-identical.
- Retain vertex/index order, left-handed winding, positive/negative buckets,
  separate decimation, negative split, sampling/RNG, KD traversal, raycasts,
  visibility weighting, scoring, alignment, projection, split and emission.
- No coefficient tuning, visibility sorting, tie epsilon, geometry/config
  mutation, fallback, M73/time/motion/comment/footer masks or expected-failure
  assertion. Browser WASM, Windows, macOS and Linux remain core API targets.
- No global f32-to-f64 helper rewrite or adjacent defect repair. If whole
  output or neighbors leave a residual, stop and report its source boundary.

## Other upstream its_transform sites: audit only

All four literal callsites in `SeamPlacer.cpp` were inspected:

| Source | Existing Rust mapping | Disposition |
|---|---|---|
| :640, per-volume MODEL_PART or NEGATIVE_VOLUME | Same volume loop in `mesh.rs`, `volume.transform().transform_point(vertex)` then `as f32` | Already f64 at this boundary; leave unchanged for both buckets |
| :661, merged positive+negative mesh object transform | `mesh.rs` final object loop currently uses `transform_point_f32` | This call only; it naturally covers the combined mesh, not a new negative-volume branch |
| :723, painted enforcers, double `obj_transform * mv->get_matrix()` | No corresponding painted-facet gather/transform consumer in the current seam mesh/candidate shell | Adjacent unported behavior; no painting or transform implementation in this slice |
| :728, painted blockers, same composed double matrix | Same absent painted-facet gather boundary | Deferred; no edit or parity claim |

This continues the object-stage deferral explicitly recorded in
[the volume-affine slice](seam-occlusion-volume-affine.md). Do not rewrite that
historical volume-only scope or broaden this new object-only scope.

## Accepted causal witness and public seam

Frozen input `tests/parity/seam-occlusion-object-affine/x1c-thin-0.3mf`, SHA256
`eb7bdc6e74029dfa25da954c231142f59eb37da648943a425516f1eb54dd894e`.
Committed C0-neck geometry is reused without any mesh/transform/member changes
except global project configuration conditioned to the official X1C stock.
Classic, ccw, detect_thin_wall0; stored object seam is aligned_back.

Read-only Native/Rust probes establish identical object matrix Tz0.300000012
and float input z-0.30000001192092896. Double-affine-then-float gives Native
bottom Z7.907102750337458e-11; premature coefficient narrowing gives Rust0.
All32 transformed vertices match the respective measured producers;16 bottom
vertices differ.49 seam positions/angles and first4 pre-split loops agree.
Captured pools through unchanged KD methods reproduce all49 respective live
neighbor orders. For first-layer right candidates12/21, membership and every
sample visibility are identical; iteration order alone reproduces the rounded
visibility and champion flip. Three layers do not form a six-seam alignment
string; projection/split receive already different choices.

The clockwise-rectangle deletion was separately replayed and left this
witness's entire raw Ares output unchanged.

Use unchanged `runner::native_stream_case::check -> native_api_cli::NativeCase`:
Native2 stable FIRST, actual validated byte API and bare CLI, complete ordered
`golden::compare_ordered_bytes_generator_only`, raw API==CLI. Register only in
an existing group file; root parity registry and400-line runner stay untouched.

Commit input/test-only RED first and observe genuine strict FAIL in official
and seeded modes. The helper writes both outputs/comparisons before its API
panic; independently verify raw files on RED without claiming its final raw
assertion ran. Then commit production-only GREEN. Require full same-input
PASS and dual-mode seam neighbors, rustfmt and all-target clippy.
Retain all raw replay/cache/process/comparison artifacts outside Git.
