# Task 22O.38 — Direct bridge-direction selection

## Status and source boundary

Released as implementation/documentation commits `04920e0`/`2d6154d`.
Exact-SHA Tier-1 run `31303115603` passed all five jobs and both browser
executions at `2d6154d401c3c954bed69de6ba631a53af05f1a3`; its authoritative run
JSON is archived outside the repository at
`/tmp/task22o38-tier1-exact-sha.json`. The compiling stub RED had 17 body-
dependent failures and one shape-equivalent pass. The source-shaped
implementation, pinned original-Orca CLI/helper, audited MSVC model, repaired
one-at-a-time mutation campaign, complete native/WASM/static/rollback gates,
and both final independent review tracks pass. Both local Playwright attempts
failed before test code because Chromium could not load `libglib-2.0.so.0`;
exact-SHA CI executed and passed the browser suite twice. O38 remains crate-
private and inactive, changes no Option, lifecycle, adapter, fixture branch,
golden expectation, or G-code byte, and public slicing still consumes O26
before returning `ProjectSlicingIncomplete`.

Exact predecessor O37 is released as implementation/documentation commits
`a0caa5a`/`4d83d15`; exact-SHA Tier-1 run `31291016394` passed all five jobs and
both browser executions at
`4d83d15832c7905d7ea9727d14c07c5a75eb7312`. Its authoritative run JSON is
archived outside the repository at `/tmp/task22o37-tier1-exact-sha.json`.
Pinned Orca remains v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only the independently callable geometry dependency of the next
external-surface stage:

- `detect_bridging_direction(const Lines &, const Polygons &)` in
  `OrcaSlicer/src/libslic3r/BridgeDetector.hpp:75-119`;
- `compute_moments_of_area_of_triangle` in
  `PrincipalComponents2D.hpp:12-17` / `PrincipalComponents2D.cpp:8-64`;
- `compute_principal_components` in `PrincipalComponents2D.hpp:19-20` /
  `PrincipalComponents2D.cpp:66-138`;
- `Line::normal` at `Line.hpp:180`;
- Eigen 5.0.1 normalization as pinned by
  `OrcaSlicer/deps/Eigen/Eigen.cmake:7-8`: `Eigen/src/Core/Dot.h:66-100`
  (`squaredNorm`, `norm`, and `normalized`) and
  `Eigen/src/Core/functors/BinaryFunctors.h:415-426` (scalar quotient).

This is a coherent prerequisite slice, not an Ares-owned direction algorithm.
The header helper is callable without `LayerRegion`, and its empty-edge branch
directly owns the principal-components dependency. Freezing it separately keeps
its mixed `f32`/`f64` and MSVC-container tie behavior observable before adding
Clipper offset/difference and bridge mutation.

Deferred to O39: `detect_bridge_directions` at `LayerRegion.cpp:262-308`,
including ordered bridge-anchor lookup, `to_polygons`/`to_polylines`,
`expand(..., float(SCALED_EPSILON))`, `diff_pl`, and assignment of
`M_PI + atan2(direction.y, direction.x)` to `Bridge.angle`. Also deferred:
`merge_bridges` at `LayerRegion.cpp:310-351`,
`expand_bridges_detect_orientations` at lines 398-437, active
`LayerRegion::process_external_surfaces` at lines 486-623 with declaration at
`Layer.hpp:86`, lifecycle activation, Options, public adapters, fill, toolpath,
seam, motion, serialization, G-code, post-processing, and normalized KSR
parity.

## Ares destination and API

Add one crate-private, platform-neutral geometry entry:

```rust
pub(crate) fn detect_bridging_direction(
    floating_edges: &[Line],
    overhang_area: &[Polygon],
    scale: CoordinateScale,
) -> ((f64, f64), f64);
```

The nested tuple is the direct Rust equivalent of
`std::tuple<Vec2d, double>`: the first pair is `(x, y)` and the second value is
the unsupported-distance cost. Do not add a public vector type, request object,
trait/generic overload, error wrapper, alternate entry, or production test
seam.

The explicit `CoordinateScale` is the required Ares adaptation of Orca's
mutable global `SCALING_FACTOR`. The nonempty-edge branch does not use scale,
but the empty-edge PCA branch executes `unscaled(point).cast<float>()` for
every point. The same caller-supplied scale must therefore reach PCA unchanged.
Do not infer, rescale, or hard-code Normal scale.

O38 remains a crate-private geometry prerequisite with no project-slice caller.
It changes no lifecycle, Option, adapter, golden expectation, or G-code byte.
Public slicing must still consume O26 output and return
`ProjectSlicingIncomplete`.

