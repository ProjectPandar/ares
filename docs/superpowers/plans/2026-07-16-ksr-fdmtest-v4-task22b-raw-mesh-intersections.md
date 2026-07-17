# Task 22B: Scaled Raw Mesh Intersections Implementation Plan

> **Execution contract:** Follow the approved SDD workflow and this checklist
> in order. No production or test implementation may begin until these exact
> plan bytes receive literal `VERDICT: APPROVE` from both a fresh independent
> Codex reviewer and the required default-model OpenCode reviewer. Execute the
> ten bounded vertical packages with fresh implementer subagents and no package
> commits. Every package needs fresh specification-compliance and code-quality
> approval. Update tracked architecture/roadmap documentation, commit, and push
> only after whole-implementation approval and a fresh release matrix.

**Approved specification:**
`docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md`

**Approved specification bytes / SHA-256:**
`62280` /
`975d763a80a19eaea27a57bdec008c8a3e73f0508a5e0064b53d27c464012eb5`

**Pinned OrcaSlicer checkout / SHA / tree:**
`C:\Users\Indexyz\AppData\Local\Temp\Ares-Orca-8500fcdc` /
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549`

**Ares baseline SHA / branch / released Tier 1:**
`91fc19f1dbfc85d21431791d2d5acb78af818671` /
`codex/ksr-fdmtest-v4-parity` /
`29543841835`

## Goal and immutable behavior ledger

Port only the approved `libslic3r` raw mesh-intersection slice. The bounded
loader must materialize Bambu vertices through f32, normalize import winding,
omit empty meshes, center fresh meshes with exact f32 subtraction and f64
transform compensation, and bound build-reachable component expansion. The
Task 22A private plan supplies ordered `slice_z` values. Task 22B selects one
request-local coordinate scale, constructs shared-edge topology, intersects
each admitted indexed volume in deterministic face-major order, and retains
directed scaled raw lines with vertex/edge provenance inside the private
project state.

The fixed upstream boundary is the approved specification's cited portions of:

- `libslic3r.h` and `Point.hpp` for `coord_t`, normal/large scale, truncating
  coordinate conversion, unscale, and integer point ordering;
- `Format/bbs_3mf.cpp`, `TriangleMesh.cpp`, `Model.cpp`, `Model.hpp`, and
  Eigen 5.0.1 for f32 source materialization, import signed volume, one-time
  winding normalization, empty omission, fresh centering, and compensated
  component transforms;
- `ObjectID.hpp`, `Print.hpp`, `PrintObject.cpp`, `PrintApply.cpp`, and
  `PrintObjectSlice.cpp` for creation-order volume identity, raw center,
  centered object transform, admitted volume types, `slice_z`, and lifecycle;
- `TriangleMesh.cpp` for normalized edge groups and shared edge IDs;
- `TriangleMeshSlicer.cpp` for facet ownership, rounding, endpoint direction,
  multi-plane dispatch, and raw `IntersectionLine` output.

This package stops before `make_loops`. It must not call, rename, or adapt the
existing Ares `stl`, `model`, `planning`, `segments`, `contours`, or `pipeline`
scaffolds. Those paths lose indexed vertex/edge identity and use incompatible
epsilon, f64 interpolation, endpoint ordering, or `print_z` behavior. No
`Polyline`, polygon, Clipper, repair, region, surface, toolpath, G-code, or
generated metadata behavior enters this task.

### Non-negotiable exact semantics

- `Vertex` and model-unit multiplication use f32 before exact promotion into
  the existing f64 `ProjectMesh` compatibility shell. There is no retained f64
  parse/unit fallback.
- Import signed volume follows the specified f32 face-order expression. Only a
  final strict `< 0.0` swaps triangle slots 1 and 2. Zero, negative zero, and
  NaN do not flip or create a new error.
- Empty vertex or triangle meshes are omitted before volume metadata selection,
  `ProjectVolume` construction, shared-key retention, and ordinal creation.
- Fresh centering subtracts `(center_shift as f32)` from f32 vertices and keeps
  the full f64 shift in `C * T(center_shift)`. Metadata `matrix` remains source
  provenance. When the complete shift is zero, including signed zero, every
  source vertex bit remains untouched. The two operations may not be
  algebraically cancelled.
- Build-reachable component cycles are preflighted iteratively. BFS pending
  items carry only path, object ID, and transform. One request-wide checked
  1,000,000-unit budget claims occurrences before queue growth and nonempty
  vertex/triangle units before materialization.
- `ProjectVolume::id()` remains numeric 3MF provenance. Private one-based
  `VolumeOrdinal(NonZeroU32)` is per source object, follows nonempty BFS
  occurrence order before type filtering, and deliberately retains gaps.
- Explicit `mesh_shared` presence and any repeated nonempty numeric leaf ID are
  rejected request-wide as `shared_mesh_centering`. Empty occurrences do not
  participate. No fresh-mesh approximation of Orca's shared branch is allowed.
- The post-Task-22A preflight order is nonempty ranges, print-object centering,
  shared-mesh centering, ordinal projection, then dense slots. The exact
  1,000,000-slot check precedes scale, center, coordinate, topology, raw-line
  work, and allocation for every object.
- Scale comes only from resolved 3MF `printable_area`: span at or below 2,147
  mm uses `0.000001`; larger uses `0.00001`. It is request-local and immutable.
- External scaled coordinates are checked in f64 against `[-2^63, 2^63)` and
  truncate toward zero. Interior edge intersections use
  `floor(value + 0.5)`, not `round()`.
- Raw center uses importer-centered model-part vertices only, the first source
  instance with all translation removed, compensated volume transforms, f64
  bounds, and quantized/unscaled XY center. Slicing separately retains object Z
  and removes instance XY, pretranslates the negative quantized center, composes
  object then volume, prescales XY only, casts the matrix and vertices to f32,
  and never re-flips a mirrored affine triangle.
- Topology is fully built before any facet intersection. One/two-use normalized
  edge groups are supported; more than two uses returns `mesh_topology`.
- Facet planes use strict f32 equality. General lines run collected point 1 to
  point 0; owned top edges are reversed; bottom and horizontal cases are not
  retained. Zero-length results are preserved.
- Multi-plane slicing is face-major then eligible-plane-major. Duplicate planes
  and empty layer slots remain. Each volume is transformed/indexed once, and
  triangle spans use binary search rather than a triangle-by-layer matrix.
- One request-wide raw-line budget permits exactly 1,000,000 retained lines and
  claims before append.
- Final private ownership nests each `PlannedPrintObject` inside its
  `IntersectedPrintObject`, whose volumes each own exactly one vector per
  planned layer. No parallel top-level plan/raw vectors survive.
- A valid supported request still returns `ProjectSlicingIncomplete`, but only
  after this state exists. The exact 49,004-byte Bambu config block is unchanged.
- KSR counts, hashes, records, filenames, and fixture values occur only in
  tests. New tests read only committed 3MF bytes, never the reference G-code or
  `options-v242.json`, and never open or grep Orca source.

## Frozen baseline and workspace discipline

Use ignored `.superpowers/sdd/task22b-evidence.md` as the only execution ledger.
Record approved spec/plan hashes, baseline and package manifests, RED/GREEN
commands and exit codes, reviewer identities/verdicts, local release evidence,
commit/push evidence, and exact-SHA Tier 1. Never stage the ledger. Do not edit
or trust an older task ledger for Task 22B state.

Before this plan, the tracked tree is the released baseline and the approved
Task 22B spec is the sole untracked tracked-candidate file. Preserve any later
unrelated user change. Ordinary `git diff` omits untracked files, so inspect
every new file with a full read and run the exact no-index loop in the
structural-audit section before review.

Baseline byte guards:

- KSR 3MF: 183,007 bytes, SHA-256
  `698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9`;
- KSR G-code: 6,339,134 bytes, SHA-256
  `10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3`;
- KSR option oracle: 456,004 bytes, SHA-256
  `33C99EE71594ED7F80B44ABC3007DF8E9AE4EC0800411E3B5DBA500F47FD085B`;
- dynamic baseline: 675 LF rows, SHA-256
  `0DCEA4C112EF10F0D6E8C8EE7F63CFEF1831D7C2AE2E399016F1E38372543BE7`;
- dynamic allowlist: 2 rows, SHA-256
  `6B9C3BA6A1C52118A14D66F607CF85A9D13C27185B1FA22D670983E9371A94B6`;
- exact KSR config block: 49,004 bytes, SHA-256
  `b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Fresh baseline manifests are:

- `ares-core`: 4,678 listed tests, of which 4,677 are nonignored;
- `task22a_`: 42; `task22b_`: 0;
- `-E 'test(/^geometry::/)'`: 0;
- `-E 'test(/^mesh_slicer::/)'`: 0;
- `-E 'test(/^project_slice::/)'`: 33;
- `-E 'test(/^project::/)'`: 268 listed / nonignored;
- `config_export`: 30;
- `ares-core --test no_unapproved_dynamic_values`: 29 nonignored;
- `ares-cli`: 16 listed / 15 nonignored; `ares-wasm`: 5 listed/nonignored;
- fresh `cargo +1.91.0 nextest run -p ares-core project`: 430/430 GREEN.

Every new test begins with `task22b_`. Cumulative Task 22B counts are frozen as
4, 12, 17, 22, 28, 33, 37, 46, 52, and 58 after Packages A, P, E, C, T, F, M,
G, X, and I. The final listed `ares-core` count is 4,736, with 4,735 nonignored;
final listed/nonignored module counts are geometry 5, mesh_slicer 15,
project_slice 54, and project 285. The one unchanged ignored ares-core test is
outside those four modules. Count drift requires review before implementation
continues.

## Subagent-Driven execution and review discipline

