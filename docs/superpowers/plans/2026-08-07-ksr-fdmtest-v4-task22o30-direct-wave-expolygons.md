# Task 22O.30 Implementation Plan

## Goal

Implement the reviewed direct supplied-seed `RegionExpansionEx` slice from
pinned OrcaSlicer `Algorithm/RegionExpansion.cpp:480-503` and
`RegionExpansion.hpp:85-92`. Convert unchanged O27 polygon waves into ordered
`ExPolygon` records with the upstream post-propagation debug assertion,
adjacent ID grouping, singleton bypass, and NonZero multi-polygon union. Add no
source discovery, parameter construction, lifecycle, public API, Option, or
G-code behavior.

## Execution state

The reviewed implementation is locally complete. A fresh worker wrote the
approved Rust implementation and initial tests; because that worker runtime had
no command tool, the parent ran every RED/GREEN/oracle/mutation command and,
after the worker paused, strengthened the same allowed test shard with a
same-source boundary transition before final verification. There were no
concurrent writers and no out-of-allowlist code edit.

Chronological RED is five assertion/panic-expectation failures plus one valid
zero-output pass against the empty stub. Final focused debug and release runs
pass 6/6; RegionExpansion passes 64/64, PolyTree 6/6, O26 lifecycle 3/3, and
workspace 6,005/6,005 with 2 skipped. Sixteen runtime mutations are killed,
two oracle-demonstrated semantic survivors are disclosed, one signature-shape
mutation is compiler-rejected, and restored GREEN passes. Native check,
warning-denying Clippy, rustfmt, all four WASM checks, both optimized WASM
builds, export/syntax audits, and two 11/11 Playwright runs are green. Final LOC
are 74, 218, 62, 156, 6, and 263 across the approved Rust files. Exact
allowlist/LOC/visibility/forbidden-pattern static audit and disposable
exact-predecessor rollback are green. Final independent six-dimensional and
default-model OpenCode implementation reviews both return literal
`VERDICT: APPROVE`. O30 was released as implementation commit `0a19939` and
documentation commit `6ccb145`; exact-SHA Tier-1 run `31184069746` passed all
five jobs at `6ccb145dbb1867e5724538fb071795a7fd4179f0`.

## Reviewed specification, baseline, and ownership

- Reviewed spec:
  `docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o30-direct-wave-expolygons.md`
- Exact predecessor: `118f6a72b33926efe41ced1c931f9a51b26b2945`
- Predecessor exact-SHA Tier-1: run `31168584784`, successful on every required
  job at that SHA
- Pinned OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`
- Included source: `RegionExpansion.cpp:480-503`,
  `RegionExpansion.hpp:85-92`, and the `pftNonZero` default declaration at
  `ClipperUtils.hpp:548`

One fresh implementation worker is the sole writer for all Rust, tests,
documentation, and repairs. Reviewers are read-only. The parent checks every
diff and reruns authoritative commands. Stop for a spec amendment and renewed
spec/plan review if another production/test file or broader behavior is needed.

Allowed Rust files:

- `crates/ares-core/src/geometry/region_expansion/types.rs`
- `crates/ares-core/src/geometry/region_expansion/propagate.rs`
- `crates/ares-core/src/geometry/region_expansion.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/tests/region_expansion.rs`
- new
  `crates/ares-core/src/geometry/tests/region_expansion/expolygon_output.rs`

Allowed docs are the O30 spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O29 release-state corrections in
its spec/plan. Explicitly unchanged: manifests/lockfile, `lib.rs`, O27/O28/O29
implementation and test shards, Clipper implementation/tests, all
`project_slice*` production files, CLI, WASM source/browser JavaScript,
ARD-0024, Options, fixtures, checkpoints, persisted state, G-code, and
reference-G-code handling.

## Tasks

### 1. Freeze predecessor and baseline evidence

1. Verify and archive under `/tmp/task22o30-baseline-*`:
   - `HEAD == origin/main == 118f6a72b33926efe41ced1c931f9a51b26b2945`;
   - Orca checkout equals the pinned commit;
   - staging is empty;
   - the only existing worktree changes are the approved O29 release-state
     corrections, O30 spec/plan work, and ignored `.pi-subagents/` artifacts;
   - run `31168584784` is completed/successful and its `headSha` matches exactly.
2. Record baseline physical LOC. Require every allowed Rust file `<400` and the
   eventual new shard `<=300`.
3. Run baseline:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
```