## Frozen empty-edge and principal-component behavior

When `floating_edges` is empty, execute the pinned PCA literally.

For each polygon in input order:

1. Trust a nonempty point list and take the first point as `p0`.
2. Convert each scaled integer coordinate through
   `(coordinate as f64 * scale.factor()) as f32`, preserving Orca's unscale-
   then-`float` cast.
3. Triangulate with `(p0, points[i - 1], points[i])` for `i = 2..len`.
4. Compute the sign with the exact `f32` comparison
   `cross(p1 - p0, p2 - p1) > 0.0`; zero chooses `-1.0`.
5. Accumulate signed `f32` area, first moments, second moments, and covariance
   in polygon/triangle order using the source expression trees, not merely
   algebraically equivalent formulas. Preserve: `jacobian * 0.5f` for area;
   `jacobian * (a + b + c) / 6.0f` for first moments;
   `jacobian * (a*a + b*b + b*c + c*c + a*(b+c)) / 12.0f` componentwise for
   second moments; and `(jacobian * (1.0f / 24.0f)) * inner` for covariance,
   with the source's written left-associated inner sum. Preserve every
   `sign * value` before each ordered `+=`.

If accumulated area is `<= 0.0f`, return two exact zero vectors. Otherwise:

- compute centroid and variance in `f32`;
- compute the covariance subtraction in `f32` before widening to `f64`, as in
  the C++ assignment;
- compare `abs(covariance) < 1e-4` strictly;
- in that branch, construct `(variance.x, 0)` and `(0, variance.y)` and put the
  larger-variance vector first, with equality retaining X first;
- otherwise preserve the source's mixed arithmetic: compute
  `(variance.x - variance.y)` and its square in `f32`, widen that squared term
  to `f64` only when combining it with `4.0 * covariance * covariance`, take
  the square root in `f64`, cast each eigenvalue to `f32`, cast each
  eigenvector X component back to `f32`, and order by strict
  `eigenvalue_a > eigenvalue_b`.

The direct helper discards `pc1`. If `pc2 == (0.0f, 0.0f)` componentwise,
return `((1.0, 0.0), 0.0)`. Otherwise reproduce Eigen 5.0.1 `normalized()`:
evaluate `z = x*x + y*y` in `f32`; if `z > 0.0f`, divide each component by
`sqrt(z)` using scalar `/` in `f32`, otherwise return the input vector
unchanged. Then widen components to `f64` and return cost `0.0`. Do not use a
reciprocal multiply or stable/blue norm.

Do not replace these formulas with a matrix/eigensolver dependency, polygon
area helper, centroid shortcut, `f64`-only arithmetic, absolute polygon area,
orientation normalization, sorting, validation, or fallback. A source-shaped
empty polygon remains a trusted internal panic. Degenerate/nonpositive signed
area follows the zero result; no public error is added.

## Frozen nonempty-edge direction behavior

For every `Line` in input order:

1. Compute Orca's integer normal exactly as `(b.y - a.y, -(b.x - a.x))` before
   widening to `f64`.
2. Reproduce Eigen 5.0.1 `normalized()`: evaluate
   `z = normal.x*normal.x + normal.y*normal.y` in `f64`; if `z > 0.0`, divide
   both components by `sqrt(z)` using scalar `/`, otherwise retain the original
   vector unchanged. Do not use reciprocal multiplication or stable/blue norm.
3. Compute the key as `ceil(atan2(normal.y, normal.x) * 1000.0)`.
4. Insert `(key, normal)` with `unordered_map::emplace` semantics: the first
   equal key wins; later equal keys do not replace it.

Then create direction-cost records in the target map's iteration order. For
each floating edge in original input order, widen `(b - a)` to `f64` and add
`abs(dot(line, candidate_normal))` to every candidate in map order. Start the
result at `((1.0, 1.0), f64::MAX)`. Traverse costs in map order and replace the
result only on strict `cost < min_cost`, using `(normal.y, -normal.x)`.
Return the selected direction and exact accumulated cost.

Do not canonicalize opposite normals, sort numeric keys, use `<=`, recompute a
candidate from a later duplicate, use line length separately, normalize the
edge vector for the cost, average directions, add validation beyond Eigen's
literal `z > 0` branch, or map NaN to a fallback. A zero-length line retains the
exact zero normal, quantizes from `atan2(0.0, 0.0)`, contributes zero cost, and
is tested as source behavior rather than rejected.