### Pre-implementation plan review gate

Before Package A, compute this plan's exact byte count, line count, and SHA-256
and record them in the ignored Task 22B evidence ledger. Dispatch a fresh
read-only Codex reviewer against that exact hash and require literal
`VERDICT: APPROVE`.

Then use the `opencode-agent` skill's Windows helper with the default configured
model: do not pass `-m`. Write the bounded English review prompt to ignored
`.superpowers/sdd/task22b-opencode-plan-prompt.md`; require it to recheck the
exact spec/plan hashes, pinned sources, manifest, TDD sequence, commands, and
scope directly without editing or delegating. Invoke it exactly with runtime
subagent/write denial:

```powershell
$env:OPENCODE_CONFIG_CONTENT =
  '{"permission":{"task":"deny","edit":"deny"}}'
$OutputEncoding = New-Object System.Text.UTF8Encoding $false
Get-Content -Raw -Encoding UTF8 `
  -LiteralPath .superpowers/sdd/task22b-opencode-plan-prompt.md |
  powershell -NoProfile -ExecutionPolicy Bypass -File `
    C:\Users\Indexyz\Projects\dots\skills\opencode-agent\scripts\run_opencode_agent.ps1
if ($LASTEXITCODE -ne 0) { throw "default-model OpenCode plan review failed" }
```

Accept only a direct review ending in literal `VERDICT: APPROVE`. Any plan-byte
revision invalidates both plan verdicts, requires a new hash, and restarts both
reviews. Record reviewer identity, exact hash, command exit, delegation/write
denial, and verdict in the ignored ledger before implementation.

Packages are serial in the shared worktree even where their source dependencies
could theoretically overlap. Several packages revisit `assemble.rs`,
`project/transform.rs`, `mesh_slicer.rs`, `raw_intersections.rs`, or state/test
support; serial ownership preserves genuine RED/GREEN evidence and prevents
parallel agents from masking one another's changes. Parallelism is used for the
independent specification-compliance and code-quality reviewers after every
freeze, and for the three whole-implementation reviewers.

For every production package A through X:

1. dispatch one fresh implementer with only that package's owned paths, exact
   approved spec/plan hashes, prerequisites, named REDs, and acceptance commands;
2. require tests first, run the stated RED, and record exact missing-symbol or
   behavioral failures before any production edit;
3. implement only the named behavior, run the focused GREEN and common gates,
   inspect the complete owned patch, and freeze path/SHA-256 hashes;
4. dispatch a fresh specification-compliance reviewer and a different fresh
   code-quality reviewer in parallel; each must end with literal
   `VERDICT: APPROVE` for identical bytes and evidence;
5. on revision, use a bounded fixer, rerun affected and package gates, refreeze,
   and rerun both reviewers. Any byte edit invalidates both verdicts.

Package I is a test-only post-implementation acceptance package. Its exact KSR
oracle was already introduced as a genuine pre-production RED in Package X.
Package I adds independent lifecycle, repeatability, identity-shrink regression,
and mutation falsifiers; they should be GREEN against approved Package X bytes.
Any Package I failure is a genuine defect: reopen the owning production package,
apply a bounded fix, rerun its gates/reviews, then rerun Package I. Never
manufacture a RED or change production merely to make a post-implementation
falsifier fail first.

Do not commit between packages. Do not add `#[allow(...)]`, `#[expect(...)]`,
test-only production branches, placeholder lines/G-code, a feature flag, an
option-registry entry, a dependency, a public geometry API, or any legacy
fallback. Every changed/new Rust file stays below 400 physical lines.

After each package, require its exact cumulative `task22b_` count and run:

```powershell
cargo +1.91.0 nextest list -p ares-core task22b_
cargo +1.91.0 nextest run -p ares-core task22b_
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core project_slice
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
```

Starting with Package C, also run the exact geometry-module filter; starting
with Package T, run the exact mesh-slicer-module filter:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^geometry::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^mesh_slicer::/)'
```

Require 675 unchanged dynamic rows, the unchanged two-line allowlist, fixture
guards, no ignored new test, and no warning or skipped applicable failure.

## Exact tracked manifest

**Create:**

- `crates/ares-core/src/project/tests/task22b_transform.rs`;
- `crates/ares-core/src/project/load/mesh_prepare.rs`;
- `crates/ares-core/src/project/load/mesh_prepare/tests.rs`;
- `crates/ares-core/src/project/tests/model/task22b_materialization.rs`;
- `crates/ares-core/src/project/load/assemble/tests.rs`;
- `crates/ares-core/src/project/tests/model/task22b_expansion.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/coord.rs`;
- `crates/ares-core/src/geometry/tests.rs`;
- `crates/ares-core/src/geometry/tests/coord.rs`;
- `crates/ares-core/src/geometry/tests/scale.rs`;
- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/topology.rs`;
- `crates/ares-core/src/mesh_slicer/intersection.rs`;
- `crates/ares-core/src/mesh_slicer/tests.rs`;
- `crates/ares-core/src/mesh_slicer/tests/topology.rs`;
- `crates/ares-core/src/mesh_slicer/tests/facet.rs`;
- `crates/ares-core/src/mesh_slicer/tests/dispatch.rs`;
- `crates/ares-core/src/project_slice/raw_intersections.rs`;
- `crates/ares-core/src/project_slice/tests/raw_support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_preflights.rs`;
- `crates/ares-core/src/project_slice/tests/raw_transform.rs`;
- `crates/ares-core/src/project_slice/tests/raw_lifecycle.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs`;
- `docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md`;
- `docs/superpowers/plans/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md`.

**Modify:**

- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project.rs`;
- `crates/ares-core/src/project/transform.rs`;
- `crates/ares-core/src/project/model_xml.rs`;
- `crates/ares-core/src/project/load.rs`;
- `crates/ares-core/src/project/load/assemble.rs`;
- `crates/ares-core/src/project/load/volume_metadata.rs`;
- `crates/ares-core/src/project/domain.rs`;
- `crates/ares-core/src/project/tests/model.rs`;
- `crates/ares-core/src/project/tests/model/document.rs`;
- `crates/ares-core/src/project/tests/model/import.rs`;
- `crates/ares-core/src/project/tests/model/volume_defaults.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/state.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/support.rs`;
- `crates/ares-core/src/project_slice/tests/integration.rs`;
- `crates/ares-core/src/project_slice/tests/fixture.rs`;
- after whole approval only, `docs/architecture/option-parity-v4.md` and
  `docs/roadmap.md`.

**Delete:** nothing.

No other tracked path may change. In particular, keep `project/load/graph.rs`,
Cargo manifests/lockfile, dynamic baseline/allowlist, all three KSR files, CLI
and WASM signatures, browser files, old STL/model/planning/pipeline/segments/
contours modules, and unrelated user paths byte-identical. The existing graph
APIs are sufficient for build-reachable component DFS in `assemble.rs`; do not
move the check into whole-graph loading, which would reject unreachable cycles.
An indispensable extra path requires a frozen plan revision and fresh dual plan
approval; behavior outside the approved spec requires fresh dual spec approval.

---

## Package A: Add only the transform numeric seams

**Owned paths:**

- `crates/ares-core/src/project/transform.rs`;
- `crates/ares-core/src/project.rs`;
- `crates/ares-core/src/project/tests/task22b_transform.rs`.

This package supplies shared numeric primitives for import preparation and the
later raw adapter. It does not load a project or create geometry state.

### A.1: Establish four transform REDs

Add exactly:

1. `task22b_transform_removes_xyz_translation`;
2. `task22b_transform_local_translation_is_c_times_t`;
3. `task22b_transform_pretranslation_acts_after_linear`;
4. `task22b_transform_casts_matrix_and_point_before_f32_arithmetic`.

Use noncommuting scale/rotation/translation matrices. Distinguish all-translation
removal from the existing XY-only helper, `C * T(shift)` from
`T(shift) * C`, and world pretranslation from local posttranslation. The f32
test must differ from transforming in f64 and narrowing only the final point.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_transform --no-capture
```

Require four listed tests and compile failures for the missing crate-private
seams. Unrelated compile failures are test defects.

### A.2: Implement the minimal typed transform operations

Add only all-translation removal, local translation/postcomposition, world
pretranslation, and matrix-and-point f32 application needed by the approved
source slice. Reuse the existing `then` convention. Do not expose matrix rows
publicly, add decomposition/SVD, or generalize this into a transformation
framework.

Run the common package gate and require exactly 4 `task22b_` tests. Freeze and
dual-review Package A before P.

---

## Package P: Port Bambu f32 materialization and fresh-mesh preparation

**Owned paths:**

- `crates/ares-core/src/project/model_xml.rs`;
- `crates/ares-core/src/project/load.rs`;
- `crates/ares-core/src/project/load/assemble.rs`;
- `crates/ares-core/src/project/load/mesh_prepare.rs`;
- `crates/ares-core/src/project/load/mesh_prepare/tests.rs`;
- `crates/ares-core/src/project/tests/model.rs`;
- `crates/ares-core/src/project/tests/model/task22b_materialization.rs`;
- `crates/ares-core/src/project/tests/model/document.rs`;
- `crates/ares-core/src/project/tests/model/import.rs`;
- `crates/ares-core/src/project/tests/model/volume_defaults.rs`.

### P.1: Establish eight import REDs

Grow the cumulative manifest from 4 to 12 with exactly:

1. `task22b_vertex_units_materialize_through_f32_before_promotion`;
2. `task22b_vertex_unit_product_nonfinite_precedes_effective_config`;
3. `task22b_import_winding_uses_face_order_f32_and_strict_negative_flip`;
4. `task22b_import_winding_preserves_positive_zero_negative_zero_nan_and_degenerate_faces`;
5. `task22b_fresh_mesh_centering_keeps_f32_subtraction_and_f64_shift`;
6. `task22b_fresh_mesh_compensation_is_component_then_shift_and_metadata_stays_provenance`;
7. `task22b_empty_geometry_is_omitted_before_volume_metadata_association`;
8. `task22b_ksr_import_preparation_has_exact_mesh_facts`.