Stop on SHA drift, staging, unexpected tracked files, predecessor CI mismatch,
LOC violation, or baseline failure.

### 2. Build disposable source oracle and select compact witnesses

1. Under `/tmp/task22o30-oracle-*`, adapt the already verified O27 standalone
   pinned-Clipper oracle to perform exactly the source postprocessing:
   propagation; sorted check as a harness precondition; adjacent expanded
   grouping; direct singleton contour; `ctUnion` with `pftNonZero` into
   `PolyTree`; recursive ordered `ExPolygon` conversion.
2. Compile against copied files from the pinned Orca checkout. Keep the C++
   source, binary, and output outside the repository.
3. Probe deterministic candidates until complete vectors cover:
   - singleton output retaining the exact O27 contour and changing under a
     singleton-through-union mutation;
   - equal-key multi output that unions to one result;
   - equal-key multi output that unions to multiple islands;
   - both source and boundary ID transitions;
   - a boundary-first/source-second comparator conflict, such as increasing
     boundary IDs with decreasing source IDs;
   - nonadjacent repeated IDs in the release unsorted witness, so global
     regrouping differs from source adjacent grouping;
   - if naturally reachable, one seed producing multiple expanded polygons or
     another witness that distinguishes expanded-group length from seed count;
   - if naturally reachable, propagated winding/topology that distinguishes
     NonZero from Positive and produces a direct hole.
4. Search for a natural direct hole and every mutation-sensitive condition
   above, not merely a preferred compact witness. If exhaustive bounded oracle
   probing finds no natural end-to-end direct hole, NonZero/Positive
   distinction, expanded-count distinction, or post-propagation union error,
   archive the candidates and reason each one is unreachable or semantically
   equivalent at this boundary. Only then may the corresponding direct test or
   runtime mutation be omitted. The existing complete `union_ex` hole/range
   regressions must still run, but must not be claimed as direct O30 execution.
5. Inspect every emitted source ID, boundary ID, contour point, hole point, and
   output order. Freeze only compact human-readable behavior-named Rust
   literals. Do not commit C++, generated output, a serialized oracle blob, or
   a digest in place of complete vectors.
6. Archive the exact compile/run commands, source provenance, search space, and
   mutation-witness disposition. Clearly call this a disposable pinned-source
   oracle, not an invocation of the final Ares implementation or full Orca
   application.

### 3. Establish compiling RED

The worker now owns all repository edits.

1. Add `RegionExpansionEx` with exactly the reviewed derive, fields, and
   crate-private visibility in `types.rs`.
2. Add the exact `propagate_waves_ex` signature in `propagate.rs` as a temporary
   `Ok(Vec::new())` stub.
3. Reexport record and function crate-privately in `region_expansion.rs` and
   `geometry.rs`. Add function-pointer assertions for type/arity/return shape;
   do not claim those assertions prove visibility or semantics.
4. Register ordinary `mod expolygon_output;` and add the new shard. Keep helper
   snapshot code local to the shard unless an already existing helper suffices.
5. Add complete-vector assertions for the reviewed evidence matrix:
   - empty and zero-propagated output;
   - singleton direct identity/no holes, using the mutation-sensitive witness;
   - one-result and multi-result NonZero unions;
   - complete adjacent ID transitions, equal-key acceptance, and the
     boundary-first/source-second comparator conflict;
   - successful unsorted debug panic and release adjacent-order behavior with
     nonadjacent repeated IDs that distinguish global regrouping;
   - unsorted propagation error before debug assertion;
   - sorted direct propagation error;
   - the complete direct-hole vector whenever oracle probing finds a natural
     propagated witness;
   - every other naturally reachable expanded-count or NonZero-sensitive
     witness found in Task 2.
6. A direct hole may be omitted only after the archived search establishes no
   natural end-to-end O30 witness. In that case, run the existing complete
   `union_ex` hole test separately and document it only as reused-kernel
   coverage. Apply the same explicit search/justification rule before omitting
   a seed-count/expanded-count or NonZero/Positive differentiator.
7. Use `catch_unwind` only for the trusted debug assertion witness. Conditional
   debug/release expectations must share the same successful unsorted geometry;
   release behavior may not be inferred from a debug-only run.
