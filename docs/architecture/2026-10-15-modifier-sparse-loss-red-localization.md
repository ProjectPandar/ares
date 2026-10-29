# Modifier sparse loss: public RED and coordinate-frame STOP

Status: RED observed; no production change; GREEN blocked on ownership ruling.
Approved boundary: [modifier sparse-loss specification](2026-10-15-adaptive-noop-modifier-fill-loss-spec.md).

## Exact tree and public result

Main is `3ee8f272c6d8e2db22dde9003a01afeb7201e36a`.
The approved spec rebases to `412eb7067`; public RED is the separate
`c488128e8ab3bb86d653fe50b58c799c7daafdde` commit. Its production-source diff
against main is empty. The three W3-authored input archives retain their frozen
SHA values; no Native output or observation is committed.

Both modes use runtime `OrcaRunner::slice_case`, the public
`slice_project_plate_validated` API, and the real CLI. Assertions retain ordered
whole-byte comparison with only the existing generated-by normalization, line
counts, layer vectors, and raw API/CLI equality. The default test profile has
`debug_assertions=true`; neither failing case panics.

| Input | Official | Seeded | Native/Ares lines | Native/Ares positive sparse XY/E |
| --- | --- | --- | --- | --- |
| A-C0, no modifier | PASS | PASS | 5024 / 5024 | 1207 / 1207 |
| A-C1, same-config intersecting modifier | FAIL | FAIL | 5024 / 3376 | 1207 / 0 |
| A-C2, nonintersecting modifier | FAIL | FAIL | 5031 / 3383 | 1207 / 0 |

Both nextest legs exit 100, with 1 PASS and 2 FAIL. All inputs are admitted by
Native and Ares; CLI exits 0; API/CLI bytes and layer vectors agree in all six
runs. A-C1 first differs at byte 454, line 18: Native `M73 P0 R12`, Ares
`M73 P0 R10`. A-C2 first differs at byte 691, line 25 with the same text.
The first missing positive sparse path is physical layer 4, Z0.8: Native
`G1 X137.02 Y130.97 E.05401` (A-C1 line 632, A-C2 line 639).
These diagnostics do not narrow the whole-output comparator.

Canonical environment SHA is
`e0a6d9ce027d6ae7147a4993951e5541809aceeb7ad60f22f945e9a3a6a5fb99`;
only the nextest runner is replaced with the assigned CPU11 runner. Outer
cargo/nextest/Native run under taskset CPU11, jobs1/threads2, with a fresh cache
and artifact root per mode. The sampler records all six actual Native process
identities at affinity 11, with zero violations.

## First unequal observed intermediate

Read-only GDB observes the existing accepted main FULL16 CLI binary. The
observed G-code is raw-byte identical to its ordinary CLI execution for each
input; there are no inferior calls, assignments, or source instrumentation.

All 51 observed pre-group context surface arrays match A-C0 exactly for A-C1
and A-C2. More decisively, all 43 Adaptive append inputs match exactly:
region owner, complete contour/hole coordinates, density 30, spacing
0.40707963705062866, multiline 1, and adaptive line spacing
4.4999998658895537. The first sparse contour spans scaled coordinates
`[-9223228, 9223233]` in both axes, in the object's local frame.
Thus these witnesses do not show a missing sparse surface at the region/group
handoff originally hypothesized by the spec.

At the first append call, Z0.8, the observed octree value differs:

| Value | A-C0 | A-C1 and A-C2 |
| --- | --- | --- |
| root cube X | -0.00000011605717375179681 | 117.99999988394282 |
| root cube Y | 0.00000032233026781014473 | 138.00000032233027 |
| root cube Z | 4.999999979200413 | 4.999999979200413 |

`octree.origin` and all cube-property values remain equal across the three
inputs. The modifier root centers have an additional `(118,138)` translation,
while the supplied surfaces remain local. Adaptive append returns zero
polylines on all 43 sparse layers for A-C1/A-C2; the first C0 call returns two.

The directly observed unequal value is `octree.root_cube.center` at the
Adaptive append boundary. This is a Rust control/modifier comparison; Native
internal root coordinates have not been extracted. Attempts to inspect the
optimized-out build-frame argument are retained as unavailable, not treated
as measured values. One upper-layer InternalBridge angle also differs after
coalescing; equality of sparse inputs does not assert equality of every bridge
parameter or prove that no later independent difference remains.

## Owning source boundary and decision

Upstream `PrintObject::prepare_adaptive_infill_data`,
`PrintObject.cpp:981-984`, transforms the mesh with
`to_octree * trafo_centered()`. `Print.hpp:327-328` defines that centered
transform by subtracting the object's center offset. `PrintObject.cpp:91-110`
defines the offset from the transformed object geometry without instance XY
translation. `FillAdaptive.cpp:1472-1480` rotates cube centers back; it does
not add instance XY translation. The build invokes that transform at
`FillAdaptive.cpp:1524-1526`.

The relevant Rust boundary is
`project_slice/fill_entities/adaptive.rs::build_object_octree`: it derives
`center_offset` at line 182 and passes it to `build_adaptive_octree` at line
208. `fill/adaptive/octree.rs:225-244` adds that offset back to each rotated
cube center. The current caller comment assumes layer surfaces share this
world frame; these modifier witnesses violate that assumption. The measured
root translation and unchanged local surfaces identify a coordinate-frame
handoff problem, rather than absence of sparse polygons in the observed
region/group handoff. This does not authorize removing a translation globally:
other input coordinate frames and SupportCubic remain outside this evidence.

**STOP:** the approved W6 slice excludes Adaptive preparation/tree/frame
changes, which overlap W3's Adaptive ownership. Keep the public RED frozen;
do not implement a speculative region/grouping repair. Request coordinator
ownership/scope adjudication before any GREEN. Neither this document nor the
public RED claims acceptance or a complete fix.

## External evidence

- Native-first acceptance and public API/CLI preflight:
  `/home/indexyz/ares-evidence/worker6/modifier-native.oQWrNm/`;
  aggregate `public-preflight.json`.
- Exact RED official/seeded nextest logs, exits, comparison artifacts and
  process-affinity audit:
  `/home/indexyz/ares-evidence/worker6/modifier-public-red.HjwbK4/`.
- Read-only observations, scripts, unchanged-output audit and limitations:
  `/home/indexyz/ares-evidence/worker6/modifier-localization/audit.json`;
  per-input `A-C*/final/{observations.json,gdb.log,observed.gcode}`.

All generated products above remain outside the repository. No full/workspace
run, Native rebuild, assertion reversal, comparator change, or push is part of
this RED investigation.
