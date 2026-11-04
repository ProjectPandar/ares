# Epsilon-aware complex layer-range lookup rewrite

Status: accepted; production followed independent runtime Native RED. Builds on
Stage2. This is an independent libslic3r source slice, not an expansion of an Ares-owned range pipeline.
Architecture: [lookup boundary ARD](../../architecture/2026-10-08-layer-range-epsilon-boundary.md).

## Owning source and Rust boundary

Admitted OrcaSlicer 2.4.2 PrintObjectSlice.cpp SHA256:
`d1aece12e7189353aa9a5d8ae6b1f0f320087c0d2574d7da64fe372eca076021`.

- PrintObjectSlice.cpp:244–255, `layer_range_first`: lower-bound predicate is
  `max_z < z && abs(max_z - z) > EPSILON`. Exact equality with a range end moves
  to the immediately adjacent range whose start equals Z, when present.
- :258–266, `layer_range_next`: advance the retained iterator while
  `max_z <= z + EPSILON`. This is not equivalent to first-query selection.
- :338–350 / :395–398, complex `slices_to_regions`: initialize the iterator
  using the first complex float Z of each work block, then call next **before
  every complex layer**, including that block's first layer.
- libslic3r.h:52 defines `EPSILON = 1e-4` as double, not machine epsilon.
- :290–312, fast shuffle: uses exact comparisons against range start/end and
  float Z. It does **not** use either epsilon helper. Do not apply the complex
  predicate globally to the fast path.

Rust destination is `project_slice/region_slices/complex.rs`, with small
source-owned lookup helpers in `region_slices/range_lookup.rs` if needed.
`VolumeRegionRange` remains the existing input mapping, not a new graph.
The current `region_slices.rs::records_at_z` partition predicate `max_z <= z`
is a temporary complex-path shell: stop using it for complex assignment while
preserving the fast shuffle's exact boundary semantics. Keep the existing
actual `slice_z as f32` conversion before promoting to f64 for comparisons.

## Included and deferred behavior

Included: first lookup's epsilon-aware lower bound and exact-adjacency rule;
next lookup's inclusive `z + EPSILON` rule; monotonically advancing complex
range iterator; float-Z precision; reuse of selected per-range volume records
for existing overlap/clipping/modifier routing. First and next remain distinct
source concepts. Iterator advancement must run on the first complex layer too.
Use linear advancement across ordered complex layers rather than per-layer
config cloning, repeated graph construction, or an invented tolerance policy.

Fast-path exact range routing is a non-regression boundary, not a target for
changing tolerance. No change to LayerRanges construction, explicit-height
profiles, PrintRegion identity/config equality, geometry, flows, or export
normalization is intended.

Deferred: MM/fuzzy painting consumers at PrintObjectSlice.cpp:884–903 and
:1047–1053, their parallel segmentation pipelines, spiral+layer ranges,
other region-option families, multifilament, painted profiles, SlicingAdaptive,
and active ZAA. New helper availability does not bring these consumers into scope.
Do not imitate C++ assertion/crash behavior or invent defaults for missing
range heights. Every Native witness retains explicit legal layer_height.

## Failure modes and independent RED plan

Pre-change failure modes: using next's tolerance on the fast path; substituting
first for next or omitting next on the first complex layer; replacing inclusive
`<=` with `<`; using machine epsilon; comparing nominal/print Z rather than
actual float slice Z; restarting an iterator incorrectly; wrong near-boundary
volume mapping; destabilizing source region IDs; hiding Native rejection or
assertion failure behind an unsupported assertion.

1. Derive a legal constant-height, explicit wall_loops range
   witness from the admitted Stage2 archive, with existing supported overlapping
   volumes or a negative/modifier volume that **actually forces complex routing**
   at the boundary. First prove its no-range/far-from-boundary Native controls;
   preserve every attempted input/outcome. Do not assume the single-volume
   Stage2 witness exercises the epsilon helpers: it takes the fast path.
2. Bind an actual float slice Z, then vary range ends on both sides of that Z:
   exact equality, below/above within 1e-4, and outside 1e-4. Include an inclusive
   threshold case only when the input's parsed double and actual float operands
   establish the threshold exactly; never infer it from decimal print-Z text.
   The required primary RED must have Native exit0 and a complete output while
   existing Rust differs at the complex range boundary. Controls must establish
   that overlap/range mapping, not another unsupported feature, owns the change.
3. Capture executable/resources/source/seed/CPU identities, archive bytes,
   exact range doubles, actual float Z, argv/status, full Native/Ares streams,
   and first difference in `/tmp`. Existing Native/GDB diagnosis may supplement
   the public byte API; do not rebuild OrcaSlicer. Diagnostic region/cursor data
   is not a replacement for the whole-output oracle.
4. Commit independent failing runtime-Native E2E before production edits, using
   shared fail-closed native_environment support. Add fast single-volume exact
   boundary controls before changing complex selection. Extend one case at a
   time only after its own independent RED, not a prewritten Rust golden.
5. Implement first/next source semantics in the complex consumer only, then run
   unchanged fixtures/comparators. A zero-layer object continues to fail at
   the already-merged slice-completion boundary; no step/cursor special cases
   masking invalid inputs. A Native crash, assertion, rejection, or timeout is
   retained non-PASS/inconclusive as appropriate, never retried into PASS or
   converted into an expected unsupported-guard assertion. If legal Native
   admission cannot be established, return to spec review with that evidence.

## Acceptance and review

Complete Native/Ares comparison excludes only the entire generated-by line.
No motion, M73, config, Z, template, or region normalization. Primary and
near-boundary complex cases must match full streams; fast-path controls,
merged wall-loop witness, Stage1 height controls, and zero-height error remain
unchanged. Broader complex options require their own source-cited TDD slices.

Fresh same-main ordered full required seeded base/candidate, full workspace
nextest, all-target/all-feature clippy with warnings denied, rustfmt, and
core/WASM all-feature browser target check; bind complete common-byte audit
and every changed/new stream to input identities. Disclose missing baseline
new-test coverage and all non-PASS attempts.

Native outputs, runtime records and diagnostics stay outside the repository;
commit only fixture/source and conclusion-only docs.

At spec time, evidence was source reading only: no new fixture, runtime probe,
test or production edit existed for this slice. Subsequent runtime evidence is
recorded in the epsilon verification ARD.