8. Capture real assertion RED from:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expolygon_output
```

At least the nonempty complete-output assertions must fail against the stub.
Unresolved imports, compiler errors, or a panic caused solely by a malformed
test do not count. Archive the chronological RED under `/tmp` before replacing
the stub.

### 4. Implement exact direct conversion

Replace only the stub body.

1. Call `propagate_waves(seeds, boundary, params)?` once and retain the complete
   returned vector.
2. After success, execute a debug-only lexicographic nondecreasing check of
   original `(boundary, src)` keys. Equal keys pass. Do not sort, reject in
   release, inspect paths, or move the assertion earlier.
3. Consume expanded records once with a forward iterator. Start each group from
   one record and collect only adjacent records with both matching
   `boundary_id` and `src_id`.
4. Retain group IDs before moving polygons.
5. For group length one, emit
   `ExPolygon::new(polygon, Vec::new())` directly.
6. For length greater than one, call only
   `union_ex(&polygons, FillRule::NonZero)?`, then move every returned
   `ExPolygon` into output in existing order with the retained IDs.
7. Return the complete vector. Add no shortcut before O27, validation, error
   mapping, fallback, partial output, safety offset, post-sort, topology
   normalization, public hook, or new abstraction.

Run focused GREEN in debug and release:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expolygon_output
cargo nextest run --release -p ares-core geometry::tests::region_expansion::expolygon_output
```

Then run the full RegionExpansion and PolyTree filters. If any complete vector
differs from the pinned oracle, inspect the first point/topology/order mismatch;
do not normalize expected or actual output to force a pass.

### 5. Mutation, ordering, and structural proof

Archive a manifest under `/tmp/task22o30-mutation-manifest.txt`. Apply one
mutation at a time, run the named killing test in debug and/or release as
needed, then restore and rerun GREEN. Attempt every behaviorally applicable
reviewed mutation:

1. skip O27 or swallow a propagation error;
2. move debug sorted assertion before O27;
3. remove the assertion;
4. make sortedness validation active in release;
5. reject equal keys;
6. compare source before boundary;
7. sort or globally regroup expanded results;
8. group by source only;
9. group by boundary only;
10. branch using seed count instead of expanded group length;
11. union singleton output;
12. wrap every multi polygon separately;
13. use `FillRule::Positive`;
14. retain only one union island, reverse output, drop holes, or overwrite IDs;
15. alter `RegionExpansionEx` or function signature shape.

A natural union-error mutation is required only if a genuine post-propagation
union failure was constructed. Do not add injection solely to create one.
Before accepting any survivor, deliberately search for the mutation-sensitive
witnesses defined in Task 2, including comparator-conflict ordering,
nonadjacent repeated IDs, expanded-group-length versus seed-count behavior,
singleton point-order identity, and NonZero versus Positive winding/topology.
A survivor is acceptable only when the archived search establishes genuine
end-to-end unreachability or semantic equivalence, not merely equivalence for
the initially convenient fixtures. Do not count compiler rejection as a
runtime kill. Preserve the original RED chronology separately from mutation
evidence.

Finish with source/diff structural audit confirming exactly one O27 call,
post-success debug assertion, one existing NonZero union call in the multi
branch, direct singleton construction, no clone of propagated polygons, no
second clipping engine, and no mutation residue.

### 6. Update bounded documentation

After focused GREEN and restored mutations, update:

- O30 spec with truthful implemented evidence, actual test counts, selected
  oracle witnesses/limitations, LOC, review state, and pending release gates;
- this plan with actual execution state;
- `docs/roadmap.md` and `docs/architecture/option-parity-v4.md` with the bounded
  direct `RegionExpansionEx` prerequisite;
- O29 spec/plan release status with exact commits, run `31168584784`, and exact
  SHA.

State explicitly that O30 changes no Option, public API/export, lifecycle,
checkpoint, persisted state, KSR golden expectation, or G-code byte. Public
slicing still uses O26 and returns `ProjectSlicingIncomplete`. Name the next
candidate as source/scalar `propagate_waves_ex` at lines 506-520; keep all later
merge/external-surface/fill/toolpath/motion/G-code work deferred. Do not add or
modify an ARD.

### 7. Verify native, release, WASM, browser, and static gates

