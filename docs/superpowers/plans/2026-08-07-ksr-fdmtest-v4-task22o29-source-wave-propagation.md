# Task 22O.29 Implementation Plan

## Goal

Implement the approved source-taking RegionExpansion overload slice from
pinned OrcaSlicer `Algorithm/RegionExpansion.cpp:463-466,468-477` as two crate-private
Rust wrappers. Compose unchanged O28 sorted seed discovery with unchanged O27
propagation; add no lifecycle, public API, geometry engine, Option, or G-code
behavior.

## Source, baseline, and ownership

- Approved spec:
  `docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o29-source-wave-propagation.md`
- Exact predecessor: `be334375be871eb12ca98c98d889b65a92d13a37`
- Predecessor Tier-1: `31156094839`, successful at the exact predecessor SHA
- Pinned OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`
- Source: `RegionExpansion.cpp:463-466,468-477` and
  `RegionExpansion.hpp:74-83`
- One implementation worker is the sole writer for every Rust, test,
  documentation, and repair edit, including O28 ship-state corrections.
  Reviewers are read-only. The parent verifies every claimed diff and command.

Allowed Rust files:

- `crates/ares-core/src/geometry/region_expansion/propagate.rs`
- `crates/ares-core/src/geometry/region_expansion.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/tests/region_expansion.rs`
- new `crates/ares-core/src/geometry/tests/region_expansion/composition.rs`

Allowed docs are the O29 spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and the already identified O28
ship-state corrections in its spec/plan. Stop for a spec amendment and renewed
dual review if another production/test file is necessary.

Explicitly unchanged: manifests/lockfile, `lib.rs`, all `project_slice*`
production files, O27/O28 implementation/test shards, CLI, WASM source/browser
JavaScript, ARD-0024, Options, fixtures, checkpoints, persisted state, G-code,
and reference-G-code handling.

## Tasks

### 1. Freeze baseline and verification contract

1. Verify and archive under `/tmp/task22o29-baseline-*`:
   - `HEAD == origin/main == be334375be871eb12ca98c98d889b65a92d13a37`;
   - Orca checkout equals the pinned commit;
   - staging is empty;
   - uncommitted files are only approved documentation plus ignored
     `.pi-subagents/` artifacts;
   - exact predecessor run `31156094839` has matching `headSha` and success.
2. Record physical LOC for every allowed Rust file. Require every Rust file
   `<400` and the new shard `<=300`.
3. Run baseline:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
```

Stop on SHA drift, unexpected files, staging, LOC violation, or baseline
failure.

### 2. Establish compiling REDs and observe complete vectors

1. Add the approved signatures as temporary `Ok(Vec::new())` stubs in
   `propagate.rs`.
2. Reexport them crate-privately through `region_expansion.rs` and
   `geometry.rs`; add function-pointer arity/type-shape assertions without
   claiming visibility or adjacent-`f32` semantic proof.
3. Register ordinary `mod composition;` and add the new shard using existing
   `helpers::{expolygon, params, polygon, snapshots}`.
4. Before production bodies, use the existing explicit pipeline only
   (`build -> wave_seeds(..., true, scale) -> propagate_waves`) to observe and
   archive complete IDs/point vectors under `/tmp/task22o29-vector-*`:
   - compact source square `20..30` and boundary `0..100`, reusing the complete
     O28→O27 vector already checked in;
   - the existing two-source/two-boundary sorted-versus-unsorted witness;
   - a Normal/LargeBed scalar witness, initially source `200_000..300_000`,
     boundary `0..1_000_000`, inputs `(100_000.0, 10_000.0, 5)`.
5. Freeze human-readable complete Rust literals only after direct inspection.
   The dual-scale vectors must both be nonempty and observably different. If
   not, stop and select a reviewed asymmetric/near-boundary witness; do not
   weaken to count/area/bounds or add instrumentation.
6. Cover live complete-vector relationships separately from transcription:
   - assert parameter-wrapper output equals the explicit
     `wave_seeds(..., true, scale) -> propagate_waves` pipeline and the complete
     literal vector;
   - assert sorted wrapper output equals the explicit `sorted=true` pipeline,
     differs from the explicit `sorted=false` pipeline, and freezes both
     complete ordered results;
   - valid empty inputs;
   - invalid tiny expansion before empty shortcuts;
   - scalar full/step/max preconditions before empty shortcuts;
   - direct discovery error;
   - valid discovery followed by propagation error;
   - scalar construction before invalid geometry;
   - for Normal and LargeBed, assert scalar-wrapper output equals exactly one
     `RegionExpansionParameters::build` followed by
     `propagate_waves_from_sources`, equals its complete literal vector, and
     the two scale outputs differ.