The unit table names micron, millimeter, centimeter, inch, foot, and meter and
requires exact f32 products for source `0.3`: `0.0003`, `0.3`, `3.0`,
`7.6200004`, `91.44`, and `300.0`. The inch `[0, 0.3]` range must produce
center coordinate 3,810,000 rather than 3,809,999. Source parse or product
nonfinite errors must be the existing bounded vertex error and must beat
effective-config resolution.

Use closed tetrahedra, fully reversed faces, planar/zero-area faces, explicit
positive/negative zero, and an overflow/NaN accumulator discriminator. Require
only strict negative final volume to swap slots 1/2. For f32 bounds
`128.0..128.00001525878906`, freeze the approved f64/f32 shifts, centered and
reconstructed bounds, and raw-center distinction. A nonidentity linear
component transform must prove `C * T(shift)` and metadata `matrix` provenance.

The empty-first/same-ID metadata test must retain only one volume and select the
first part row for that nonempty volume. KSR asserts 6,109 vertices, 12,234
triangles, positive f32 volume, exact zero shift, exact bounds, and first face
`[2,0,1]`; it also proves every zero-shift prepared vertex retains its exact
source f32 bits and reads only the 3MF.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_vertex_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_import_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_fresh_mesh_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_empty_geometry_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_ksr_import_preparation_ --no-capture
```

Require genuine missing/numeric REDs; never force a fixture failure.

### P.2: Replace the old f64 import path once

Make XML vertex fields and `ModelUnit::millimeter_factor` f32. Validate both the
parsed scalar and f32 product as finite. `mesh_prepare` consumes source faces
and f32 vertices and returns one paired prepared result containing the exactly
promoted centered `ProjectMesh` and compensated transform.

Implement the signed-volume expression in the exact specified operation order,
including zero-vector normalization, f32 accumulator, and strict negative
test. Compute f32 extrema over all vertices, promote extrema before the f64
midpoint, add exactly `-(shift as f32)` in f32, then compose
`component_transform * T(shift)` in f64. Do not retain or reconstruct the
uncentered mesh. If the complete center shift is zero, including negative zero,
skip vertex arithmetic so every source f32 bit is preserved exactly.

Wire the result at the existing assembly seam. Omit empty vertex/triangle
meshes before metadata selection and before calling preparation. Remove the
superseded f64 `project_mesh` multiplication path; do not leave it as fallback.
Update only old tests whose direct raw-mesh assertions legitimately change;
world geometry expectations must remain equal through compensation.

Run the common package gate and require exactly 12 `task22b_` tests. Freeze and
dual-review Package P before E.

---

## Package E: Bound build-reachable component expansion

**Owned paths:**

- `crates/ares-core/src/project/load/assemble.rs`;
- `crates/ares-core/src/project/load/assemble/tests.rs`;
- `crates/ares-core/src/project/tests/model.rs`;
- `crates/ares-core/src/project/tests/model/task22b_expansion.rs`.

Do not modify `project/load/graph.rs`; use its validated object and component
lookup APIs from assembly.

### E.1: Establish five expansion REDs

Grow the cumulative manifest from 12 to 17 with exactly:

1. `task22b_expanded_model_budget_accepts_limit_rejects_next_and_overflow`;
2. `task22b_component_cycle_preflight_is_iterative_build_reachable_and_precedes_materialization`;
3. `task22b_component_expansion_is_ancestry_free_and_claims_before_queue_growth`;
4. `task22b_component_expansion_is_breadth_first_not_depth_first_or_source_id_sorted`;
5. `task22b_expanded_model_budget_is_request_wide_instances_reuse_dag_and_ksr_claims_18345`.

Pure budget tests claim exactly 1,000,000 without allocating that many objects,
reject the next, and map checked overflow to the same exact error. Cycle cases
include reachable self/two-node cycles, an unreachable cycle, and proof that
cycle failure occurs before mesh materialization. A depth-32 chain ending in
32 leaves freezes linear node/edge visitation and no ancestor-path multiplier.
The BFS discriminator uses group A before leaf 3 and A's leaves 1 then 2,
requiring admitted order `[3,1,2]`. The final test spans source objects,
repeated physical instances, and KSR's `2 + 6109 + 12234` claims.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_expanded_model_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_component_ --no-capture
```

Require the five newly listed tests to fail only on missing expansion-budget,
cycle-preflight, or ancestry-free BFS behavior.

### E.2: Preflight cycles and replace ancestor-carrying BFS

Before expansion, run one explicit-stack three-color DFS over component
identities reachable from unique build roots, in declared order. Reject a gray
back-edge with the existing exact cycle error; do not reject unreachable cycles
or duplicate the graph's existing identity/target validation.

Replace the pending tuple's cloned ancestor vector with exactly path, object ID,
and accumulated transform. Create one request-wide `ExpandedModelBudget` in
project assembly. Claim root/child occurrence before enqueue; for a nonempty
leaf, checked-claim vertices plus triangles before clone/materialization.
Repeated build instances reuse the already assembled source DAG. Preserve BFS
and metadata semantics from Package P.

Keep `assemble.rs` below 400 physical lines by deleting the ancestor and old
mesh conversion code already superseded, not by creating an unapproved helper
module.

Run the common package gate and require exactly 17 `task22b_` tests. Freeze and
dual-review Package E before C.

---

## Package C: Add request-local coordinate scale and integer point domain

**Owned paths:**

- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/coord.rs`;
- `crates/ares-core/src/geometry/tests.rs`;
- `crates/ares-core/src/geometry/tests/coord.rs`;
- `crates/ares-core/src/geometry/tests/scale.rs`.

### C.1: Establish five coordinate REDs

Grow the cumulative manifest from 17 to 22 with exactly:

1. `task22b_scale_selection_uses_printable_span_threshold_ksr_and_empty_default`;
2. `task22b_request_local_scales_are_repeated_and_concurrently_isolated`;
3. `task22b_checked_coordinate_scaling_truncates_and_round_trips`;
4. `task22b_checked_coordinate_scaling_rejects_nonfinite_and_half_open_i64_range`;
5. `task22b_point_equality_and_order_are_integer_x_then_y`.

Freeze spans at 2,147 and just above, KSR's 256 mm area, empty area, positive
and negative fractional truncation, exact zero/integer round trips, both i64
range edges, nonfinite input, full-point equality, and lexicographic order.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_scale_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_request_local_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_checked_coordinate_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_point_ --no-capture
```

Require the five newly listed tests to fail only on the missing private
coordinate domain and request-local scale.

### C.2: Implement the private integer subset

Define private `Coord = i64`, a two-coordinate integer `Point`, and a small
copyable request scale. Check quotient finiteness and the f64 half-open i64
range before cast. Unscale with the same factor. Do not rely on Rust's
saturating float cast, add a process-global, expose a public API, or port later
path-domain behavior.

Run the common gate plus the exact geometry module filter and require exactly
22 cumulative Task 22B tests and 5 geometry-module tests. Freeze and
dual-review Package C before T.

---

## Package T: Port shared indexed-mesh edge topology

**Owned paths:**

- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/topology.rs`;
- `crates/ares-core/src/mesh_slicer/tests.rs`;
- `crates/ares-core/src/mesh_slicer/tests/topology.rs`.

### T.1: Establish six topology REDs

Grow the cumulative manifest from 22 to 28 with exactly:

1. `task22b_topology_pairs_opposite_oriented_neighbors`;
2. `task22b_topology_assigns_one_id_to_a_boundary_edge`;
3. `task22b_topology_pairs_two_same_oriented_uses`;
4. `task22b_topology_rejects_more_than_two_uses_before_intersection`;
5. `task22b_topology_edge_id_range_is_checked_without_large_allocation`;
6. `task22b_topology_indexing_is_deterministic`.

Require normalized vertex-pair keys, every local edge mapped, one increasing ID
per group, the explicit face/local-edge tie normalization, exact
`mesh_topology`, and exact edge-range errors. The range test uses a pure checked
ID seam rather than allocating billions of edges.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_topology_ --no-capture
```

Require all six newly listed tests to fail only on missing topology behavior.

### T.2: Implement topology before any intersection

Create three directed uses per face in source face order. Sort by normalized
key, then face/local edge only as Ares's equal-key normalization. Reject groups
larger than two before returning any index. Pair opposite or same-oriented
two-use groups and assign boundary IDs. Use safe Rust and checked conversions.
Do not weld coordinates, infer topology from float values, or call old STL
segments.

Run the common and exact geometry/mesh-slicer module gates. Require exactly 28
cumulative Task 22B tests and 6 mesh-slicer-module tests. Freeze and dual-review
Package T before F.

---

## Package F: Port ordinary facet-plane intersection

**Owned paths:**

- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/intersection.rs`;
- `crates/ares-core/src/mesh_slicer/tests.rs`;
- `crates/ares-core/src/mesh_slicer/tests/facet.rs`.

### F.1: Establish five facet REDs

Grow the cumulative manifest from 28 to 33 with exactly:

1. `task22b_facet_crossing_preserves_direction_and_endpoint_provenance`;
2. `task22b_facet_conversion_distinguishes_vertex_truncation_from_interior_floor_plus_half`;
3. `task22b_facet_single_vertex_dedup_uses_exact_id_and_strict_plane_equality`;
4. `task22b_facet_top_bottom_and_horizontal_ownership_matches_orca`;
5. `task22b_facet_rounding_preserves_zero_length_lines`.

Use a sloped triangle whose coordinate order differs from the required directed
order. Exercise positive/negative fractional inherited vertices and
positive/negative half interior intersections. Cover the specified lowest
vertex tie rule, one exact on-plane vertex, nearby non-equal f32 Z, owned
reversed top, non-retained bottom, horizontal face, and a rounding collapse.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_facet_ --no-capture
```