Run and archive fresh exact-candidate evidence:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expolygon_output
cargo nextest run --release -p ares-core geometry::tests::region_expansion::expolygon_output
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Also run:

- `cargo check -p ares-core --target wasm32-unknown-unknown`;
- the same core check with `--features task22n-browser-oracle`;
- `cargo check -p ares-wasm --target wasm32-unknown-unknown`;
- the same adapter check with the feature;
- default and feature `cargo build --release -p ares-wasm
  --target wasm32-unknown-unknown` into separate `/tmp` target dirs;
- wasm-bindgen against both artifacts, export-name audit, and JavaScript syntax
  check;
- two complete Playwright executions through the existing Nix/`steam-run`
  environment when required by host libraries.

Static audits must prove:

- exact allowlist and empty staging before intentional staging;
- every Rust file `<400` LOC and new shard `<=300`;
- no manifest/lockfile/dependency/ARD change;
- no `unsafe`, FFI, filesystem/thread/platform branch, include macro, source
  concatenation, public export, fixture identity, reference G-code, binary
  oracle, source text/hash/line pinning, fallback, or broad lint allowance;
- `RegionExpansionEx` and `propagate_waves_ex` remain absent from `lib.rs`,
  public API, project slicing, CLI, WASM, and browser exports;
- O27/O28/O29 production and existing tests are unchanged;
- public lifecycle/checkpoints/state and golden KSR test are unchanged;
- no staged `.pi-subagents/`, `target`, `/tmp`, generated bindings, or oracle
  artifacts;
- `git diff --check` and rustfmt are clean.

### 8. Rehearse rollback

In a disposable clean worktree based on exact O29 predecessor:

1. apply the complete O30 candidate and verify byte identity with the primary
   tracked candidate;
2. remove only O30 record/function, private reexports/assertions, shard/module,
   O30 docs, and O29 release-state corrections;
3. verify retained files match exact O29 predecessor;
4. run RegionExpansion 58/58, PolyTree, and O26 lifecycle baselines;
5. remove the worktree and prove primary hashes, diff, and staging did not
   change.

Archive exact commands and final `ROLLBACK_REHEARSAL_OK` marker.

### 9. Independent six-dimensional review and repair loop

Dispatch fresh read-only independent and default-model OpenCode reviewers on the
same final documented candidate. Require literal `VERDICT: APPROVE` from both.
They must evaluate:

1. requirements completeness;
2. logic/source fidelity;
3. edge cases and error/assertion order;
4. code quality/visibility/LOC/forbidden patterns;
5. test/oracle/mutation coverage and truthful evidence accounting;
6. actual native/release/WASM/browser/static/rollback runtime results.

Provide spec, plan, exact predecessor/diff, evidence paths, and explicit
limitations. If either reviewer requests changes, the sole writer fixes only
the approved allowlist, reruns affected and final verification, updates docs,
and both reviewers re-review. Reviewers never modify repository files.

### 10. Commit, push, and exact-SHA ship gate

1. Recheck status, allowlist, empty staging, diff, LOC, forbidden patterns, and
   reviewer-no-modification evidence.
2. Stage only approved O30 files and release-state docs; never stage
   `.pi-subagents/`, `/tmp`, `target`, oracle artifacts, or generated bindings.
3. Create Conventional Commits, separating implementation and documentation if
   that keeps history reviewable.
4. Push `main`, confirm `HEAD == origin/main`, and capture the exact pushed
   documentation SHA.
5. Wait for Tier-1 and verify run `headSha` equals that exact SHA and every
   format, WASM/browser, Linux, Windows, and macOS job succeeds.
6. Do not claim O30 released before the exact-SHA run is green. Record the run
   in the next bounded milestone's release-state update.

## Completion conditions

- Direct supplied-seed `propagate_waves_ex` and `RegionExpansionEx` exactly
  match the reviewed bounded source behavior.
- Complete vectors prove singleton, one-result union, multi-island union, IDs,
  ordering, zero output, and debug/release/error precedence.
- Mutations and structural audits are truthful and fully restored.
- Native debug/release, full workspace, Clippy, fmt, WASM, browser, static,
  rollback, and dual-review gates pass.
- Exact pushed-SHA Tier-1 passes.
- O30 remains a crate-private prerequisite; full KSR G-code parity still awaits
  later source-cited external-surface, fill, toolpath, motion, serialization,
  and post-processing slices.
