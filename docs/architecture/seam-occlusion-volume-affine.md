# SeamPlacer occlusion volume-affine precision

This is a libslic3r rewrite slice, not a new
Ares seam-selection, visibility, travel or timing policy.

## Source owner and exact Rust destination

`OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:637-641`, inside
`SeamPlacerImpl::compute_global_occlusion`, obtains the ModelVolume double matrix,
copies its float indexed_triangle_set, and calls
`its_transform(model_its, model_transformation, true)` before merging MODEL_PARTs.
`OrcaSlicer/deps_src/admesh/stl.h:362-366::its_transform` evaluates
`(t * v.cast<T>()).cast<float>()`, with T=double here: promote the float vertex,
perform the affine in f64, then narrow the completed result back to float.

Destination: `crates/ares-core/src/project_slice/seam_placement/mesh.rs`, only the
MODEL_PART volume transformation callsite in `TriangleMesh::from_object` (base
line200). It currently uses `Transform3d::transform_point_f32`, a slicing-specific
contract that narrows coefficients and intermediate arithmetic early. That use
is a temporary compatibility shell, not the cited occlusion transform contract.

Included future rewrite: use the EXISTING f64 `Transform3d::transform_point`,
then narrow each resulting coordinate with `as f32` at this volume mesh boundary.
Keep vertex/index order, left-handed winding correction, merge and decimation.
Do not introduce another transform API, abstraction, threshold, fallback or flag.
Core remains platform-neutral for browser WASM, Windows, macOS and Linux.

## Non-negotiable boundaries and explicit deferrals

- Change neither `transform_point_f32` nor `transform_z_f32`, their slicing users,
  nor ARD-0076. Their per-product/sum f32 rounding is intentional for slicing;
  changing the shared helper would conflate distinct upstream contracts.
- The OBJECT stage, `its_transform(triangle_set, obj_transform, true)` at
  `SeamPlacer.cpp:661`, also uses a double affine upstream. Its Rust call at
  `mesh.rs:227` remains unchanged and explicitly DEFERRED. Both object matrices
  are identity in this witness, so nonidentity object-affine precision has NOT
  been independently qualified. Do not claim that adjacent stage is ported.
- Do not change sampling/RNG, triangle areas/normals, ray/AABB/KD queries,
  visibility aggregation, embedding/overhang, nearest comparator, seam split,
  medial/Voronoi, InterlockingGenerator, geometry loading/centering, travel,
  spiral lift, cooling, M73/time, brim/skirt, painted/MMS, ironing or infill order.
- Do not absorb a strict residual into another producer or consumer. If the
  volume-only future correction does not pass COMPLETE frozen streams, STOP
  and report it; no silent object-stage widening or consumer compensation.

## Qualified first-producer evidence

Source arrays are not recorded in this SPEC or committed Native output.

- Native/Rust runtime inputs are identical: two f64 volume matrices, all36 float
  source vertices and64 triangle indices. Native's actual post-volume vertices
  match f64 affine then f32 narrowing for every vertex. Early f32 gives14/18
  different vertices per volume. Example first volume X: Native double->float
  -4.191534515030071e-7 versus early-f32 -4.76837158203125e-7.
- Host arithmetic on immutable DWARF snapshots exactly predicts all64 Native
  occlusion triangles with double->float and all64 Rust triangles with early
  f32. 60/64 triangles differ. Both object transforms are identity. This is
  numerical-boundary evidence, NOT a modified pipeline run or GREEN claim.
- Of30,000 samples,7,719 positions and1,930 visibility values differ downstream.
  All30,000 normals, total area1288.110595703125 and radius.7203308343887329 match.
- In96 nearest-choice records, the first40 choices agree. First differing choice
  index40 (printZ1.8/sliceZ1.7) has identical cursor/candidate positions/angles and
  zero overhang; all5 candidates share the hidden (<-.5) class. Tiny embedding
  differences do not change that class. Native visibility at candidates2/3 is
  .9699428081512451/.9710490703582764; Rust nearby sample values are all1.
  Native chooses2, Rust3. Offline visibility-only substitution into the captured
  penalty inputs selects2, without inferior mutation or a pipeline GREEN run.
- The pre-travel post-seam loop is the same closed coordinate cycle, rotated to
  different vertices. Thus the first G3/destination difference below already
  exists before travel/spiral lift; those consumers are not the owning slice.

All16 accepted read-only GDB probes exit normally and preserve complete non-GDB
output (Native unchanged canonical generator normalization; Rust raw equality).
`may-call-functions off`; only field/memory reads and normal break/return
observation, no inferior calls or data/register writes. Empty/stopped probes are
excluded by `trace/observer-proof.json`. Native BP640 resolves641 AFTER the
transform: the valid v2 reads immutable `model_volume.m_mesh` as the true input
and transformed `model_its` as output; no misleading pre-state claim is used.

## Frozen GUI-reachable public RED pair

Two closed polygon-prism parts,20x20mm footprint,4mm/20 layers, one object,
filaments1/2, Classic/nearest, scarf none. Each part has18 vertices/32 nondegenerate
triangles, with directed-edge manifold checks. GUI: import meshes as parts,
assemble, assign filaments, configure interlocking boundary avoidance1 and toggle
beam. `GUI/Tab.cpp:2334,2387,2691,2695-2699` exposes these settings.

Use the ALREADY Native-exported input-only archives unchanged:

| Role | Future fixture | SHA256 |
|---|---|---|
| Target | occlusion-volume-affine/beam1-target.3mf | `ad35795f29e68c292d644a0c8019dd96fb1dac2736adfc651692c2d3a14173f1` |
| Sole-key control | occlusion-volume-affine/beam0-control.3mf | `171ce0b6e5160bbe697212fd548beaa5228fd91cd741fb456ddcdf596feebabd` |

Only Metadata/project_settings.config differs, solely interlocking_beam1->0;
all geometry/other payloads match. No G-code members or Native output committed.
Do not regenerate inputs, change settings to avoid residuals, or duplicate
existing fixture hashes. Discovery eight fresh Native runs already established
both-mode stable Native2 BEFORE byte API/real CLI; target complete FAIL/control
complete PASS both modes, API success/CLI0 and both raw API/CLI pairs equal.

Strict generator-only first difference is NON-M73: canonical byte40963,
line1827,col10: Native `G3 Z2.2 I-.093 J-1.213 P1  F18000` versus
Ares `G3 Z2.2 I.072 J-1.215 P1  F18000`. Next destination is
Native110.32,101.56 versus Ares110.458,101.503. Preserve ALL M73, motion, timing,
metadata and footers; no masks, selective acceptance or expected-error success.

## Verification sequence

1. Input/test-only public Native-first RED. Reuse unchanged `runner::native_stream_case::check`
   delegating to canonical `native_api_cli::NativeCase`; assert SHA and T0/T1.
   Native2 must be stable before API/CLI for each case, in official and seeded.
2. Require each mode target FAIL/control PASS, both complete Native/API and
   Native/CLI comparisons, API success/CLI0. On RED, the canonical helper panics
   before its final raw assertion: independently audit actual API/CLI files;
   do not claim that final assertion or post-compare adapter code was reached.
3. Freeze inputs, tests, helper, normalizer and SPEC; the production-only GREEN
   corrects only the volume
   callsite using existing f64 transform followed by per-coordinate as f32.
4. GREEN must pass whole target/control streams in both modes, raw equality,
   selected nearest/aligned/mesh/interlocking Native neighbors, fmt, strict
   workspace/all-target clippy and WASM gate. Changed text files stay <400LOC.