## Deterministic MSVC 14.44 map-order compatibility

C++ does not specify `unordered_map` iteration order, but the KSR reference and
existing comparator-sensitive oracle policy target the Windows x64 MSVC STL
14.44 implementation. The audited compatibility-target provenance is recorded
in ARD-0024, `docs/architecture/ard-0024-safe-indexed-clipper6-kernel.md:41`
and `:152-160`: `_MSVC_STL_VERSION=143`,
`_MSVC_STL_UPDATE=202503L`, toolset directory `14.44.35207`, with exact audited
header hashes. O38 reuses only that accepted toolset target; its unordered-map
control flow is separately audited from official `microsoft/STL` tag
`vs-2022-17.14`. Ares must produce one deterministic order on every Tier-1
platform; it must not use a platform branch or the host Rust hash map.

Implement a small private order adapter matching official
`microsoft/STL` tag `vs-2022-17.14`:

- `type_traits`: `_Hash_representation` and `hash<double>` use 64-bit FNV-1a
  over the little-endian IEEE-754 representation, with `-0.0` mapped to
  `+0.0`;
- `xhash`: `_Min_buckets == 8`, maximum load factor `1.0`, and the bucket count
  grows eightfold while below 512, then to the next power of two;
- a new distinct key for an occupied bucket is inserted before that bucket's
  current low element; an empty-bucket key appends at list end;
- rehash preserves first-encounter order between bucket groups, but each later
  distinct member of a group is front-inserted before its current low element,
  explicitly reversing that group's pre-rehash member order;
- equality remains ordinary `double ==`, including NaN behavior.

This adapter is only the source-compatibility representation for candidate
ordering; it is not a second geometry engine or general map. Keep it private to
the bridge-direction module. Pin its semantics with reviewed behavior vectors,
not tests that ingest STL source text, hashes, or line numbers. The proprietary
MSVC compiler/toolset is not installed on this Linux host, as already disclosed
for O28. Audit the exact open-source `vs-2022-17.14` `xhash` and `type_traits`
bytes and compare production output to an independently written disposable
control-flow model under `/tmp`; exact pushed-SHA Windows Tier-1 remains the
platform gate. If those independent results disagree or any source branch
remains unresolved, treat it as a release blocker. Do not silently substitute
insertion, reverse-insertion, numeric-key, or host-hash order.

## Tests, oracle, and chronological TDD

Use ordinary split test modules and prefix every focused test name with
`task22o38_` so the documented Nextest filter is exact. Every committed expected
value is a manually reviewed behavior-named Rust literal. Raw C++/MSVC helper source, binaries,
serialized output, and generated G-code stay under `/tmp`; never commit or read
reference G-code content.

Capture a real compiling RED against a temporary body returning only
`((1.0, 1.0), f64::MAX)`. Function-pointer shape is not RED. Empty-input or
pathological cases equivalent to that stub must be disclosed rather than
called failures. Record chronological RED separately from post-hoc mutation
results.

Focused tests must cover:

- empty edges with empty, degenerate, reversed/nonpositive, axis-aligned, and
  covariance-bearing polygon inputs;
- complete `f32`-sensitive PCA vectors at Normal and LargeBed scales, including
  exact direction components/bits where the pinned helper is stable and a
  non-axis-aligned vector that exposes the exact Eigen square/sqrt/division
  sequence;
- strict covariance threshold, equal variance ordering, `pc2` zero fallback,
  and nonzero `pc2` normalization;
- one horizontal and one vertical edge with exact direction and cost;
- multiple candidates with complete costs and strict-min selection;
- duplicate quantized keys proving first-emplace wins;
- reversed normals remaining distinct;
- equal-cost ties exposing the target map iteration order and strict `<`;
- occupied-bucket insertion, the ninth distinct key rehash, and at least one
  post-rehash collision whose expected order distinguishes within-group reversal
  from preservation, against reviewed MSVC 14.44 literals;
- original input line order in both candidate creation and cost accumulation;
- trusted empty-polygon panic and zero-length behavior with only Eigen's
  literal `z > 0` branch;
- exact function signature and crate-private reachability.

Run the pinned original Orca CLI on the KSR 3MF in a disposable environment as
the project E2E, retaining only exit/result metadata and nonzero output size
under `/tmp`, then delete generated G-code without content ingestion. Build one
disposable helper from the exact pinned Orca functions in Debug and `NDEBUG`;
require byte-identical non-tie and PCA vectors. For tie, collision, duplicate,
and rehash vectors, archive the exact official STL tag/commit, `xhash` and
`type_traits` hashes/branches, and an independently written disposable model's
complete output. A host-GCC `unordered_map` result is not MSVC-order evidence.
Only transcribe behavior literals after reviewing both source audit and helper
or model output; exact-SHA Windows Rust tests confirm the platform-neutral
rewrite executes identically, not the unavailable proprietary C++ runtime.