7. Capture a compiling assertion RED:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
```

RED must be failing assertions from stub output, not unresolved imports or
compilation errors.

### 3. Implement the parameter source wrapper

Replace only its stub with literal source composition:

```rust
let seeds = wave_seeds(
    src,
    boundary,
    params.tiny_expansion,
    true,
    scale,
)?;
propagate_waves(&seeds, boundary, params)
```

Preserve literal `true`, complete discovery before propagation, original
references, direct `ClipperError`, and O28's assertion-before-empty behavior.
Add no shortcut, normalization, sort/regroup, retry, fallback, validation, or
partial output.

### 4. Implement scalar build-once delegation

Replace only its stub with:

```rust
let params = RegionExpansionParameters::build(
    expansion,
    expansion_step,
    max_nr_steps,
    scale,
);
propagate_waves_from_sources(src, boundary, &params, scale)
```

Build once before any shortcut/geometry and pass the same scale to builder and
parameter entry. Do not rescale, manually construct parameters, duplicate the
pipeline, or call `wave_seeds`/O27 directly from this entry.

Run:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion
```

### 5. Mutation and structural audit

For each temporary mutation: save production digests, mutate once, run the
named witness, record classification/failure excerpt, restore exact bytes,
verify digests, and rerun GREEN. Store only
`/tmp/task22o29-mutation-manifest.txt`.

Required runtime mutations:

1. sorted `true` -> `false`;
2. `tiny_expansion` -> another `f32` field;
3. skip discovery/pass empty seeds;
4. reverse or sort expansion output;
5. hardcode scalar-builder scale;
6. swap expansion and expansion-step forwarding;
7. alter max-step forwarding;
8. move build after an empty shortcut;
9. suppress/map discovery or propagation error.

Required compile-time mutation: change arity or a differently typed argument
position and record function-pointer/call-site compiler rejection. Do not call
it a compiling behavioral mutation.

Faithful scalar inlining is behaviorally equivalent and is not a mutation.
Instead inspect the final diff/source and require exactly one builder call then
one `propagate_waves_from_sources` call in the scalar body, with no direct
`wave_seeds` or O27 call. Do not add instrumentation or claim a killed mutation.

### 6. Document bounded completion before final-state audits

After focused GREEN tests and restored mutations, update O29 spec/plan,
roadmap, and `option-parity-v4.md` with exact source boundary, Rust
destination, vectors, RED/mutations, structural audit, verification status,
rollback contract, and residual risks. Retain the concrete O28 ship-state
corrections already made for run `31156094839`.

State explicitly that O29 changes no KSR checkpoint or G-code byte, public
slicing still consumes O26 and returns `ProjectSlicingIncomplete`, O27/O28 are
unchanged, ARD-0024 is unchanged, and all `_ex`/merge/external-surface/fill/
toolpath/motion/G-code/post-processing behavior remains deferred. Do not claim
pending CI as passed.

The same sole implementation writer owns these documentation edits. After the
docs are final, rerun every final-state static/diff/hash/rollback gate below so
the reviewed and committed state—not a pre-documentation state—is verified.

#### Task 6 execution evidence

The bounded implementation ports pinned
`RegionExpansion.cpp:463-466,468-477` and `RegionExpansion.hpp:74-83` to the
crate-private `propagate_waves_from_sources` and
`propagate_waves_from_sources_with_steps` destinations. The first wrapper uses
literal sorted O28 discovery followed by unchanged O27 propagation. The second
builds parameters once and delegates once while forwarding the same retained
explicit scale. It adds no public export, lifecycle/checkpoint, G-code byte,
option, persisted state, or ARD change; ARD-0024 and the O26 public incomplete
lifecycle remain unchanged.

Complete literals cover the compact handoff, sorted and unsorted multi-ID
results, a 16-point Normal result, and a 128-point LargeBed result. The final
composition shard passes 5/5 and the full RegionExpansion regression passes
58/58. Ten runtime mutations were killed, restored byte-for-byte, and followed
by GREEN; one differently typed signature mutation was rejected by the
compiler and restored. These mutation runs are post-hoc recurrence evidence,
not original RED. The structural delegation audit makes no false mutation
claim: the scalar body contains one builder call followed by one
`propagate_waves_from_sources` call and contains no direct `wave_seeds` or O27
call.

The chronological compiling RED log
`/tmp/task22o29-red-focused-all.txt` belongs to an earlier eight-test version:
seven empty-stub assertions failed, while `scalar_scale_outputs_differ` passed
while both wrapper stubs returned empty because that comparison used the
explicit pipelines. The final tests were subsequently consolidated and
strengthened into five tests, including valid discovery before the bounded
propagation failure. No chronological RED exists for that exact final list and
none is reconstructed.

The frozen six-argument scalar signature exceeds the configured five-argument
Clippy threshold and therefore has one function-scoped, reasoned
`#[expect(clippy::too_many_arguments)]`; no lint `allow` was added. Final LOC are
`propagate.rs` 172, `region_expansion.rs` 55, `geometry.rs` 150, the
RegionExpansion test root 5, and `composition.rs` 264. Mechanical rollback
removes only the two wrappers, their crate-private reexports and signature
assertions, the composition shard/module registration, and O29 documentation,
while retaining O27, O28, and O26 lifecycle bytes.

