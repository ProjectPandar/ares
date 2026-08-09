# Task 22O.38 — Direct bridge-direction implementation plan

## Goal, approved boundary, and baseline

Implement only the approved contract in
`docs/superpowers/specs/2026-08-09-ksr-fdmtest-v4-task22o38-direct-bridge-direction.md`:
the independently callable
`detect_bridging_direction(const Lines &, const Polygons &)` from pinned Orca
v2.4.2 `BridgeDetector.hpp:75-119`, including its exact
`PrincipalComponents2D.cpp:8-138`, `Line.hpp:180`, and pinned Eigen 5.0.1
`Dot.h:66-100` / `BinaryFunctors.h:415-426` dependencies. O38 freezes
mixed-width PCA and deterministic MSVC STL 14.44 direction-order behavior as a
crate-private geometry prerequisite. `LayerRegion.cpp:262-308` composition,
external-surface activation, Options, adapters, and G-code remain deferred.

Exact predecessor O37 is released as implementation/documentation commits
`a0caa5a`/`4d83d15`; Tier-1 run `31291016394` passed exactly five jobs and both
browser executions at
`4d83d15832c7905d7ea9727d14c07c5a75eb7312`.

Tasks 1-10 are complete. The compiling RED, pinned original-Orca CLI/helper,
audited MSVC model, frozen Rust body, test-only Clippy repair, repaired one-at-
a-time campaign, exact restoration, complete native/WASM/static/rollback
matrix, both final review tracks, separate commits, push, and exact-SHA ship
gate all pass. Implementation/documentation commits are `04920e0`/`2d6154d`;
Tier-1 run `31303115603` passed exactly five jobs and both browser executions
at `2d6154d401c3c954bed69de6ba631a53af05f1a3`. The authoritative run JSON is
archived only at `/tmp/task22o38-tier1-exact-sha.json`; no tracked O38 byte
changed after that run. O38 is released but remains inactive, and public
slicing still stops after O26 with `ProjectSlicingIncomplete`. Its direct
`LayerRegion.cpp:262-308` consumer is now locally implemented only inside the
inactive crate-private O39 boundary; no lifecycle or public adapter is added.

Success means the exact crate-private three-argument API, literal signed `f32`
PCA formulas, explicit scale forwarding, source normal/quantization/cost/minimum
order, first-emplace semantics, and one platform-neutral adapter for the
audited MSVC STL 14.44 x64 map iteration order. No host hash order, platform
branch, project-slice caller, or invented pipeline is allowed.

## Sole-writer and evidence contract

Use one delegated worker session as sole writer for every Rust/test edit,
witness repair, one-at-a-time mutation, and exact byte restoration. The parent
may read, run commands, diagnose and authorize RED/GREEN/mutations, write only
the approved documentation and notice, perform reviews, commit, push, and
inspect CI. No second Rust/test writer is allowed in the active worktree.

Raw Orca/MSVC helper/model source, binaries, generated output, and mutation logs
stay under `/tmp`. `.pi-subagents/`, `target/`, `/tmp`, generated G-code, and
serialized diagnostics remain untracked/unstaged.

## Exact path allowlist

Rust:

1. `crates/ares-core/src/geometry.rs` — module registration, crate-private
   reexport, exact function-shape assertion;
2. new `crates/ares-core/src/geometry/bridge_direction.rs` — direct helper,
   private vector arithmetic and MSVC order adapter, at most 220 LOC;
3. new
   `crates/ares-core/src/geometry/bridge_direction/principal_components.rs` —
   PCA formulas, at most 220 LOC;
4. `crates/ares-core/src/geometry/tests.rs` — ordinary module registration;
5. new `crates/ares-core/src/geometry/tests/bridge_direction.rs` — focused
   helpers and ordinary submodules, at most 140 LOC;
6. new `crates/ares-core/src/geometry/tests/bridge_direction/pca.rs` — PCA and
   empty-edge tests, at most 300 LOC;
7. new
   `crates/ares-core/src/geometry/tests/bridge_direction/selection.rs` —
   nonempty direction and MSVC-order tests, at most 300 LOC.