Require all five newly listed tests to fail only on missing facet-intersection
behavior.

### F.2: Implement the exhaustive raw vocabulary and exact algorithm

Model endpoint provenance as `Vertex(u32)` or `Edge(u32)` and retained type as
`General` or `Top`. Use strict f32 min/max/equality and the source lowest-index
selection. Deduplicate on-plane vertices by ID. Order crossing edge vertices by
ID, interpolate from f32 source values in f64, apply `floor(x + 0.5)` only to
strict interior points, and emit General from point 1 to point 0. Return owned
top immediately with reversed vertices; do not retain bottom/horizontal.
Preserve zero-length output. Add no chaining flags or coordinate sort.

Run common and module gates; require exactly 33 cumulative Task 22B tests and
11 mesh-slicer-module tests. Freeze and dual-review Package F before M.

---

## Package M: Port bounded multi-plane dispatch and raw-line budget

**Owned paths:**

- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/tests.rs`;
- `crates/ares-core/src/mesh_slicer/tests/dispatch.rs`.

### M.1: Establish four dispatch/budget REDs

Grow the cumulative manifest from 33 to 37 with exactly:

1. `task22b_multi_plane_dispatch_preserves_boundaries_duplicates_and_empty_slots`;
2. `task22b_multi_plane_dispatch_is_face_major_then_eligible_plane_major`;
3. `task22b_raw_line_budget_claims_before_append_and_checks_limit_or_overflow`;
4. `task22b_multi_plane_slicing_is_repeatably_deterministic`.

Freeze lower-bound `>= min_z`, upper-bound `> max_z`, duplicate f32 planes,
empty slots, a triangle whose eligible span is a strict subset, multiple faces,
and unsorted-by-content expected output. The pure budget/sink test accepts
exactly 1,000,000 claims, rejects the next and checked overflow, and proves a
failed claim cannot append without allocating one million records.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_multi_plane_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_raw_line_budget_ --no-capture
```

Require all four newly listed tests to fail only on missing dispatch or raw-line
budget behavior.

### M.2: Transform/index once and dispatch by binary-search span

The private slicer takes already ordered f32 planes and one transformed indexed
mesh. Build topology before output. Visit source faces in ascending index, find
the eligible half-open plane range by binary search, and visit plane indices in
ascending order. Retain every plane slot. Feed retained lines through a
zero-cost sink that claims the shared `RawIntersectionBudget` before push.
Never allocate a triangle-by-layer matrix, content-sort lines, or add threads.

Run common/module gates; require exactly 37 cumulative Task 22B tests and 15
mesh-slicer-module tests. Freeze and dual-review Package M before G.

---

## Package G: Add post-planning gates, volume ordinals, and dense-slot preflight

**Owned paths:**

- `crates/ares-core/src/project/load/volume_metadata.rs`;
- `crates/ares-core/src/project/load/assemble.rs`;
- `crates/ares-core/src/project/domain.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/state.rs`;
- `crates/ares-core/src/project_slice/raw_intersections.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_preflights.rs`.

Keep `domain.rs` below 400 lines. Retain only a small explicit-`mesh_shared`
presence field/accessor with a false default for synthetic volumes; do not put
ordinals or raw lines in project domain.

### G.1: Establish nine post-plan preflight REDs

Grow the cumulative manifest from 37 to 46 with exactly:

1. `task22b_volume_ordinals_follow_nonempty_bfs_order_and_keep_filter_gaps`;
2. `task22b_volume_ordinals_distinguish_bfs_from_dfs_and_restart_per_object_request`;
3. `task22b_mesh_shared_presence_and_repeated_numeric_keys_are_rejected_request_wide`;
4. `task22b_shared_mesh_gate_ignores_empty_occurrences_and_precedes_dense_or_coordinate_errors`;
5. `task22b_layer_range_preflight_runs_after_task22a_and_before_all_raw_geometry`;
6. `task22b_print_object_centering_gate_accepts_collapsed_xy_and_rejects_distinct_or_mismatched_groups`;
7. `task22b_dense_slot_budget_counts_only_nonempty_sliceable_volumes_request_wide`;
8. `task22b_dense_slot_budget_accepts_exact_limit_and_rejects_next_or_overflow`;
9. `task22b_raw_preflight_order_is_range_centering_shared_then_dense_slots`.

The ordinal table uses `[empty, blocker, model, modifier, enforcer, negative]`
and requires admitted ordinals 2, 3, and 5. The separate nested-group topology
requires leaf order `[3,1,2]`, each source object restarts at one, physical
instances reuse, and repeated/concurrent requests do not leak state.

Treat any `mesh_shared` key value, including `"0"`, as presence. Exercise
duplicates across paths, roots, objects, and types; empty duplicate/explicit
sharing is ignored. Range, centering, sharing, and dense conflicts freeze exact
request-wide precedence. Dense arithmetic uses pure counts for the approved
exact-limit, excess, and overflow examples and counts only nonempty model,
negative, and modifier volumes. Existing shrink gates must beat writer,
planning, and Task 22B; Package I adds that already-released behavior as a
GREEN regression rather than manufacturing a Package G RED.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_volume_ordinals_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_mesh_shared_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_shared_mesh_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_layer_range_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_print_object_centering_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_dense_slot_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_raw_preflight_ --no-capture
```

Require all nine newly listed tests to fail only on missing post-plan projection
or preflight behavior.

### G.2: Retain sharing presence and build a real preflight projection

Stop discarding the metadata key, thread only its presence through selected
volume metadata and the loader, and default synthetic volumes to absent without
changing `ProjectVolume::id()` meaning or public constructor shape.

In `raw_intersections.rs`, define private `VolumeOrdinal(NonZeroU32)` and a
projected object/volume representation. After Task 22A planning, run exactly:
range gate, centering gate, sharing gate, ordinal projection, then whole-request
dense-slot preflight. Ordinals derive from nonempty `ProjectObject::volumes()`
positions before type filtering. Retain model/negative/modifier projections in
ascending ordinal order and plan-length slot counts. Use checked arithmetic and
exact errors.

Wire this vertical package into state so all new production fields/functions
are consumed: the package-stage state may temporarily own the private projected
objects, and `slice_project` must consume them before returning incomplete.
Package X replaces that direct intermediate ownership with final intersected
objects; no projection fallback or parallel top-level vector remains afterward.

Run common/module gates; require exactly 46 cumulative Task 22B tests and 42
project-slice-module tests. Freeze and dual-review Package G before X.

---

## Package X: Construct raw center/transforms and retain real intersections

**Owned paths:**

- `crates/ares-core/src/project/transform.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/state.rs`;
- `crates/ares-core/src/project_slice/raw_intersections.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/integration.rs`;
- `crates/ares-core/src/project_slice/tests/fixture.rs`;
- `crates/ares-core/src/project_slice/tests/raw_support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_transform.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs`.

### X.1: Establish six adapter/intersection REDs

Grow the cumulative manifest from 46 to 52 with exactly:

1. `task22b_raw_center_quantizes_importer_f32_vertices_before_unscale`;
2. `task22b_raw_center_uses_f64_transforms_all_vertices_and_model_parts_only`;
3. `task22b_centered_slice_transform_composes_translation_scale_rotation_and_z_exactly`;
4. `task22b_mirrored_affine_preserves_import_normalized_indices_and_direction`;
5. `task22b_project_adapter_uses_slice_z_not_print_z_and_keeps_object_volume_identity`;
6. `task22b_ksr_fixture_matches_exact_raw_counts_components_records_and_digests`.

Use the approved tiny positive/negative center discriminators, f64 volume-scale
case, asymmetric quantization, unreferenced vertex, excluded volume types,
source instance XYZ translation, volume XY translation, object Z, noncommuting
linear transforms, and f32 cast distinction. A mirrored affine must keep the
import-normalized triangle order. Distinct `slice_z`/`print_z`, two source
objects, ordinal gaps, and every empty/nonempty slot prove no cross-wiring. The
adapter RED also supplies a finite input whose transformed component becomes
nonfinite and finite X/Y values whose scaled quotients hit both excluded range
boundaries; each must return exactly
`InvalidInput("project mesh slicing coordinate is nonfinite or outside the scaled coordinate range")`.
The KSR oracle reads only the committed 3MF and must fail before production
because real raw-intersection ownership does not yet exist; a forced fixture
failure is forbidden.

Run before production:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_raw_center_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_centered_slice_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_mirrored_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_project_adapter_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_ksr_fixture_matches_ --no-capture
```

Require all six newly listed tests to fail only on missing raw-center,
slice-transform, project-adapter, or real raw-intersection behavior.

### X.2: Complete the source-cited project adapter

Select request scale from resolved printable area only after Package G's dense
preflight. For each object, compute the raw f64 model-part box from prepared
vertices under first-source transform with all translation removed and
compensated volume transforms. Quantize/unscale center XY. Separately retain the
sole group transform's Z, remove XY, pretranslate negative center, then compose
each admitted volume. Prescale output XY only, cast matrix and prepared vertices
to f32, and pass f32 `slice_z` values to `mesh_slicer`.

Do not inspect raw linear determinant or swap indices. Build each volume's
transform/topology once. Share one request `RawIntersectionBudget`; claim via
the Package M sink before every append. Each final `IntersectedPrintObject`
owns its `PlannedPrintObject` and ordered `RawVolumeIntersections`, and every
volume owns exactly plan-length layer vectors.