The restored final local state passes composition 5/5, RegionExpansion 58/58,
O26 lifecycle 3/3, workspace 5,999/5,999 with 2 skipped, native all-target
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized WASM
builds, wasm-bindgen export/JavaScript syntax audits, two 11/11 Playwright runs,
static audits, and disposable rollback. Final documented-state independent
six-dimensional and default-model OpenCode rereviews return literal
`VERDICT: APPROVE`. Implementation commit `55c2c23` and documentation commit
`118f6a7` were pushed; exact-SHA Tier-1 run `31168584784` passed all format,
WASM/browser, Linux, Windows, and macOS jobs at
`118f6a72b33926efe41ced1c931f9a51b26b2945`.

### 7. Verify native, WASM, browser, static, and lifecycle gates

Required commands:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-core --target wasm32-unknown-unknown --features task22n-browser-oracle
cargo check -p ares-wasm --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22n-browser-oracle
```

Also repeat `.github/workflows/tier1.yml` optimized default/feature WASM builds,
wasm-bindgen export audit, JavaScript syntax checks, and Playwright twice.
Archive output under `/tmp/task22o29-*`.

Static audits must prove:

- exact changed-file allowlist and empty staging before deliberate commit;
- no O29 symbol under project slicing, `lib.rs`, CLI, or WASM;
- no manifest/lockfile/dependency/ARD/public/lifecycle/fixture change;
- all Rust files `<400`, new shard `<=300`;
- no `include!`, `include_bytes!`, source concatenation, `unsafe`, FFI,
  filesystem/native-thread, platform branch, production hardcoding,
  source-pinning, fixture-identity, reference-G-code, `_ex` wrapper, or
  fallback addition;
- `ProjectSlicingIncomplete` and O26 lifecycle remain unchanged;
- `git diff --check` passes.

### 8. Rehearse final documented mechanical rollback outside the primary worktree

Record primary diff, approved untracked files, staging, file list, and hashes.
Reproduce the exact O29 state in a disposable worktree based on the predecessor.
Remove only the two wrappers/reexports/assertions, composition shard/module,
O29 docs, and identified O28 ship-state corrections; restore retained files to
predecessor bytes. Require clean predecessor `git status` and `git diff --check`.
Delete the disposable worktree, prove the primary state/hashes were unchanged,
and rerun focused composition and RegionExpansion tests.

### 9. Final independent review/fix loop

Submit the same final documented/restored diff and evidence to:

1. a fresh independent six-dimensional reviewer covering requirement
   completeness, logic, boundary cases, code quality, tests, and actual runs;
2. default-model OpenCode.

Both must return literal `VERDICT: APPROVE`. The parent turns any findings into
one repair list; the sole writer applies accepted repairs, reruns affected and
full gates, and both reviewers re-review. Repeat until dual approval. Any
unresolved issue or missing literal verdict blocks commit.

### 10. Commit, push, and exact-SHA Tier-1

1. Explicitly stage only reviewed implementation/tests and commit using
   Conventional Commits, for example
   `feat(core): compose source wave propagation`.
2. Explicitly stage only reviewed docs and commit, for example
   `docs(parity): record source wave composition`.
3. Never stage `.pi-subagents/`, `/tmp`, `target`, generated bindings,
   `node_modules`, oracle/vector output, or mutation evidence.
4. Push `main`; require `HEAD == origin/main`.
5. Find and watch only the Tier-1 run whose `headSha` equals the exact final
   pushed SHA. Require format, Linux, macOS, Windows, WASM, exports, and both
   browser runs green. Do not create a post-CI docs commit that invalidates the
   exact-SHA gate.

## Dependencies and stop conditions

- Spec and plan dual approval block production changes.
- Compiling RED and inspected literal vectors block production bodies.
- Parameter wrapper precedes scalar wrapper.
- All mutations must be restored before documentation edits.
- Final documentation blocks full/static/browser verification and rollback.
- Final-state verification and rollback block independent review.
- Dual final approval blocks commit/push.
- Exact pushed-SHA Tier-1 blocks release.
- Stop for renewed spec review if another file, public/lifecycle behavior,
  `propagate_waves_ex`, merge helper, external-surface orchestration, Option,
  dependency, or fallback becomes necessary.

## Residual risks

- The observed and frozen dual-scale scalar vectors are Rust pipeline evidence,
  not a new independent Orca oracle.
- Function-pointer assertions cannot distinguish adjacent `f32` semantics;
  ordered vectors and a swap mutation own that evidence.
- Boundary/source range failures share `CoordinateOutOfRange`; O28 remains the
  authoritative internal-order witness.
- Parameter-entry scale can be output-insensitive for valid AABB recovery;
  explicit signatures, unchanged O28 dual-scale tests, and scalar dual-scale
  output are the accepted combined evidence.
- O29 remains a pure prerequisite; full KSR parity requires later source-cited
  external-surface, fill, toolpath, motion, serialization, and processor slices.