Documentation/provenance:

1. O37 spec and plan release-state corrections;
2. O38 spec and this plan;
3. `docs/roadmap.md`;
4. `docs/architecture/option-parity-v4.md`;
5. `THIRD_PARTY_NOTICES.md`, only broadening the existing MSVC STL notice for
   O38 `xhash`/`type_traits` source/tag provenance.

No `line.rs`, polygon/ExPolygon/Clipper kernel, project-slice code/test,
manifest/lock/dependency/license text, lifecycle/stage/predecessor, adapter,
workflow, golden, fixture expectation, or G-code file may change.

## Task 1 — Freeze baseline and exact source audit

Before Rust changes:

1. record `HEAD == origin/main ==
   4d83d15832c7905d7ea9727d14c07c5a75eb7312`;
2. record pinned Orca HEAD
   `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
3. require clean tracked/index state, allowing only known untracked
   `.pi-subagents/`;
4. record current allowlist existence/LOC, baseline patch/status, and SHA-256
   under `/tmp`;
5. inspect every cited source range and the deferred `LayerRegion` call site.

Fetch official `microsoft/STL` tag `vs-2022-17.14` only under `/tmp`. Record tag
object `94b3a6df7fa03423d2df5b936b2cfa5a8da243b2`, peeled commit
`1f6e5b16ec02216665624c1e762f3732605cf2b4`, and exact audited files:

- `stl/inc/xhash` SHA-256
  `b5b183c4fb05fa5c1079a6eb79b7de6b395bd5cb405c09832820e89e82423435`;
- `stl/inc/type_traits` SHA-256
  `357e102b4e6ab85a864980a01bba28440791311df288b8987b580e577c928d5c`.

Fetch the pinned Eigen 5.0.1 archive only under `/tmp`; verify the
`Eigen.cmake:7-8` SHA-256
`0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`.
Audit `Dot.h:66-100` and `BinaryFunctors.h:415-426`: squared norm is the ordered
sum of squares, `normalized()` retains the original vector unless `z > 0`, and
the positive branch uses `component / sqrt(z)` through scalar quotient, not a
reciprocal multiply or stable norm.

Audit and archive exact branches for 64-bit FNV-1a/hash<double>, `-0.0`
canonicalization, `_Min_buckets`, unique `emplace`, `_Find_last`, node insertion,
rehash threshold/growth, and `_Forced_rehash`. The proprietary compiler/toolset
is unavailable locally; state that limitation exactly rather than claiming an
MSVC executable run.

Run the pinned original Orca CLI on the KSR 3MF in the established disposable
flow. Require exit 0, success result metadata, and nonzero generated output
size; delete generated G-code without reading its content.

## Task 2 — Build disposable source and order oracles

Under `/tmp`, build one C++ helper from the exact pinned Orca direct helper and
PCA implementation. Exercise:

- empty/degenerate/nonpositive area;
- axis-aligned and covariance-bearing polygons at Normal/LargeBed scales;
- non-axis-aligned `f32` and `f64` vectors that pin Eigen's ordered square,
  square-root, strict `z > 0`, and scalar-division result bits;
- single/multiple non-tied floating-edge candidates;
- duplicate quantized directions whose selected result is order independent.

Compile and run Debug and `NDEBUG` from one helper source. Require byte-identical
complete direction/cost/PCA bit output for every committed non-order-sensitive
literal.

Separately write an independent disposable control-flow model from the audited
`vs-2022-17.14` files. It must use explicit IEEE-754 little-endian bytes and
64-bit FNV-1a, eight initial buckets, first-emplace equality, occupied-bucket
front insertion, the exact growth rule, and rehash group reversal. Exercise:

- `+0.0`/`-0.0` hashing;
- duplicate first-emplace;
- empty and occupied buckets;
- equal-cost tie order;
- eight entries before growth, the ninth-entry 8→64 rehash;
- a post-rehash collision whose order distinguishes within-bucket reversal from
  preservation.

Archive model source, audited branch notes, inputs, complete outputs, and exits
under `/tmp`. Compare the model independently to hand-traced source transitions
before transcribing behavior-named Rust literals. GCC/host `unordered_map`
iteration is explicitly not evidence. Any disagreement blocks GREEN.

## Task 3 — Sole writer installs API, stub, and focused tests

The sole worker edits only the seven Rust paths. Add:

```rust
pub(crate) fn detect_bridging_direction(
    floating_edges: &[Line],
    overhang_area: &[Polygon],
    scale: CoordinateScale,
) -> ((f64, f64), f64);
```

Register only ordinary modules and the exact function-pointer shape. The
temporary production body returns `((1.0, 1.0), f64::MAX)` and does nothing
else. The PCA submodule and final MSVC adapter body remain absent or stubbed as
needed for compilation; no production test seam is added.

Every focused test name starts with `task22o38_`, making the Task 4 Nextest
filter exact. Focused split tests freeze:

1. exact empty-edge PCA fallback, axis-aligned, covariance, orientation, scale,
   threshold and normalization behavior;
2. one horizontal/vertical edge and complete direction/cost bits;
3. multiple candidates, duplicate first-emplace, reversed normals, strict
   minimum, input/cost order;
4. audited MSVC empty/occupied bucket, tie, ninth-entry rehash and post-rehash
   collision vectors;
5. trusted empty-polygon and zero-length arithmetic behavior;
6. exact API/result shape and crate-private visibility.

Every expected vector is complete and behavior-named. No test ingests Orca/MSVC
source text/hash/line numbers or serialized oracle blobs.

## Task 4 — Capture authoritative compiling RED

After rustfmt and any witness-only compile repair, while the return-only stub is
still present, the parent runs:

```bash
cargo fmt --all
cargo nextest run -p ares-core task22o38
```

Archive real output under `/tmp`. Compilation must succeed. List each
body-dependent failure that reaches the stub; disclose any stub-equivalent
pathological witness. Do not reconstruct RED after installing the body.

Only after parent authorization may the same worker replace the stub.

## Task 5 — Install the frozen source body

The worker installs only the reviewed body/submodule:

1. empty edges delegate once to PCA with unchanged scale;
2. PCA executes the literal source expression trees and association for ordered
   `f32` triangle/moment accumulation and the mixed-width eigensystem;
3. both f32 pc2 and f64 line normals reproduce Eigen 5.0.1: ordered squared
   norm, strict `z > 0`, scalar division by `sqrt(z)`, otherwise unchanged;
4. nonempty lines compute `(dy,-dx)`, normalize f64, quantize with
   `ceil(atan2*1000)`, and first-emplace through the private fixed-MSVC adapter;
5. create costs in adapter list order;
6. accumulate original line order × candidate order in f64;
7. select only on strict `<` from `(1,1)/MAX` and rotate `(y,-x)`;
8. return direct direction/cost with no validation beyond Eigen's literal
   normalization branch, and no error, retry, or fallback.

Run rustfmt, focused debug/release, complete geometry tests, and O37/O36
regressions. Repair only incorrect witnesses; do not alter frozen semantics to
fit a test.

## Task 6 — Audit and one-at-a-time mutations

Audit exact arithmetic width and cast points, source loop order, first-emplace,
FNV bytes/constants, bucket list transitions, scale flow, strict comparisons,
pathological IEEE behavior, private visibility, and absence of project-slice
reachability.

The same worker applies/restores one mutation at a time while the parent runs
focused debug and records status. Include:

- fixed Normal scale or raw scaled-coordinate PCA;
- f64 triangle/moment accumulation;
- sign, area, covariance threshold, equal-variance or eigen ordering changes;
- reciprocal-multiply/stable norm, missing `z > 0`, or normalize after widening
  pc2;
- reversed/alternate normal;
- floor/round or wrong quantization factor;
- last duplicate wins;
- host/numeric/insertion order;
- FNV offset/prime/byte order or missing `-0.0` canonicalization;
- wrong bucket count/growth, occupied-bucket insertion, rehash group order;
- reverse input/cost loop, `<=` minimum, wrong rotation/initial result;
- signature/visibility mutation.

Classify runtime kills, compiler rejections, and equivalent survivors
separately. Numeric/order substitutions not exposed by bounded vectors remain
structurally audited survivors; do not add instrumentation to force kills.
Restore exact production/test hashes, then rerun focused debug/release and
rustfmt.

## Task 7 — Initial independent implementation review

Run in parallel:

1. a fresh read-only six-dimensional reviewer covering requirements, logic,
   edge cases, code quality, tests, and actual results;
2. default-model OpenCode over the same exact diff/evidence.

Require literal `VERDICT: APPROVE`. The parent creates a repair list. The same
sole worker performs accepted Rust/test repairs; the parent performs approved
documentation/evidence repairs. Every repair requires affected tests,
restoration/mutation refresh where relevant, complete exact-candidate gates,
and both rereviews.

## Task 8 — Update truthful documentation and provenance

Correct O37 spec/plan, roadmap, and option-parity records to released commits
`a0caa5a`/`4d83d15`, run `31291016394`, exact SHA
`4d83d15832c7905d7ea9727d14c07c5a75eb7312`, five successful jobs, and two
successful browser executions.

Record O38 as locally implemented, crate-private, inactive, and unreleased
pending final reviews, separate commits, push, and exact-SHA Tier-1. Record the
local proprietary-MSVC limitation without weakening the exact audited
open-source tag/model evidence. Broaden the existing MSVC notice title/body only
to name `geometry/bridge_direction.rs`, tag `vs-2022-17.14`, and
`xhash`/`type_traits`; retain the existing Apache-2.0 WITH LLVM-exception text
and statement that Ares neither links nor invokes MSVC STL/runtime.

State public slicing still stops after O26 and KSR parity is incomplete. Name
O39 exactly as `LayerRegion.cpp:262-308` composition, with merge/orchestration
and G-code deferred.

## Task 9 — Verify exact documented candidate

On exact bytes intended for final review, archive:

- O38 debug/release and complete geometry suite;
- O37/O36/O35, O28/O30, RegionExpansion/external-surface regressions;
- PolyTree, boolean-paths, offset, and O26 lifecycle;
- `cargo nextest run --workspace`;
- workspace all-target check;
- all-feature/all-target Clippy with `-D warnings`;
- `cargo fmt --all --check` and `git diff --check`;
- four wasm32 checks;
- two optimized WASM builds, bindgen/export audit, npm and JS syntax;
- full Playwright suite twice.

If Chromium lacks `libglib-2.0.so.0`, preserve both failures as environment
failures and keep both exact-SHA CI browser runs mandatory.

Static-audit exact 7-Rust/7-documentation-provenance paths, ordinary modules,
LOC, private visibility, numeric/order structure, notice attribution, no
forbidden/dependency/lifecycle/adapter/golden/G-code drift, empty staging, and
no generated artifact. Rehearse exact-O37 rollback in a disposable worktree,
run O37/geometry/RegionExpansion/PolyTree/boolean-paths/offset/O26, remove it
cleanly, and prove primary hashes unchanged.

## Task 10 — Final reviews, commits, push, exact-SHA Tier-1

Run both final implementation/documentation reviewers against the exact
candidate. Any repair invalidates stale evidence: rerun Task 9 completely,
refresh static/rollback hashes, and repeat both reviews.

After literal approvals and no tracked-byte change:

1. stage only seven Rust files and commit a Conventional Commit;
2. stage only seven approved documentation/provenance files and commit a
   separate Conventional Commit;
3. prove `.pi-subagents/`, `target/`, `/tmp`, oracle/model output and generated
   G-code are unstaged;
4. push `main` and require `HEAD == origin/main`;
5. wait for the push-triggered Tier-1 run with `headSha` equal to the exact
   documentation SHA;
6. require exactly five successful jobs and exactly two successful steps named
   `Run npm --prefix crates/ares-wasm/tests/browser test`;
7. archive run JSON only under `/tmp`.

Do not edit tracked O38 release state after the successful exact-SHA run. O39
records it. Any tracked byte change requires fresh exact verification/reviews,
commit/push, and a new exact-SHA run.