Replace Package G's package-stage projected state with final intersected state.
`ProjectSliceState` owns Project, resolved config, optional exact block, and
only `Vec<IntersectedPrintObject>`. Consume all fields before the unchanged
public incomplete error. Do not clone/reload/re-resolve or expose the raw state.
In this package, update the existing Task 22A integration/fixture assertions to
reach their plan through `intersected_objects[i].plan`; keep Task 22A names and
semantics so Package X's common gate can turn GREEN before review.

### X.3: Encode and satisfy the exact KSR oracle in tests only

The test-only encoder uses the exact 26-byte NUL prefix and big-endian fixed
fields from the approved spec, including `volume_ordinal` and excluding source
leaf ID. The semantic copy sorts each layer by the complete nine-field tuple
without swapping A/B. The Ares-order encoding retains production vectors.

Before Package X can turn GREEN, require:

- stream length 5,012,035;
- semantic SHA-256
  `a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21`;
- face-order SHA-256
  `1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79`;
- one object, transform 0, one model-part volume ordinal 1, 460 slots, dense
  count 460, expanded count 18,345, normal scale, 6,109 vertices, 12,234
  triangles, 18,351 opposite-paired edge IDs, and 116,472 lines;
- maximum 3,011 lines at layer 46, f32 slice Z
  `9.300000190734863`, and 41 closed components;
- representative line/component counts and all five exact representative
  records from the spec;
- no zero-length KSR line, exact empty-range/shrink prerequisites, exact unique
  normalized `(0.0, f64::MAX, None)` layer candidate, exact 49,004-byte config
  block/hash, and public incomplete result.

Closed components use the full point-plus-provenance key with directed
in/outdegree and parallel multiplicity, never coordinates alone.

Run common/module gates; require exactly 52 cumulative Task 22B tests and 48
project-slice-module tests. Freeze and dual-review Package X before I.

---

## Package I: Prove lifecycle, repeatability, and anti-hardcoding

**Owned paths:**

- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_support.rs`;
- `crates/ares-core/src/project_slice/tests/raw_lifecycle.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs`.

No production path is owned by Package I. A failure reopens the affected
production package under the common freeze/review rule rather than authorizing
an unreviewed Package I source edit.

### I.1: Add six post-implementation acceptance tests

Grow the cumulative manifest from 52 to exactly 58 with:

1. `task22b_lifecycle_preserves_load_config_writer_task22a_and_raw_error_precedence`;
2. `task22b_private_state_owns_plan_inside_intersections_and_builds_once`;
3. `task22b_ksr_fixture_is_repeatable_config_unchanged_and_publicly_incomplete`;
4. `task22b_anti_hardcoding_vertex_mutation_changes_semantic_digest`;
5. `task22b_anti_hardcoding_printable_area_mutation_changes_scale_and_coordinates`;
6. `task22b_identity_shrink_options_precede_task22a_and_raw_state`.

Run immediately after adding the tests and before any source fix:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_lifecycle_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_private_state_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_ksr_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_anti_hardcoding_ --no-capture
cargo +1.91.0 nextest run -p ares-core task22b_identity_shrink_ --no-capture
```

These are falsifiers against already approved Package X bytes, not a new
production RED package. Require all six newly listed tests to pass. If one
fails, record the genuine failure, reopen the owning production package, and
invalidate/repeat its applicable reviews; never force a failure.

The lifecycle matrix freezes:

- archive/component/materialization/expanded-budget errors before effective
  config;
- effective config and shrink gates before writer, planning, or Task 22B;
- Bambu writer before Task 22A/22B;
- Task 22A capability/numeric/range-owned-height errors before Task 22B;
- Task 22B range before centering before sharing before dense slots before
  coordinate/topology/raw-line errors;
- a later object's shared key or dense excess before an earlier object's
  otherwise invalid coordinate/topology work;
- a supported valid Bambu or non-Bambu request builds raw state then returns
  `ProjectSlicingIncomplete`.

The state test proves one load, one resolve, one optional writer call, one plan,
one raw build, nested plan ownership, and no archive/config reconstruction.

### I.2: Reverify exact state and falsify fixture-specific behavior

Reuse Package X's fixed-width encoder and exact count/digest/record assertions.
Build the complete KSR private state twice from the same committed 3MF bytes and
require complete structural equality, both exact encodings, the unchanged
49,004-byte config block/hash, and the same public incomplete result on both
requests. No cached request-local scale, ordinal, budget, or raw vector may leak
between runs.

Mutation helpers must assert a unique replacement before writing test archive
bytes. Change one finite mesh vertex and require semantic digest change. Change
only the project settings printable-area block across 2,147, preserving the
archive API with no filename input, and require normal-to-large scale plus
coordinate/digest change. Neither mutation reads G-code or options.

For a genuine source deviation, reopen only its owning production package and
stay inside the approved manifest. Rerun affected package reviews if a source
byte changes.

Run the common/module gates, fixture filters, and CLI contract:

```powershell
cargo +1.91.0 nextest run -p ares-core task22b_ksr_
cargo +1.91.0 nextest run -p ares-core task22b_anti_hardcoding
cargo +1.91.0 nextest run -p ares-cli fixture_contract_is_stable
```

Require exactly 58 Task 22B tests, 5 geometry, 15 mesh-slicer, 54 project-slice,
285 listed/nonignored project tests, 42 unchanged Task 22A, and the final
4,736/4,735 listed/nonignored ares-core totals. Freeze and dual-review Package
I.

---

## Integrated package gate

Freeze all ten package submanifests and the complete implementation manifest.
Rerun fresh specification-compliance and code-quality reviews for every package
against integrated bytes and GREEN evidence. Each role again ends in literal
`VERDICT: APPROVE`; a correction invalidates both reviews for each affected
package.

Run:

```powershell
cargo +1.91.0 nextest list -p ares-core task22b_
cargo +1.91.0 nextest run -p ares-core task22b_
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^geometry::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^mesh_slicer::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project_slice::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::/)'
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 nextest run -p ares-cli fixture_contract_is_stable
```

Require the frozen counts, exact KSR import/raw/config oracles, and unchanged
public incomplete boundary.

## Freeze, structural audits, and whole implementation reviews

Build ignored SHA-256 manifests for every tracked/untracked implementation path
and the complete patch. Require `git diff --check` and no-index whitespace
checks for every untracked path. Reject any path outside the exact manifest.

Run exact structural audits:

```powershell
function Assert-NoRgMatch {
    param(
        [Parameter(Mandatory)] [string] $Pattern,
        [Parameter(Mandatory)] [string[]] $Paths,
        [switch] $CaseInsensitive
    )

    $rgArgs = @('-n')
    if ($CaseInsensitive) { $rgArgs += '-i' }
    $rgArgs += '--'
    $rgArgs += $Pattern
    $rgArgs += $Paths
    $hits = & rg @rgArgs
    $status = $LASTEXITCODE
    if ($status -eq 0) {
        $hits | Write-Output
        throw "unexpected structural-audit match"
    }
    if ($status -ne 1) { throw "rg structural audit failed with $status" }
}

$sourcePinPattern =
  '(/OrcaSlicer|Ares-Orca|OrcaSlicer/src|TriangleMeshSlicer\.cpp|bbs_3mf\.cpp|PrintObjectSlice\.cpp)'
$sourceCitationAllowlist = @(
    'crates/ares-core/src/gcode_input_shaping.rs',
    'crates/ares-core/src/gcode_object_labels.rs',
    'crates/ares-core/src/gcode_pressure_advance.rs',
    'crates/ares-core/src/gcode_startup.rs',
    'crates/ares-core/src/gcode_temperature_transition.rs',
    'crates/ares-core/src/gcode_writer.rs'
)
$sourcePinPaths = @(
    rg -l --glob '*.rs' -- $sourcePinPattern crates |
        ForEach-Object { $_ -replace '\\', '/' } |
        Sort-Object
)
if ($LASTEXITCODE -ne 0) { throw "source-pinning path scan failed" }
$sourcePinDrift = @(
    Compare-Object ($sourceCitationAllowlist | Sort-Object) $sourcePinPaths
)
if ($sourcePinDrift.Count -ne 0) {
    $sourcePinDrift | Format-Table | Out-String | Write-Output
    throw "source-pinning/citation path set changed"
}
git diff --exit-code -- @sourceCitationAllowlist
if ($LASTEXITCODE -ne 0) {
    throw "an allowlisted preexisting source-citation file changed"
}

$task22bNewPaths = @(
    'crates/ares-core/src/project/tests/task22b_transform.rs',
    'crates/ares-core/src/project/load/mesh_prepare.rs',
    'crates/ares-core/src/project/load/mesh_prepare/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_materialization.rs',
    'crates/ares-core/src/project/load/assemble/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_expansion.rs',
    'crates/ares-core/src/geometry.rs',
    'crates/ares-core/src/geometry/coord.rs',
    'crates/ares-core/src/geometry/tests.rs',
    'crates/ares-core/src/geometry/tests/coord.rs',
    'crates/ares-core/src/geometry/tests/scale.rs',
    'crates/ares-core/src/mesh_slicer.rs',
    'crates/ares-core/src/mesh_slicer/topology.rs',
    'crates/ares-core/src/mesh_slicer/intersection.rs',
    'crates/ares-core/src/mesh_slicer/tests.rs',
    'crates/ares-core/src/mesh_slicer/tests/topology.rs',
    'crates/ares-core/src/mesh_slicer/tests/facet.rs',
    'crates/ares-core/src/mesh_slicer/tests/dispatch.rs',
    'crates/ares-core/src/project_slice/raw_intersections.rs',
    'crates/ares-core/src/project_slice/tests/raw_support.rs',
    'crates/ares-core/src/project_slice/tests/raw_preflights.rs',
    'crates/ares-core/src/project_slice/tests/raw_transform.rs',
    'crates/ares-core/src/project_slice/tests/raw_lifecycle.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs',
    'docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md',
    'docs/superpowers/plans/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md'
)
foreach ($newPath in $task22bNewPaths) {
    if (-not (Test-Path -LiteralPath $newPath -PathType Leaf)) {
        throw "missing exact-manifest file: $newPath"
    }
    $noIndexOutput = @(
        git -c core.autocrlf=false diff --no-index --check -- /dev/null `
            $newPath 2>&1
    )
    $noIndexStatus = $LASTEXITCODE
    if ($noIndexStatus -notin 0,1 -or $noIndexOutput.Count -ne 0) {
        $noIndexOutput | Write-Output
        throw "no-index whitespace check failed for $newPath"
    }
}

