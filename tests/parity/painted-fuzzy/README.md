# Stock GUI painted fuzzy-skin public RED

Upstream8500fcdccaa10b5099ac20d252af3a7c560046f1; approved SPEC2dc9cdd57:
`docs/architecture/painted-fuzzy-skin-spec.md` (includes failure inventory).
Owners: bbs_3mf canonical carrier → PrintApply actual regions → fuzzy wrapper of
segmentation_by_painting → PrintObjectSlice apply → classic FuzzySkin masked apply.

Both fixed projects retain legacy-brim-ears/O0.3mf's stock cube geometry:
8vertices/12triangles, model/instances/preset container unchanged. Only original
project setting Disabled_fuzzy→None is changed in both inputs. The target also
annotates original triangles10/11 (local+X complete vertical face) with canonical
unprefixed paint_fuzzy_skin="4" (unsplit state1). This is source-constructible GUI
serializer-equivalent painting, NOT an actual interactive GUI-save claim.

GLGizmoFuzzySkin.cpp326–354 enables painting via None;367–410 stores/restores
fuzzy_skin_facets. TriangleSelector.cpp1692–1752, Model.cpp3545–3595 and
bbs_3mf.cpp330/7339–7346 own the encoding. Do not conflate painting with global
All or Disabled_fuzzy. No geometry/Native output constants or generated G-code
fixtures. Exact input identities, source and effective settings are in manifest.

Retained stock classic/Ripple/displacement: thickness0.2, point distance0.3,
15ripples,50%offset, spacing1; no first-layer fuzzy, support, tower or beam;
stock EFC0.1. Source Ripple branch is deterministic, no stochastic waiver.

The Native-only preflight test runs reference/replay for BOTH cases, checks all
listed Native settings, positive model wall XY+extrusion and active paint motion.
Monofilament need not emit T0; a local wall-motion diagnostic checks real relative
extrusion, without inventing tool commands or using the MMU activity parser.

Public control/target use shared native_stream_case/NativeCase: actual Native
double replay before real validated byte API and real CLI. Both MUST succeed,
match complete ordered same-input Native bytes (only established generator-line
allowance) and raw API==CLI. Rejection must fail the test, not expected-error PASS.
Artifacts retain raw inputs/Native/API/CLI outputs and comparison/Native replay
JSON; on API rejection the shared seam still runs CLI and records its status.
Target comparison.json may lack case because the success assertion stops earlier:
bind its identity via the already-written native-replay.json and frozen input SHA.

Qualification must run the Native-only preflight in both official/seeded modes
before ANY public test calls, CPU18/jobs1/parity threads1. Replays/control/resource
failures STOP/non-gate; no normalization expansion or retries. No FULL here.

Tests register under painted_cases::painted_fuzzy, via a grouping submodule.
Existing test paths remain unchanged; root registry397LOC, no squeezed blank lines.
Deferred: MMU+fuzzy, beam+fuzzy, Arachne-painted, nonzero XY and broader scope.
