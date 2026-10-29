# Occlusion-mesh negative volumes and short-edge-collapse decimation

Deferred gaps from the mirrored-instance winding slice (ebc7eafa4 spec
amendment): `SeamPlacer::compute_global_occlusion` builds the raycast
occlusion mesh from MODEL_PART **and** NEGATIVE_VOLUME meshes and decimates
both with `its_short_edge_collpase` before visibility sampling. Ares'
`seam_placement` implements neither.

## Witness (RED outcome 2026-10-19)

The negative-volume leg was DROPPED after two attempts (evidence below);
the decimation leg froze as `target-decim.3mf` (RED DIVERGENT).

### Negvol attempts (dropped — no Native-active divergence)

- Attempt 1 (plate + corner-carving `negative_part`, geometry-identical
  slicing): under the canonical harness environment native picks the same
  seam as ares and the seeded reference is byte-identical to ares
  (header-only diff); under single-CPU manual runs native picks the other
  corner. The visibility delta is a near-tie whose resolution is
  execution-environment dependent, not a stable native behavior ares fails
  to reproduce.
- Attempt 2 (L-prism + floating negative box filling the carved quadrant,
  no geometry intersection): native's own output is byte-identical WITH and
  WITHOUT the negative volume (0 non-header diff lines) — the parity
  raycast has no observable effect on this witness's seam pick; ares
  matches native byte-identically.

Per the approval condition (no behavior without a RED), the
  NEGATIVE_VOLUME occlusion set and the parity raycast branch are NOT
  ported in this slice and stay deferred.

### Decimation leg (frozen)

`target-decim.3mf` (built from control.3mf with the plate mesh swapped
and `gcode_label_objects=0`): two 30×30×1 plates whose faces are
grid-subdivided to 20720 triangles each (>
`fast_decimation_triangle_count_target` 16000), flat faces only so
slice geometry is parity-stable. `gcode_label_objects=0` because the
native label id comes from uninitialized memory
(`exclude_object=0` leaves `PrintObject::m_id` unset;
`; printing object ... id:` reads it at GCode.cpp:5349 while set_id
runs only in the exclude path, GCode.cpp:8096) — on large-mesh inputs
the garbage value varies per run, so the comment must be absent.
Native decimates the occlusion mesh to 16000 triangles before
visibility sampling and starts the first-printed object's inner wall at
the top-right corner (travel `G1 X184.37 Y124.37`, wall leftwards
`G1 X155.63 Y124.37 E.8831`); ares raycasts the undecimated mesh and
starts at `X184.37 Y95.63`. CAUSALITY PROVEN (diagnostic ORCA_NO_DECIM
build, default path byte-identical to official): same input, decimation
disabled → native picks the ares corner `X184.37 Y95.63`; enabled →
`X184.37 Y124.37` (layer-1 corner visibility e.g. 0.884208/0.920625/
0.902672/0.882698 decimated vs 0.868157/0.937163/0.883435/0.848119
undecimated flips the pick). Native output verified deterministic
across 5 single-CPU runs and under the canonical harness environment
(modulo timestamp). DIVERGENT in RED with the existing legs PASS.

An earlier decimation attempt (56000-triangle ellipsoid) diverged for the
wrong reason — sub-0.1mm coordinate drift in curved-wall slicing, a
mesh-slicer concern outside this slice — and was replaced by the flat
plate.

## Root cause (source-cited)

`GCode/SeamPlacer.cpp`:

- `:626-648 compute_global_occlusion` gathers MODEL_PART **and**
  NEGATIVE_VOLUME meshes (`ModelVolumeType::NEGATIVE_VOLUME` merged into a
  separate `negative_volumes_set`), each transformed by its volume matrix
  with `fix_left_handed=true`.
- `:650-653`: `its_short_edge_collpase(triangle_set, 16000)` and the same
  for the negative set, BEFORE merging (`ShortEdgeCollapse.cpp:11-187`:
  edge-collapse by shortest edge with vertex dot-product keep-score, face
  neighbor rewiring; deterministic given the input index order).
- `:657-661`: negative set appended after model parts;
  `negative_volumes_start_index` records the boundary; whole set
  transformed by `obj_transform` with `fix_left_handed=true` (already
  ported in ebc7eafa4).
- `:128-227 raycast_visibility`: when `model_contains_negative_parts`
  (`negative_volumes_start_index < indices.size()`, :150), every ray uses
  `intersect_ray_all_hits` parity counting (:196-213): hits on negative
  faces decrement/increment via `sgn(face_normal.dot(dir))` with inverted
  sign versus model faces, and samples ON negative-volume faces cast with
  inverted direction and offset origin (:185-192); a ray is occluded iff
  the counter reaches 0 (:210-212). Without negative parts, the
  first-hit+backface test (:170-182) is used — this is the only path ares
  implements.
- `SeamPlacer.hpp:114-115`: `raycasting_visibility_samples_count = 30000`,
  `fast_decimation_triangle_count_target = 16000`.

Ares gaps (verified 2026-10-18):

- `seam_placement/mesh.rs:158-208 from_object` filters
  `ProjectVolumeType::ModelPart` only — NEGATIVE_VOLUME meshes never enter
  the occlusion set.
- `seam_placement/visibility.rs:99-126 sample_visibility` implements only
  the first-hit+backface branch; no all-hits parity path, no
  negative-start-index concept.
- No `its_short_edge_collpase` port exists in the workspace (grep
  2026-10-18); decimation is a no-op for meshes ≤16000 triangles, which is
  why all existing witnesses (low-poly) passed without it.

## Fix boundary (GREEN proposal)

Scope after the RED outcome: DECIMATION ONLY.

- New `seam_placement/short_edge_collapse.rs`: verbatim-port
  `its_short_edge_collpase` (ShortEdgeCollapse.cpp:11-187), including the
  fixed-seed `std::mt19937_64{27644437}` + `std::shuffle` face ordering
  (line ~82-104: libstdc++ `std::shuffle` + `uniform_int_distribution`
  semantics must be reproduced exactly; ares already ports `Mt19937_64`
  for `sample_its_uniform_parallel`, unchanged), `its_face_neighbors`
  (ShortEdgeCollapse uses `its_face_neighbors_par`, ShortEdgeCollapse.cpp:37;
  ares must port or reuse a semantically identical face-neighbor builder),
  face normals + `NormalUtils::create_normals` vertex normals for the
  keep-score (min vertex dot product), and the `unordered_map` final
  remap (lookups only — no iteration-order dependence in the output).
- `from_object`: decimate the model-part triangle set to 16000 before the
  obj_transform flip. (No negative-volume set in this slice.)
- No behavioral tolerance: 16000 target verbatim.

## Included / deferred

- Included: short-edge-collapse decimation of the occlusion mesh
  (decimation leg witness, regressions: crosshatch suite both modes, FULL
  both modes, clippy, fmt, wasm, 400-LOC file budget).
- Deferred (no witness, per approval condition 1): NEGATIVE_VOLUME
  occlusion set and the `raycast_visibility` parity branch
  (SeamPlacer.cpp:626-648, :128-227) — both negvol attempts produced no
  stable native-active divergence (evidence above).
- Deferred: enforcer/blocker occlusion (`gather_enforcers_blockers`,
  SeamPlacer.cpp:711+ — separate seam-painting concern, unchanged);
  order-aware boolean semantics of overlapping negative volumes (upstream
  TODO at SeamPlacer.cpp:185); decimation of the seam-slope/enforcer
  meshes (none exist in ares yet).