$task22bProduction = @(
    'crates/ares-core/src/lib.rs',
    'crates/ares-core/src/project.rs',
    'crates/ares-core/src/project/transform.rs',
    'crates/ares-core/src/project/model_xml.rs',
    'crates/ares-core/src/project/load.rs',
    'crates/ares-core/src/project/load/assemble.rs',
    'crates/ares-core/src/project/load/mesh_prepare.rs',
    'crates/ares-core/src/project/load/volume_metadata.rs',
    'crates/ares-core/src/project/domain.rs',
    'crates/ares-core/src/geometry.rs',
    'crates/ares-core/src/geometry/coord.rs',
    'crates/ares-core/src/mesh_slicer.rs',
    'crates/ares-core/src/mesh_slicer/topology.rs',
    'crates/ares-core/src/mesh_slicer/intersection.rs',
    'crates/ares-core/src/project_slice.rs',
    'crates/ares-core/src/project_slice/state.rs',
    'crates/ares-core/src/project_slice/raw_intersections.rs'
)

$task22bTests = @(
    'crates/ares-core/src/project/tests/task22b_transform.rs',
    'crates/ares-core/src/project/load/mesh_prepare/tests.rs',
    'crates/ares-core/src/project/tests/model.rs',
    'crates/ares-core/src/project/tests/model/document.rs',
    'crates/ares-core/src/project/tests/model/import.rs',
    'crates/ares-core/src/project/tests/model/volume_defaults.rs',
    'crates/ares-core/src/project/tests/model/task22b_materialization.rs',
    'crates/ares-core/src/project/load/assemble/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_expansion.rs',
    'crates/ares-core/src/geometry/tests.rs',
    'crates/ares-core/src/geometry/tests/coord.rs',
    'crates/ares-core/src/geometry/tests/scale.rs',
    'crates/ares-core/src/mesh_slicer/tests.rs',
    'crates/ares-core/src/mesh_slicer/tests/topology.rs',
    'crates/ares-core/src/mesh_slicer/tests/facet.rs',
    'crates/ares-core/src/mesh_slicer/tests/dispatch.rs',
    'crates/ares-core/src/project_slice/tests.rs',
    'crates/ares-core/src/project_slice/tests/support.rs',
    'crates/ares-core/src/project_slice/tests/integration.rs',
    'crates/ares-core/src/project_slice/tests/fixture.rs',
    'crates/ares-core/src/project_slice/tests/raw_support.rs',
    'crates/ares-core/src/project_slice/tests/raw_preflights.rs',
    'crates/ares-core/src/project_slice/tests/raw_transform.rs',
    'crates/ares-core/src/project_slice/tests/raw_lifecycle.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs'
)