Post-hoc mutations include changing PCA scale/casts/arithmetic width, sign or
area comparison, covariance threshold, eigen ordering, pc2 normalization,
normal orientation, quantization ceil/multiplier, duplicate replacement,
FNV bytes/constants, zero canonicalization, bucket count/growth, occupied-
bucket insertion, rehash order, input/cost loop order, strict minimum, result
rotation, initial result/cost, signature, or visibility. Apply one at a time and
restore exact bytes. Behaviorally equivalent scale/order mutations on bounded
vectors must be reported as survivors and fixed by literal structure/diff
review; do not add production instrumentation solely to force a kill.

## Files, limits, and prohibitions

Allowed Rust edits only:

- `crates/ares-core/src/geometry.rs`: register/reexport the crate-private direct
  helper and add its exact function-shape assertion;
- new `crates/ares-core/src/geometry/bridge_direction.rs`: direct helper,
  source-shaped vector arithmetic, and private MSVC-order adapter, at most 220
  physical lines;
- new
  `crates/ares-core/src/geometry/bridge_direction/principal_components.rs`:
  source PCA formulas, at most 220 physical lines;
- `crates/ares-core/src/geometry/tests.rs`: ordinary test registration;
- new `crates/ares-core/src/geometry/tests/bridge_direction.rs`: shared focused
  helpers and ordinary submodule registrations, at most 140 physical lines;
- new `crates/ares-core/src/geometry/tests/bridge_direction/pca.rs`: PCA/empty-
  edge witnesses, at most 300 physical lines;
- new `crates/ares-core/src/geometry/tests/bridge_direction/selection.rs`:
  nonempty direction/order witnesses, at most 300 physical lines.

No `line.rs`, polygon/ExPolygon/Clipper kernel, project-slice production or test,
manifest/lock/dependency, lifecycle/stage/predecessor, adapter, workflow,
golden, fixture expectation, or G-code path may change. Allowed documentation
is this spec/plan, O37 spec/plan release-state corrections, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and `THIRD_PARTY_NOTICES.md`. Broaden
only the existing MSVC STL notice to name the O38 `xhash`/`type_traits`
compatibility rewrite and official tag; existing license text remains
unchanged. No ARD change.

Every Rust file remains below 400 physical lines. No broad lint allowance,
`unsafe`, FFI, filesystem/native thread, platform branch, public API/hook,
hard-coded fixture identity/name/hash/layer-count/geometry branch, reference-
G-code read, binary oracle, legacy fallback, source concatenation, source-
pinning test, second clipping engine, dependency change, or host-random hash
order.

## Verification, review, release, and rollback

Require focused debug/release O38, complete geometry tests, O37/O36/O35, O28,
O30, RegionExpansion/external-surface regressions, PolyTree/boolean-paths/
offset, O26 lifecycle, workspace Nextest, all-target check, warning-denying
Clippy, rustfmt, four WASM checks, two optimized builds, export/JavaScript audit,
and two Playwright runs. If local Chromium lacks `libglib-2.0.so.0`, record each
failure exactly and require both exact-SHA CI executions; never label it a pass.

Static-audit the exact allowlist, ordinary modules, LOC, crate-private
visibility, mixed numeric/order semantics, absence of forbidden patterns and
staged/generated artifacts. Rehearse disposable rollback to exact released O37
`4d83d158...` and prove the primary candidate unchanged.

Fresh independent six-dimensional and default-model OpenCode reviewers must
approve spec, plan, implementation, and final documentation. Every accepted
review repair invalidates stale candidate evidence: rerun affected and complete
exact-byte gates, refresh static/rollback evidence, and repeat both reviews.

Use separate Conventional Commits for implementation and documentation, push
only approved files, and require Tier-1 `headSha` to equal the pushed
documentation SHA with exactly five successful jobs and both browser executions.
No tracked release-state edit follows that run; O39 records the released state.

The next bounded source candidate after O38 is `detect_bridge_directions` at
`LayerRegion.cpp:262-308`, now composing the frozen direct helper with ordered
anchor lookup, scaled-epsilon expansion, open-path difference, and Bridge angle
assignment. Merge/orchestration/fill/toolpath/motion/G-code remain deferred.