$implementationManifest = @(
    $task22bNewPaths + $task22bProduction + $task22bTests |
        Sort-Object -Unique
)
git diff --cached --quiet
if ($LASTEXITCODE -ne 0) {
    throw "implementation review must begin with an empty staging area"
}
$trackedImplementationPaths = @(git diff --name-only)
if ($LASTEXITCODE -ne 0) { throw "tracked path collection failed" }
$untrackedImplementationPaths = @(git ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw "untracked path collection failed" }
$actualImplementationPaths = @(
    $trackedImplementationPaths + $untrackedImplementationPaths |
        ForEach-Object { $_ -replace '\\', '/' } |
        Sort-Object -Unique
)
$implementationPathDrift = @(
    Compare-Object $implementationManifest $actualImplementationPaths
)
if ($implementationPathDrift.Count -ne 0) {
    $implementationPathDrift | Format-Table | Out-String | Write-Output
    throw "whole implementation paths differ from the exact manifest"
}
$implementationHashRows = @(
    foreach ($implementationPath in $implementationManifest) {
        $item = Get-Item -LiteralPath $implementationPath
        $hash = (Get-FileHash -Algorithm SHA256 `
            -LiteralPath $implementationPath).Hash.ToLowerInvariant()
        [pscustomobject]@{
            Path = $implementationPath
            Bytes = $item.Length
            Sha256 = $hash
        }
    }
)
$implementationHashRows | Format-Table -AutoSize | Out-String | Write-Output
$manifestText = (($implementationHashRows | ForEach-Object {
    "$($_.Sha256) $($_.Bytes) $($_.Path)"
}) -join "`n") + "`n"
$manifestBytes = [System.Text.UTF8Encoding]::new($false).GetBytes($manifestText)
$manifestHasher = [System.Security.Cryptography.SHA256]::Create()
try {
    $implementationManifestSha256 = [System.BitConverter]::ToString(
        $manifestHasher.ComputeHash($manifestBytes)
    ).Replace('-', '').ToLowerInvariant()
} finally {
    $manifestHasher.Dispose()
}
"implementation manifest SHA-256: $implementationManifestSha256" |
    Write-Output

Assert-NoRgMatch `
    '(ksr_fdmtest_v4|698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9|10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3|33c99ee71594ed7f80b44abc3007df8e9ae4ec0800411e3b5dba500f47fd085b|b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8|a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21|1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79|ares-task22b-raw-state-v1|183_?007|6_?339_?134|456_?004|49_?004|5_?012_?035|6_?109|12_?234|18_?351|18_?345|116_?472|3_?011|\b460\b|17_?530_?508|25_?999_?317|17_?983_?121|25_?954_?736|37_?500_?000|33_?000_?000|37_?469_?924|33_?343_?825|17_?043_?610|26_?369_?232|17_?652_?542|26_?396_?576|2_?196_?466|30_?303_?541|2_?201_?466|options-v242|generated by)' `
    $task22bProduction `
    -CaseInsensitive
Assert-NoRgMatch `
    '(ksr_fdmtest_v4\.gcode|options-v242\.json|/OrcaSlicer|Ares-Orca|OrcaSlicer/src)' `
    $task22bTests

function Assert-ExactFile {
    param(
        [Parameter(Mandatory)] [string] $Path,
        [Parameter(Mandatory)] [long] $Bytes,
        [Parameter(Mandatory)] [string] $Sha256
    )
    $item = Get-Item -LiteralPath $Path
    $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash
    if ($item.Length -ne $Bytes -or $actualHash -ne $Sha256) {
        throw "exact file guard failed: $Path"
    }
}

Assert-ExactFile 'tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf' `
    183007 '698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9'
Assert-ExactFile 'tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode' `
    6339134 '10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3'
Assert-ExactFile 'tests/ksr_fdmtest_v4/options-v242.json' `
    456004 '33C99EE71594ED7F80B44ABC3007DF8E9AE4EC0800411E3B5DBA500F47FD085B'
Assert-ExactFile 'scripts/dynamic_value_baseline.txt' `
    76581 '0DCEA4C112EF10F0D6E8C8EE7F63CFEF1831D7C2AE2E399016F1E38372543BE7'
Assert-ExactFile 'scripts/dynamic_value_allowlist.toml' `
    101 '6B9C3BA6A1C52118A14D66F607CF85A9D13C27185B1FA22D670983E9371A94B6'

foreach ($rowGuard in @(
    @{ Path = 'scripts/dynamic_value_baseline.txt'; Lf = 675 },
    @{ Path = 'scripts/dynamic_value_allowlist.toml'; Lf = 2 }
)) {
    $guardBytes = [System.IO.File]::ReadAllBytes($rowGuard.Path)
    $lfCount = @($guardBytes | Where-Object { $_ -eq 10 }).Count
    $crCount = @($guardBytes | Where-Object { $_ -eq 13 }).Count
    if ($lfCount -ne $rowGuard.Lf -or $crCount -ne 0) {
        throw "LF row guard failed: $($rowGuard.Path)"
    }
}

function Get-NextestInventory {
    param([Parameter(Mandatory)] [string] $Package)
    $jsonText = & cargo +1.91.0 nextest list -p $Package -T json
    if ($LASTEXITCODE -ne 0) { throw "nextest inventory failed: $Package" }
    $inventory = $jsonText | ConvertFrom-Json
    $cases = @(
        foreach ($suite in $inventory.'rust-suites'.PSObject.Properties) {
            foreach ($test in $suite.Value.testcases.PSObject.Properties) {
                [pscustomobject]@{
                    Suite = $suite.Name
                    Name = $test.Name
                    Ignored = [bool] $test.Value.ignored
                }
            }
        }
    )
    [pscustomobject]@{
        Listed = [int] $inventory.'test-count'
        Cases = $cases
    }
}

function Assert-ExactCount {
    param(
        [Parameter(Mandatory)] [string] $Label,
        [Parameter(Mandatory)] [int] $Actual,
        [Parameter(Mandatory)] [int] $Expected
    )
    if ($Actual -ne $Expected) {
        throw "$Label count drift: expected $Expected, got $Actual"
    }
}

$coreInventory = Get-NextestInventory 'ares-core'
$coreCases = @($coreInventory.Cases)
$coreNonignored = @($coreCases | Where-Object { -not $_.Ignored })
$task22aCases = @(
    $coreCases | Where-Object { $_.Name -match '(^|::)task22a_[^:]*$' }
)
$task22bCases = @(
    $coreCases | Where-Object { $_.Name -match '(^|::)task22b_[^:]*$' }
)
Assert-ExactCount 'ares-core listed' $coreInventory.Listed 4736
Assert-ExactCount 'ares-core case enumeration' $coreCases.Count 4736
Assert-ExactCount 'ares-core nonignored' $coreNonignored.Count 4735
Assert-ExactCount 'task22a' $task22aCases.Count 42
Assert-ExactCount 'task22b' $task22bCases.Count 58
Assert-ExactCount 'ignored task22b' `
    @($task22bCases | Where-Object Ignored).Count 0
Assert-ExactCount 'geometry' `
    @($coreCases | Where-Object { $_.Name -like 'geometry::*' }).Count 5
Assert-ExactCount 'mesh_slicer' `
    @($coreCases | Where-Object { $_.Name -like 'mesh_slicer::*' }).Count 15
Assert-ExactCount 'project_slice' `
    @($coreCases | Where-Object { $_.Name -like 'project_slice::*' }).Count 54
Assert-ExactCount 'project' `
    @($coreCases | Where-Object { $_.Name -like 'project::*' }).Count 285
Assert-ExactCount 'config_export' `
    @($coreCases | Where-Object { $_.Name -like '*config_export*' }).Count 30
$dynamicCases = @(
    $coreCases | Where-Object {
        $_.Suite -eq 'ares-core::no_unapproved_dynamic_values'
    }
)
Assert-ExactCount 'dynamic listed' $dynamicCases.Count 30
Assert-ExactCount 'dynamic nonignored' `
    @($dynamicCases | Where-Object { -not $_.Ignored }).Count 29

$cliInventory = Get-NextestInventory 'ares-cli'
Assert-ExactCount 'ares-cli listed' $cliInventory.Listed 16
Assert-ExactCount 'ares-cli nonignored' `
    @($cliInventory.Cases | Where-Object { -not $_.Ignored }).Count 15
$wasmInventory = Get-NextestInventory 'ares-wasm'
Assert-ExactCount 'ares-wasm listed' $wasmInventory.Listed 5
Assert-ExactCount 'ares-wasm nonignored' `
    @($wasmInventory.Cases | Where-Object { -not $_.Ignored }).Count 5

$task22bRustManifest = @(
    $task22bProduction + $task22bTests |
        Where-Object { $_ -like '*.rs' } |
        Sort-Object -Unique
)
foreach ($rustPath in $task22bRustManifest) {
    $physicalLines = [System.IO.File]::ReadAllLines($rustPath).Length
    if ($physicalLines -ge 400) {
        throw "$rustPath has $physicalLines physical lines; limit is below 400"
    }
}

$specificLocCaps = @{
    'crates/ares-core/src/project/load/assemble.rs' = 400
    'crates/ares-core/src/project/domain.rs' = 400
    'crates/ares-core/src/project_slice/raw_intersections.rs' = 360
    'crates/ares-core/src/mesh_slicer.rs' = 300
    'crates/ares-core/src/mesh_slicer/topology.rs' = 240
    'crates/ares-core/src/mesh_slicer/intersection.rs' = 280
}
foreach ($cap in $specificLocCaps.GetEnumerator()) {
    $physicalLines = [System.IO.File]::ReadAllLines($cap.Key).Length
    if ($physicalLines -ge $cap.Value) {
        throw "$($cap.Key) has $physicalLines lines; limit is below $($cap.Value)"
    }
}

$focusedTestChildren = @(
    'crates/ares-core/src/project/tests/task22b_transform.rs',
    'crates/ares-core/src/project/load/mesh_prepare/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_materialization.rs',
    'crates/ares-core/src/project/load/assemble/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_expansion.rs',
    'crates/ares-core/src/geometry/tests/coord.rs',
    'crates/ares-core/src/geometry/tests/scale.rs',
    'crates/ares-core/src/mesh_slicer/tests/topology.rs',
    'crates/ares-core/src/mesh_slicer/tests/facet.rs',
    'crates/ares-core/src/mesh_slicer/tests/dispatch.rs',
    'crates/ares-core/src/project_slice/tests/raw_support.rs',
    'crates/ares-core/src/project_slice/tests/raw_preflights.rs',
    'crates/ares-core/src/project_slice/tests/raw_transform.rs',
    'crates/ares-core/src/project_slice/tests/raw_lifecycle.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs'
)
foreach ($testPath in $focusedTestChildren) {
    $physicalLines = [System.IO.File]::ReadAllLines($testPath).Length
    if ($physicalLines -ge 390) {
        throw "$testPath has $physicalLines lines; focused-test limit is below 390"
    }
}
```

The source-pinning scan must equal the exact six-path citation allowlist; the
production-hardcoding and test-reference scans must each complete with the
helper's exact no-match status. The tests may `include_bytes!` the committed
project 3MF through existing fixture support; no disallowed reference path may
appear in a new Task 22B test.

Audit all added production/test lines and full new files for:

- no fixture name/hash/count/timestamp/generated-by branch in production;
- no reference G-code/options access and no upstream source opening in tests;
- no generic JSON value/map, runtime registry/dispatch, profile-label discovery,
  archive reload, C++ binding, Orca process, filesystem, UI, terminal, OpenGL,
  platform-specific core code, `unsafe`, Rayon, or native threading;
- no use of legacy `stl`, `model`, `planning`, `segments`, `contours`, or
  `pipeline` in the project path;
- exact f32 import/winding/centering, build-reachable iterative DFS,
  ancestry-free BFS, request-wide budgets/preflight order, ordinal gaps,
  request-local scale, checked conversion, raw-center/slice transforms,
  topology-before-intersection, strict facet ownership, face-major dispatch,
  slice-Z use, line-budget-before-append, and nested ownership;
- no second mirror flip, content sort, coordinate endpoint sort, f64 import
  fallback, unfiltered range slicing, fresh shared-mesh fallback, or G-code;
- no added suppression or test-only production behavior;
- every changed/new Rust file below 400 physical lines. Keep targets:
  `assemble.rs < 400`, `domain.rs < 400`, `raw_intersections.rs < 360`,
  `mesh_slicer.rs < 300`, `topology.rs < 240`, `intersection.rs < 280`,
  and every focused test child `< 390`.

Prove dynamic files, all three fixtures/oracles, `graph.rs`, Cargo files, old
scaffolds, CLI/WASM/browser sources, and unrelated user paths byte-identical.

Run the fresh local implementation release matrix:

```powershell
function Assert-NativeSuccess {
    param([Parameter(Mandatory)] [string] $Step)
    if ($LASTEXITCODE -ne 0) {
        throw "$Step failed with native exit code $LASTEXITCODE"
    }
}

cargo +1.91.0 nextest run --workspace
Assert-NativeSuccess 'workspace nextest'
cargo +1.91.0 fmt --all -- --check
Assert-NativeSuccess 'rustfmt'
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
Assert-NativeSuccess 'clippy'
cargo +1.91.0 check --workspace --all-targets --all-features
Assert-NativeSuccess 'workspace check'
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
Assert-NativeSuccess 'ares-core wasm check'
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
Assert-NativeSuccess 'ares-wasm wasm check'
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
Assert-NativeSuccess 'ares-wasm release build'
cargo +1.91.0 nextest run -p ares-cli
Assert-NativeSuccess 'ares-cli nextest'
cargo +1.91.0 install --locked wasm-bindgen-cli --version 0.2.121
Assert-NativeSuccess 'wasm-bindgen install'
wasm-bindgen --version
Assert-NativeSuccess 'wasm-bindgen version'
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm `
  --target web --out-dir target/wasm-browser
Assert-NativeSuccess 'wasm-bindgen browser output'
npm --prefix crates/ares-wasm/tests/browser ci
Assert-NativeSuccess 'browser npm ci'
npm --prefix crates/ares-wasm/tests/browser audit --audit-level=low
Assert-NativeSuccess 'browser npm audit'
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
Assert-NativeSuccess 'browser Chromium install'
npm --prefix crates/ares-wasm/tests/browser test
Assert-NativeSuccess 'browser Playwright test'
```

Require `wasm-bindgen 0.2.121`, zero npm vulnerabilities, and the real-project
headless Chromium test GREEN. On Windows capture `$LASTEXITCODE` immediately
and do not use Playwright `--with-deps` locally.

Dispatch three fresh reviewers against the identical frozen manifest, patch,
and evidence:

1. whole-specification implementation reviewer: literal
   `VERDICT: APPROVE`;
2. whole-code-quality reviewer: literal `VERDICT: APPROVE`;
3. default-model OpenCode implementation reviewer invoked without `-m`, with
   runtime inline permissions `task=deny` and `edit=deny`: literal
   `VERDICT: APPROVE`.

Any revision requires a focused regression, affected and whole checks, rebuilt
hashes, and all three fresh whole reviews. Do not update tracked architecture or
roadmap documentation before all three approve.

## Documentation gate

Only after whole implementation approval, modify:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

First correct any stale Task 22A release record to commit
`91fc19f1dbfc85d21431791d2d5acb78af818671` and Tier 1 run `29543841835`.
Then document only approved Task 22B behavior:

- pinned upstream raw-intersection boundary and private modules/ownership;
- f32 import, winding, fresh centering, empty omission, bounded graph expansion,
  and explicit shared-centering gate;
- request-local scale, raw center/slicing transforms, ordinal/topology/facet/
  dispatch semantics, and generic million-unit/slot/line boundaries;
- KSR 6,109/12,234/18,351/460/116,472 evidence and both exact raw digests;
- unchanged config block and public `ProjectSlicingIncomplete` boundary;
- explicit deferral of shared-mesh reuse, nonempty ranges, chaining, paths,
  Clipper, regions, surfaces, toolpaths, G-code, and final parity.

Do not call Task 22B released before exact-pushed-SHA Tier 1. Require a fresh
documentation reviewer to end with:

```text
ROLE: DOCUMENTATION
VERDICT: APPROVE
```

Revise/re-review until approved. Add docs to the final manifest and rerun the
complete focused gates and local release matrix from approved documentation
bytes. Any implementation byte change invalidates whole and documentation
reviews.

## Conventional commit, push, and exact-SHA Tier 1

Apply the Conventional Commits skill only after all approvals and the fresh
post-documentation matrix are GREEN.

Stage only the frozen manifest; never use `git add -A`:

```powershell
$reviewedManifest = @(
    'crates/ares-core/src/project/tests/task22b_transform.rs',
    'crates/ares-core/src/project/load/mesh_prepare.rs',
    'crates/ares-core/src/project/load/mesh_prepare/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_materialization.rs',
    'crates/ares-core/src/project/load/assemble/tests.rs',
    'crates/ares-core/src/project/tests/model/task22b_expansion.rs',
    'crates/ares-core/src/geometry.rs',
    'crates/ares-core/src/geometry/coord.rs',
    'crates/ares-core/src/geometry/tests.rs',
    'crates/ares-core/src/geometry/tests/coord.rs',
    'crates/ares-core/src/geometry/tests/scale.rs',
    'crates/ares-core/src/mesh_slicer.rs',
    'crates/ares-core/src/mesh_slicer/topology.rs',
    'crates/ares-core/src/mesh_slicer/intersection.rs',
    'crates/ares-core/src/mesh_slicer/tests.rs',
    'crates/ares-core/src/mesh_slicer/tests/topology.rs',
    'crates/ares-core/src/mesh_slicer/tests/facet.rs',
    'crates/ares-core/src/mesh_slicer/tests/dispatch.rs',
    'crates/ares-core/src/project_slice/raw_intersections.rs',
    'crates/ares-core/src/project_slice/tests/raw_support.rs',
    'crates/ares-core/src/project_slice/tests/raw_preflights.rs',
    'crates/ares-core/src/project_slice/tests/raw_transform.rs',
    'crates/ares-core/src/project_slice/tests/raw_lifecycle.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/encoding.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/closed_components.rs',
    'crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs',
    'docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md',
    'docs/superpowers/plans/2026-07-16-ksr-fdmtest-v4-task22b-raw-mesh-intersections.md',
    'crates/ares-core/src/lib.rs',
    'crates/ares-core/src/project.rs',
    'crates/ares-core/src/project/transform.rs',
    'crates/ares-core/src/project/model_xml.rs',
    'crates/ares-core/src/project/load.rs',
    'crates/ares-core/src/project/load/assemble.rs',
    'crates/ares-core/src/project/load/volume_metadata.rs',
    'crates/ares-core/src/project/domain.rs',
    'crates/ares-core/src/project/tests/model.rs',
    'crates/ares-core/src/project/tests/model/document.rs',
    'crates/ares-core/src/project/tests/model/import.rs',
    'crates/ares-core/src/project/tests/model/volume_defaults.rs',
    'crates/ares-core/src/project_slice.rs',
    'crates/ares-core/src/project_slice/state.rs',
    'crates/ares-core/src/project_slice/tests.rs',
    'crates/ares-core/src/project_slice/tests/support.rs',
    'crates/ares-core/src/project_slice/tests/integration.rs',
    'crates/ares-core/src/project_slice/tests/fixture.rs',
    'docs/architecture/option-parity-v4.md',
    'docs/roadmap.md'
)

git status --short
if ($LASTEXITCODE -ne 0) { throw "git status failed" }
git diff --check
if ($LASTEXITCODE -ne 0) { throw "unstaged whitespace check failed" }
git add -- @reviewedManifest
if ($LASTEXITCODE -ne 0) { throw "exact-manifest staging failed" }
git diff --cached --name-status
if ($LASTEXITCODE -ne 0) { throw "staged manifest listing failed" }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw "staged whitespace check failed" }
$stagedPaths = @(git diff --cached --name-only)
if ($LASTEXITCODE -ne 0) { throw "staged path collection failed" }
$manifestDrift = @(
    Compare-Object ($reviewedManifest | Sort-Object) ($stagedPaths | Sort-Object)
)
if ($manifestDrift.Count -ne 0) {
    $manifestDrift | Format-Table | Out-String | Write-Output
    throw "staged paths differ from the exact reviewed manifest"
}
```

Confirm ignored evidence, generated WASM/npm output, pinned Orca checkout,
dynamic files, fixture/reference/oracle files, graph/old scaffolds, and
unrelated user changes are not staged. Create the reviewed conventional commit:

```powershell
git commit -m 'feat(slicing): retain raw project intersections'
if ($LASTEXITCODE -ne 0) { throw "conventional commit failed" }
```

Push normally without force:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
if ($LASTEXITCODE -ne 0) { throw "branch push failed" }
```

If remote advanced, fetch/rebase without dropping user work, rerun relevant
verification, and push normally. Require local/tracking/direct remote SHA
identity and clean status:

```powershell
$branch = 'codex/ksr-fdmtest-v4-parity'
$local = git rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw "local SHA lookup failed" }
$tracking = git rev-parse "origin/$branch"
if ($LASTEXITCODE -ne 0) { throw "tracking SHA lookup failed" }
$direct = ((git ls-remote origin "refs/heads/$branch") -split '\s+')[0]
if ($LASTEXITCODE -ne 0) { throw "direct remote SHA lookup failed" }
if ($local -ne $tracking -or $local -ne $direct) {
    throw "local, tracking, and direct remote SHAs differ"
}
$remainingStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw "post-push status failed" }
if ($remainingStatus.Count -ne 0) {
    $remainingStatus | Write-Output
    throw "post-push worktree is not clean"
}
```

Locate only the Tier 1 push run whose `headSha` equals `$local`, watch it to
completion, and require all five jobs GREEN:

```powershell
$ErrorActionPreference = 'Stop'
$deadline = (Get-Date).AddMinutes(5)
$matchingRuns = @()
do {
    $runJson = gh run list --workflow tier1.yml --branch $branch `
      --commit $local --event push `
      --json databaseId,headSha,status,conclusion,createdAt --limit 10
    if ($LASTEXITCODE -ne 0) { throw "Tier 1 run lookup failed" }
    $matchingRuns = @(
        ($runJson | ConvertFrom-Json) |
            Where-Object { $_.headSha -eq $local }
    )
    if ($matchingRuns.Count -gt 1) {
        throw "more than one Tier 1 push run matched the exact SHA"
    }
    if ($matchingRuns.Count -eq 0) { Start-Sleep -Seconds 5 }
} while ($matchingRuns.Count -eq 0 -and (Get-Date) -lt $deadline)

if ($matchingRuns.Count -ne 1) {
    throw "exact-SHA Tier 1 push run did not appear before the deadline"
}
$runId = [long] $matchingRuns[0].databaseId
gh run watch $runId --exit-status
if ($LASTEXITCODE -ne 0) { throw "exact-SHA Tier 1 run failed" }
$runViewJson = gh run view $runId --json headSha,conclusion,jobs
if ($LASTEXITCODE -ne 0) { throw "Tier 1 result readback failed" }
$runView = $runViewJson | ConvertFrom-Json
if ($runView.headSha -ne $local -or $runView.conclusion -ne 'success') {
    throw "Tier 1 readback does not prove exact-SHA success"
}
$requiredJobs = @(
    'format',
    'ubuntu-latest',
    'wasm',
    'macos-latest',
    'windows-latest'
)
$jobNames = @($runView.jobs | ForEach-Object { $_.name } | Sort-Object)
$jobDrift = @(Compare-Object ($requiredJobs | Sort-Object) $jobNames)
if ($jobDrift.Count -ne 0) {
    $jobDrift | Format-Table | Out-String | Write-Output
    throw "Tier 1 job manifest differs from the required five jobs"
}
$failedJobs = @($runView.jobs | Where-Object { $_.conclusion -ne 'success' })
if ($failedJobs.Count -ne 0) {
    $failedJobs | Select-Object name,conclusion | Format-Table | Out-String |
        Write-Output
    throw "one or more Tier 1 jobs did not succeed"
}
```

Required jobs are `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
`windows-latest`. Only then record Task 22B as released in ignored evidence.
The persistent complete G-code-parity goal remains active.

## Plan exit criteria

This plan is complete only when:

- exact spec and plan bytes were dual-approved before implementation;
- production Packages A, P, E, C, T, F, M, G, and X each followed genuine
  test-first RED/GREEN, Package I added its six post-implementation GREEN
  falsifiers without a manufactured RED, and every package received independent
  spec-compliance and quality approval;
- all 58 frozen Task 22B tests, 42 Task 22A tests, module/config/dynamic
  regressions, and exact KSR import/raw/config oracles are GREEN;
- source-pinning, production-hardcoding, reference-access, manifest, LOC,
  fixture, dynamic, and old-scaffold audits are clean;
- whole specification, whole quality, default OpenCode, and documentation
  reviews approved identical applicable bytes;
- the fresh native/WASM/browser release matrix passed after documentation;
- only the exact frozen manifest was conventionally committed and pushed
  normally;
- local/tracking/direct SHAs match and all five exact-pushed-SHA Tier 1 jobs are
  GREEN.

**Status: DRAFT — production and test implementation is forbidden until a
fresh independent Codex plan reviewer and the required default-model OpenCode
plan reviewer both return literal `VERDICT: APPROVE` for these exact bytes.**
