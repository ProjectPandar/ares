# Task 19B.3 Typed FDM Normalization and Effective Project Configuration Implementation Plan

> **For agentic workers:** REQUIRED WORKFLOW: use `sdd-workflow` and
> Superpowers Subagent-Driven Development. Every implementation slice is owned
> by a fresh bounded Agent and starts with a qualifying RED. Independent
> reviewers are read-only and must not be the implementer. Do not begin any
> production or test implementation until this exact plan receives literal
> `VERDICT: APPROVE` from an independent Agent and OpenCode.

**Goal:** Port the fixed OrcaSlicer 2.4.2 typed FDM normalization, cold
double-`Print::apply` effective-project configuration, and explicitly bounded
used-filament discovery needed to resolve the committed `ksr_fdmtest_v4` 3MF
before geometry slicing.

**Architecture:** Normalize a cloned typed `ProjectSettings` before variant
materialization, preserve the three fixed normalization-input usage phases,
materialize from the fresh normalized source on both apply passes, and produce
one crate-private `BoundedResolvedProjectConfig`. Each project object owns
sorted transform groups plus one shared candidate set generated from its first
group. The returned usage type remains explicitly incomplete. The public
project path calls the resolver and then still returns
`ProjectSlicingIncomplete`; this task does not slice geometry or emit G-code.

**Tech stack:** Rust 1.91.0, edition 2024, existing typed option/project owners,
Cargo Nextest, rustfmt, warning-denying Clippy, `wasm32-unknown-unknown`,
wasm-bindgen browser tests, PowerShell, independent Agent and OpenCode review
gates. No new crate or dependency is authorized.

---

## Approved specification and immutable base

- Reviewed specification:
  `docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b3-effective-project-config.md`
- Frozen specification SHA-256:
  `1CCF806964B96425DBF4BB6B5426FB3149CD9B4515B900D5824D7CEC083E0877`
- Independent fixed-source reviewer: `VERDICT: APPROVE`
- Independent architecture/implementability reviewer: `VERDICT: APPROVE`
- Fixed OrcaSlicer commit/tag:
  `8500fcdccaa10b5099ac20d252af3a7c560046f1` / `v2.4.2`
- Pinned source worktree:
  `C:\Users\Indexyz\AppData\Local\Temp\Ares-Orca-8500fcdc`
- Implementation base commit:
  `d5a50bd64b7ebe048c80919edc6028b57f83fefa`
- Project fixture SHA-256:
  `698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9`

Before dispatching Slice 1, run the immutable checks and record their complete
output in the ignored SDD evidence ledger:

```powershell
(Get-FileHash docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b3-effective-project-config.md -Algorithm SHA256).Hash
git rev-parse HEAD
git status --short --untracked-files=all
git -C C:\Users\Indexyz\AppData\Local\Temp\Ares-Orca-8500fcdc rev-parse HEAD
(Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf -Algorithm SHA256).Hash
```

Expected values are the exact spec hash, base commit, pinned Orca commit, and
project fixture hash listed above. A mismatch stops dispatch.

Any specification edit invalidates both specification approvals. Any plan edit
after plan approval invalidates all plan approvals. The mutable repository
`OrcaSlicer` directory is not the source boundary.

## Fixed upstream rewrite boundary

This plan implements only the approved source slices:

- `PrintConfig.hpp:628-631` and `PrintConfig.cpp:8520-8740` for typed
  `normalize_fdm_1` and changed-key-returning `normalize_fdm_2`.
- `PrintApply.cpp:1113-1194,1256-1283,1525-1768` and
  `src/slic3r/GUI/PartPlate.cpp:3503-3510` for the cold two-apply lifecycle,
  cardinality derivation, pre-region first late call, first-call region state,
  second early call, second late call, and final views.
- `PrintApply.cpp:104-168` for exact transform grouping and
  `PrintApply.cpp:342-395,548-553,595-660,886-945` for normalized layer ranges
  and f32 Z-slab occupancy.
- `PrintObject.cpp:3555-3709` for object clamps and region precedence.
- `PrintApply.cpp:1662-1747` for one region candidate set per source object,
  generated from the first sorted transform group and shared across its groups.
- `PrintRegion.cpp:71-110`, `Model.cpp:2512-2564`,
  `Print.cpp:451-546,588-591,3290-3301,3385-3388`, and
  `Print.hpp:362-365,429-431` for bounded feature, raw, support, brim, raft,
  and wipe-tower filament participation.

The Rust destination is limited to typed `ares-core::options`,
`ares-core::project`, the existing public project caller, and exhaustive WASM
error formatting. Preset/UI `set_num_*` behavior, dynamic `SliceOptions`,
geometry/toolpaths/G-code, and complete used-filament discovery are outside
this task.

## Locked architecture and file policy

- `ProjectSettings` remains the only full typed configuration. Do not add a
  second 653-field effective struct or any string-key/dynamic option map.
- New typed normalization lives under
  `options/project_fdm_normalization/`; the legacy
  `SliceOptions::normalize_fdm` path remains untouched for STL consumers and is
  never called by the project resolver.
- Effective project code lives under `project/effective_config/`; split by
  normalization-independent responsibility and keep every changed Rust file
  below 400 physical lines.
- Treat existing large roots as registration/integration-only:
  `options.rs` is 315 lines, `project/domain.rs` 348, `print_apply.rs` 326,
  `src/tests/mod.rs` 334, and `project/tests/model/invalid.rs` 386 at the base.
  Do not place new behavior in them or add tests to the near-limit invalid file.
- `ResolvedProjectObject` owns one `layer_candidates` vector and an ordered
  vector of transform-only print-object groups. Candidates never move under a
  group and are never unioned across groups.
- Transform ordering scans Ares' matrix column by column to match Eigen
  `data()`, compares finite values with ordinary `<`/`>`, and treats signed
  zero as equal. `f64::total_cmp` is forbidden at this seam.
- `BoundedProjectUsage` exposes only `TypedConfigSourcesOnly` and has no
  conversion into a future complete usage type. No G-code consumer accepts it.
- Only public/untrusted project-boundary failures use compact validation.
  Internal helpers trust validated typed state; do not add speculative guards.
- No production code or test branches on the fixture name, fixture hash,
  object name, archive path, or expected values. No new or modified Task 19B.3
  test reads the reference `.gcode` or derives effective-config expectations
  from it. The unchanged pre-existing CLI golden test may run only as a
  regression audit and is not Task 19B.3 acceptance evidence.
- Do not add committed tests that read or pin Orca source text. Fixed-source
  inspection is review evidence only. Preserve unrelated option inventory and
  source-citation documentation tests that do not read an Orca checkout.
- Do not add filesystem, process, terminal, native-only, JSON-erased, or
  target-specific behavior to `ares-core`.
- Before Slice 12 production wiring, a new Task 19B.3 seam used only by tests
  may carry the narrowest possible
  `#[cfg_attr(not(test), expect(dead_code, reason = "Task 19B.3 production caller lands in Slice 12"))]`.
  Prefer an individual function/method; the wholly new `effective_config`
  module root may own one propagated expectation for its child graph. Rust
  considers an item used at its first non-test reference even when that caller
  is itself inside an otherwise unreachable private graph. The slice adding
  that first reference must own the defining file and remove the now-
  unfulfilled item expectation in the same slice. Retain only expectations on
  still-unreferenced items; the outer resolver/module expectation remains until
  `project_slice.rs` calls it in Slice 12. Do not add `allow(dead_code)`, dummy
  reads, or `cfg(test)` production behavior to defer this cleanup.
- After Slice 12, the only permitted Task 19B.3 dead-code expectations use the
  distinct reason `Task 19B.3 bounded result consumer is deferred` and are
  limited to output-only `BoundedResolvedProjectConfig`,
  `BoundedProjectUsage`, `ProjectConfigViews`, and the preserved
  `ResolvedLayerCandidate::{min_z,max_z,source_range_index}`,
  `ResolvedModelPartCandidate::volume_index`, and
  `ResolvedPrintObjectConfig::transform` fields. Add only members that a
  post-wiring `clippy -D warnings` run proves remain output-only; an
  unnecessary `expect` must fail as an unfulfilled lint expectation. Dummy
  reads, public API expansion, or manual trait code to silence these fields are
  forbidden.
- Do not update tracked architecture/roadmap docs until the whole
  implementation is approved. Do not stage or commit individual slices.

## Dispatch, TDD, and review rules

- Execute Slices 1-12 in order. Production edits are serialized in the shared
  checkout. Read-only research and review may run in parallel.
- `effective_config.rs`, `project.rs`, `options.rs`, `lib.rs`,
  `project_slice.rs`, the WASM root, and `print_apply` registration roots are
  integration choke points. Only the active sequential slice owner edits one;
  never dispatch parallel writers to those files.
- A fresh implementer Agent owns each slice. Give it only the approved spec,
  this plan's slice, the frozen starting manifest, its file ownership, and its
  commands. It may not expand scope.
- Add the complete slice test first. A qualifying RED must reach the intended
  missing/wrong behavior. If a new approved symbol initially prevents
  compilation, add only its exact private signature with an incomplete no-op
  body, rerun, and record the assertion/panic RED. Syntax, filter,
  environment, and unrelated failures do not count; no placeholder remains at
  GREEN.
- After GREEN, the primary Agent inspects the diff and reruns the focused and
  cumulative commands. A different read-only Agent reviews the exact bytes and
  returns literal `VERDICT: APPROVE` or `VERDICT: REVISE`.
- A `REVISE` stops dependent slices. Use a fresh fix implementer, rerun the
  slice and cumulative verification, freeze a new manifest, and repeat review.
- Record RED/GREEN commands, exit codes, test names, reviewer verdicts, and
  hashes in ignored `.superpowers/sdd` evidence. Never rely on stale output.

## Plan approval gate

This document remains a draft until its exact bytes are frozen and reviewed.
Before Slice 1:

- [ ] Recompute and record this plan's SHA-256.
- [ ] Dispatch a fresh read-only independent Agent to review source fidelity,
  dependency order, TDD quality, file ownership, cleanup timing, and whether
  every spec requirement has exactly one implementation slice. Require literal
  `VERDICT: APPROVE`.
- [ ] Dispatch a separate fresh OpenCode session against the same hash. Require
  literal `VERDICT: APPROVE`.
- [ ] Any plan edit invalidates both approvals. A `REVISE` requires a corrected
  plan, a new hash, and both reviews from scratch.
- [ ] Do not write production or test code until both approvals apply to the
  same bytes.

---

## Slice 1: Typed `normalize_fdm_1`

**Upstream:** `PrintConfig.hpp:628-631`;
`PrintConfig.cpp:8520-8614,8617-8685`.

**Production files:**

- Create `crates/ares-core/src/options/project_fdm_normalization.rs`
- Create `crates/ares-core/src/options/project_fdm_normalization/stage1.rs`
- Modify `crates/ares-core/src/options.rs`

**Test files:**

- Create `crates/ares-core/src/options/tests/project_fdm_normalization.rs`
- Create
  `crates/ares-core/src/options/tests/project_fdm_normalization/stage1.rs`
- Modify `crates/ares-core/src/options/tests.rs`

- [ ] **Step 1: Add the complete Stage 1 RED**

Add compiling tests that clone a typed `ProjectSettings`, call the crate-private
stage, and assert:

- sparse/internal/top/bottom propagation, including the original-snapshot case
  where bottom propagation overwrites the earlier top result;
- ordinary and nullable retract-on-layer-change vectors retain cardinality and
  become concrete false under spiral mode;
- exact spiral writes for wall loops, alternate extra wall, top shell layers,
  and sparse density;
- resolution below, equal to, and above `0.001`, including a finite negative;
- all fields outside the fixed write set remain unchanged.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project_fdm_normalization::stage1/)'
```

Expected RED: the typed stage is absent or the deliberate no-op preserves the
sentinels that fixed normalization must change.

- [ ] **Step 2: Make Stage 1 GREEN**

Implement only the typed source-ordered writes in `stage1.rs`. Read every
top/bottom/internal source value before mutating the destination fields. Do not
route through serialization, registry lookup, `SliceOptions`, or variant
resizing. The root module exposes only crate-private functions and carries
narrow temporary non-test dead-code expectations through Slice 10; Slice 11
owns the root and removes them when the resolver first calls both functions.

- [ ] **Step 3: Verify and review Slice 1**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project_fdm_normalization::stage1/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Freeze Slice 1 paths and require `VERDICT: APPROVE` for snapshot order, exact
write set, typed ownership, absence of resize/dynamic fallback, and test
strength.

## Slice 2: Typed `normalize_fdm_2` and changed keys

**Upstream:** `PrintConfig.cpp:8688-8740`.

**Production files:**

- Modify `crates/ares-core/src/options/project_fdm_normalization.rs`
- Create `crates/ares-core/src/options/project_fdm_normalization/stage2.rs`

**Test files:**

- Modify `crates/ares-core/src/options/tests/project_fdm_normalization.rs`
- Create
  `crates/ares-core/src/options/tests/project_fdm_normalization/stage2.rs`

- [ ] **Step 1: Add the complete Stage 2 RED**

Freeze the truth table for used counts zero/one/many; by-layer/by-object with
one/multiple PrintObjects; traditional/smooth timelapse; wrapping off/on;
already-disabled values; no reverse re-enable; and exact changed-key order.
Use a small compile-time enum whose serialized names are exactly
`enable_prime_tower` and `independent_support_layer_height`.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project_fdm_normalization::stage2/)'
```

Expected RED: the stage is absent/no-op and fails the tower/support truth table.

- [ ] **Step 2: Make Stage 2 GREEN**

Implement the fixed predicates and two-field write set. `used_filaments == 0`
is an exact no-op. Return only keys whose values changed, in fixed call order.
Do not accept or dispatch runtime strings. Keep both functions in the Slice 1
root module, but give `normalize_fdm_2` its own narrow item-level temporary
expectation alongside `normalize_fdm_1`. Slice 11 owns the root and removes
both expectations when orchestration first calls them. Do not add a module-
level or broad allowance.

- [ ] **Step 3: Verify and review Slice 2**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project_fdm_normalization/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for the full truth table, exact changed-key behavior,
zero no-op, and proof that no third field is writable.

## Slice 3: Unsupported-project error contract

**Upstream:** the approved deterministic boundary around deferred valid project
features; exhaustive Ares adapter ownership.

**Production files:**

- Modify `crates/ares-core/src/lib.rs`
- Modify `crates/ares-wasm/src/lib.rs`

**Test files:**

- Create `crates/ares-core/src/tests/slice_error.rs`
- Modify `crates/ares-core/src/tests/mod.rs`
- Modify the focused unit tests in `crates/ares-wasm/src/lib.rs`

- [ ] **Step 1: Add core/WASM error REDs**

Add tests for:

- compact errors name only the key and never echo supplied documents;
- exact Display/WASM text `unsupported project feature: {feature}` and an
  exhaustive WASM match with no wildcard;
- every pre-existing error variant retains its exact Display/WASM text.

Name the focused tests
`unsupported_project_feature_display_is_stable` and
`unsupported_project_feature_has_stable_javascript_mapping` so the filters
below cannot silently select zero tests. To obtain a behavioral RED for this
new public enum variant, add the approved variant and exhaustive match arms with
an intentionally incomplete placeholder string only after the tests exist,
then record the failing exact-string assertions. A non-exhaustive compile error
alone is not the qualifying RED, and the placeholder must not survive GREEN.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/unsupported_project_feature/)'
cargo +1.91.0 nextest run -p ares-wasm -E 'test(/unsupported_project_feature/)'
```

Expected RED: the deliberately incomplete Display/WASM placeholder strings fail
the named exact-string assertions.

- [ ] **Step 2: Make the error contract GREEN**

Add `SliceError::UnsupportedProjectFeature(String)` and its exhaustive WASM
arm. Do not add a wildcard or alter other error strings.

- [ ] **Step 3: Verify and review Slice 3**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/unsupported_project_feature/)'
cargo +1.91.0 nextest run -p ares-wasm -E 'test(/(unsupported_project_feature|slice_error)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
git diff --check
```

Require `VERDICT: APPROVE` for compact errors and exhaustive WASM behavior.

## Slice 4: Materialized cardinality and settings validation

**Upstream:** `PrintApply.cpp:1113-1194,1256-1283`;
`PrintObject.cpp:3555-3579`; fixed assertions used by `Print.cpp:488-546`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config.rs`
- Create `crates/ares-core/src/project/effective_config/cardinality.rs`
- Create
  `crates/ares-core/src/project/effective_config/selector_validation.rs`
- Modify `crates/ares-core/src/project.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config.rs`
- Create
  `crates/ares-core/src/project/tests/effective_config/cardinality.rs`
- Create
  `crates/ares-core/src/project/tests/effective_config/selector_validation.rs`
- Create `crates/ares-core/src/project/tests/effective_config/support.rs`
- Modify `crates/ares-core/src/project.rs` test registration
- Modify `crates/ares-core/src/project/tests/model.rs`
- Modify `crates/ares-core/src/project/tests/model/fixture.rs`

- [ ] **Step 1: Add materialized-settings REDs**

Add typed tests for:

- non-empty physical `nozzle_diameter` and logical `filament_diameter`;
- `filament_map.len() == logical_count` and every one-based entry within the
  physical count;
- the four directly indexed vectors `filament_ironing_flow`,
  `filament_ironing_spacing`, `filament_ironing_inset`, and
  `filament_ironing_speed` covering logical count;
- non-zero wipe selector satisfying both `selector < physical_count` and
  `selector <= logical_count`, including unequal-count valid/invalid cases;
- both shrink vectors covering logical count and every active value exactly
  100%; a non-100% entry returns `UnsupportedProjectFeature` with its concrete
  key;
- negative support selectors rejected by their concrete key while values above
  logical count remain valid inputs for the later fixed clamp-to-one;
- raw object, volume, and layer-range `extruder` values restricted to
  `0..=logical_count`, with object/volume/layer call-site sentinels;
- raw four-/eight-stride variant vectors remain unchanged before/after
  validation.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(cardinality|selector_validation)/)'
```

Expected RED: no materialized-project validator exists or an invalid boundary
is accepted.

- [ ] **Step 2: Make cardinality GREEN**

Implement a crate-private result containing only physical/logical counts and
the eager settings/domain checks above. Do not resize, fill, clamp, or
synthesize any vector. The later candidate resolver consumes this validated
state and lets `ObjectOptions::resolve` perform only the fixed greater-than-
logical support clamp. Register the wholly new `effective_config` module with
one narrow propagated non-test dead-code expectation; it remains until Slice
12 makes the final resolver production reachable.

Establish the sole shared effective-config test builder now. Reuse the existing
`ProjectParts` ZIP builder by widening only `project::tests::model::fixture`
and the required `ProjectParts::{valid,fixture,insert_text,replace,
make_single_model,set_model_settings_objects,bytes}` test-only visibility to
`crate::project::tests`; wrap/re-export it from `effective_config/support.rs`.
Slice 4 must make this support surface complete for the later approved tests;
later slices reuse it without editing `effective_config/support.rs`, copying
ZIP builders, or adding a production-only `LayerConfigRange` constructor.

- [ ] **Step 3: Verify and review Slice 4**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(cardinality|selector_validation)/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_variants|project_config_views|object_options|region_options)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for logical/physical separation, strict wipe bounds,
no resize, concrete unsupported keys, and compact failures.

## Slice 5: `Transform3d` fixed numeric primitives

**Upstream:** `PrintApply.cpp:104-168,548-553,595-660,886-945`.

**Production files:**

- Modify `crates/ares-core/src/project/transform.rs`

**Test files:**

- Modify `crates/ares-core/src/project/tests/transform.rs`

- [ ] **Step 1: Add transform REDs**

Freeze:

- value-only XY translation removal;
- column-major 16-scalar ordering, exact equality, and signed-zero equality;
- matrix composition `object_without_xy.then(volume)`;
- `transform_z_f32` casting every coefficient and point coordinate before
  multiply/add, using a sentinel that differs from f64-then-cast.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project::tests::transform/)'
```

Expected RED: missing value methods or the existing f64 transform produces the
forbidden precision result.

- [ ] **Step 2: Make transform primitives GREEN**

Add only crate-private `without_xy_translation`, fixed-order compare/equality,
and `transform_z_f32`. Do not add a second matrix representation. Do not use
`total_cmp`, rounded comparisons, or an epsilon. Give each new method only its
own temporary non-test dead-code expectation; no module-wide allowance.

- [ ] **Step 3: Verify and review Slice 5**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/project::tests::transform/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for fixed scalar order, signed zero, no precision
substitution, and no public API expansion.

## Slice 6: Production layer normalization and lookup

**Upstream:** `PrintApply.cpp:342-395`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config/layers.rs`
- Modify `crates/ares-core/src/project/effective_config.rs`
- Modify `crates/ares-core/src/print_apply.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/layers.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`
- Delete `crates/ares-core/src/print_apply/tests/layer_ranges.rs`
- Delete `crates/ares-core/src/print_apply/tests/layer_range_lookup.rs`
- Modify `crates/ares-core/src/print_apply/tests.rs`

- [ ] **Step 1: Add production range/lookup REDs**

Use the Slice 4 shared archive builder to insert typed layer-range XML, load a
real `Project`, and test its real `LayerConfigRange` owners without exposing
private fields or adding a production-only test constructor. Cover:

- empty, negative, reversed, overlap, gap, exact boundary, tiny gap, tiny
  configured range, and unconfigured infinite tail;
- immutable raw ranges and lookup's subtract-EPSILON/match-EPSILON behavior;
- preservation of the exact source range index on configured intervals.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::layers/)'
```

Expected RED: only dead staged integer-config helpers exist and no production
domain-owned range path satisfies the tests.

- [ ] **Step 2: Make new production tests GREEN**

Port the fixed algorithm with `EPSILON = 1e-4`, source range indices, and no
mutation. Give each production range/lookup entry point only its own temporary
expectation while it has no non-test caller; Slice 9 removes those expectations
when the candidate builder first calls both helpers.

- [ ] **Step 3: Replace obsolete layer staging**

After the new tests are green, delete only `LayerConfigRangeInput`,
`NormalizedLayerRange`, `normalize_layer_ranges`, and
`layer_range_config_id` from `print_apply.rs`, plus their two old tests and
registrations. Rerun the new tests to prove equivalent production coverage
before accepting the deletion.

- [ ] **Step 4: Verify and review Slice 6**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(effective_config::layers|project::tests::layer_config_ranges)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for exact range/lookup semantics, real domain
ownership, raw immutability, and deletion of only superseded staging.

## Slice 7: Sorted print-object transform groups

**Upstream:** `PrintApply.cpp:104-168`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config/grouping.rs`
- Modify `crates/ares-core/src/project/effective_config.rs`
- Modify `crates/ares-core/src/project/transform.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/grouping.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`

- [ ] **Step 1: Add grouping REDs**

Construct synthetic project objects and assert:

- XY-only instance translation differences collapse into one group;
- Z translation, rotation, or scale differences remain separate;
- reversed instance input order produces the same sorted groups;
- signed-zero transforms group together;
- non-printable instances produce no group;
- equal transforms on different project objects never share a group;
- effective print-object count is total groups, not object or instance count.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::grouping/)'
```

Expected RED: no project grouping helper exists.

- [ ] **Step 2: Make grouping GREEN**

Use the Slice 5 value/comparator methods and one sorted exact set per project
object. Preserve project-object order and group order; retain only transforms
needed by later candidate ownership. Do not add epsilon grouping, transform
hashing, or shrink compensation because non-100% shrink is already rejected.
Remove only the temporary expectations from `without_xy_translation`,
`fixed_order_less_than`, and `fixed_order_equal` when this production grouping
caller makes them used. Retain the still-unused `transform_z_f32` expectation.
Rust reports an `unfulfilled_lint_expectations` warning when an expected
`dead_code` lint becomes reachable even through this otherwise private call
graph, so deferring those three removals to Slice 12 cannot pass `-D warnings`.
The grouping entry point itself remains individually expected until the Slice
11 resolver first calls it.

- [ ] **Step 3: Verify and review Slice 7**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(effective_config::grouping|project::tests::transform)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for exact ordering, input-order independence,
signed-zero behavior, exclusion, object isolation, and correct count.

## Slice 8: Minimal f32 Z-slab occupancy

**Upstream:** `PrintApply.cpp:548-553,595-660,886-945`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config/occupancy.rs`
- Modify `crates/ares-core/src/project/effective_config.rs`
- Modify `crates/ares-core/src/project/transform.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/occupancy.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`

- [ ] **Step 1: Add occupancy REDs**

Use real meshes/volumes to freeze:

- the single-normalized-range special case, which admits every non-empty
  ModelPart without a slab test;
- multi-range expansion by `EPSILON` and strict edge rejection at both bounds;
- triangle-edge occupancy rather than vertex-only or bounding-box occupancy;
- combined transform order
  `print_object_without_xy.then(volume.transform())` and cleared combined XY;
- cast-before-multiply `transform_z_f32` behavior with the approved precision
  sentinel;
- no mesh, transformed vertices, XY box, polygon, or modifier parent retained
  in the result.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::occupancy/)'
```

Expected RED: no production occupancy helper exists.

- [ ] **Step 2: Make occupancy GREEN**

Implement only the fixed edge/slab boolean. Consume a representative
print-object transform supplied by the caller and the source volume transform;
do not choose a group or resolve a region in this helper. Remove the temporary
`transform_z_f32` expectation when this production occupancy caller makes the
method used; retaining it would emit `unfulfilled_lint_expectations` under the
warning-denying gate. The occupancy entry point itself remains individually
expected until Slice 9 candidates first call it.

- [ ] **Step 3: Verify and review Slice 8**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(layers|occupancy)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for the single/multi-range distinction, strict
predicates, composition/cast order, and absence of deferred geometry state.

## Slice 9: Shared object and model-part candidates

**Upstream:** `PrintObject.cpp:3555-3709`;
`PrintApply.cpp:958-1111,1662-1747`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config/types.rs`
- Create `crates/ares-core/src/project/effective_config/candidates.rs`
- Modify `crates/ares-core/src/project/effective_config/layers.rs`
- Modify `crates/ares-core/src/project/effective_config/occupancy.rs`
- Modify `crates/ares-core/src/project/effective_config.rs`
- Modify `crates/ares-core/src/project/domain.rs`
- Modify `crates/ares-core/src/options.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/candidates.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`

- [ ] **Step 1: Add candidate/topology REDs**

Freeze the crate-private topology and behavior:

- `ResolvedProjectObject` owns resolved `ObjectOptions`, sorted transform-only
  groups, and exactly one shared `layer_candidates` vector;
- one object with two Z-distinct groups supplied in reverse input order uses
  only the lexicographic first group for occupancy; layer ranges carry distinct
  feature-selector sentinels so evaluating/unioning the second group fails;
- different project objects own separate candidate sets;
- no printable group means no candidate set and no later source participation;
- object support selectors greater than logical count clamp to one while
  validated zero/positive selectors retain fixed meaning;
- process -> object -> volume -> no-material -> layer region precedence;
- interval source index and source volume index remain visible;
- only non-empty ModelPart volumes receive printable candidates;
- negative/support volumes receive none;
- a table test covers every detectable usage-affecting ParameterModifier key:
  `wall_loops`, `sparse_infill_density`, `top_shell_layers`,
  `bottom_shell_layers`, and all six feature filament selectors; each returns
  `UnsupportedProjectFeature` naming that first fixed key;
- zero-width `Painted` brim returns
  `UnsupportedProjectFeature("brim_type")`;
- project material remains `None` while existing pure material tests stay green.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::candidates/)'
```

Expected RED: no resolved topology/candidate builder exists.

- [ ] **Step 2: Make candidates GREEN**

Build candidates only from Slice 4 validated state, Slice 6 normalized ranges,
Slice 7 sorted groups, and Slice 8 occupancy. Read existing object/volume
override accessors, but retain their existing dead-code attributes through
Slice 11 because they are `allow` attributes rather than lint expectations.
Call both Slice 6 range/lookup entry points and the Slice 8 occupancy entry
point from the production candidate builder, then remove only those newly
fulfilled item expectations from `layers.rs` and `occupancy.rs`. Give the
candidate-builder entry point its own temporary expectation until the Slice 11
resolver first calls it. Do not invent modifier parents, material documents,
painted points, region deduplication, or per-group vectors. In `options.rs`,
re-export only `RegionBase` and `RegionOverrideSources` as crate-private seams
so candidates can call the existing `RegionOptions::resolve`; do not expose
them publicly or copy its merge logic.

- [ ] **Step 3: Verify and review Slice 9**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(cardinality|selector_validation|grouping|layers|occupancy|candidates)/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(object_options|region_options|project::tests::model)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for shared ownership, first-group-only occupancy,
typed precedence, domain validation, honest unsupported cases, and no deferred
geometry invention.

## Slice 10: Bounded supported used-filament composition

**Upstream:** `PrintRegion.cpp:71-110`; `Model.cpp:2512-2564`;
`Print.cpp:451-546,588-591,3290-3301,3385-3388`;
`Print.hpp:362-365,429-431`.

**Production files:**

- Create `crates/ares-core/src/project/effective_config/usage.rs`
- Modify `crates/ares-core/src/project/effective_config/types.rs`
- Modify `crates/ares-core/src/project/effective_config.rs`

If `usage.rs` would reach 400 physical lines, split only real responsibilities
into `usage/roles.rs` and `usage/support.rs`; do not add speculative interfaces.

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/usage.rs`
- Create `crates/ares-core/src/project/tests/effective_config/usage/roles.rs`
- Create `crates/ares-core/src/project/tests/effective_config/usage/raw_support.rs`
- Create `crates/ares-core/src/project/tests/effective_config/usage/wipe.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`

- [ ] **Step 1: Add used-filament REDs**

Tests must independently distinguish:

- all six feature-role predicates and their one-based to zero-based selectors;
- print-wide supported brim with per-object raft suppression: an object
  qualifies for `AutoBrim` regardless of width or another non-`NoBrim` type
  with positive width only when that same object has no raft; a mixed sentinel
  with raft on object A and valid brim on object B must still make print-wide
  brim true;
- object/volume raw fallback for ModelPart and ParameterModifier, exclusion of
  negative/support volumes, and default one;
- unconditional positive raw layer `extruder`, independent of occupancy;
- one integrated nonintersecting-layer sentinel whose feature selector is
  absent from candidates/usage while that same raw range's positive
  `extruder` still participates;
- the no-printable-group gate for region/raw/brim/support sources;
- support disabled, support/raft enabled, zero/current, positive selectors,
  clamp-to-one results, and current support appending all object extruders;
- exact dedup timing: object vector dedup, support vector dedup, concatenate
  without cross-vector dedup, wipe `len > 1` gate, then final sort/dedup;
- explicit wipe predicate using a separately supplied phase/tower config:
  preliminary tower enabled and either wrapping with more than two exclusion
  points, smooth timelapse, or non-spiral mode with more than one logical
  filament; plus its validated selector;
- `BoundedProjectUsage { coverage: TypedConfigSourcesOnly }` with no complete
  conversion.
- the Slice 9 reverse-input two-Z-group sentinel contributes only the
  representative group's feature selector to the used set; the other group's
  selector is absent before orchestration supplies the count to `_2`.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::usage/)'
```

Expected RED: no bounded collector/type exists.

- [ ] **Step 2: Make used-filament composition GREEN**

Implement the fixed source-order collector over the shared candidate set once
per qualifying project object. Keep the wipe-predicate config explicit so the
orchestrator can pass previous or fresh state without hidden mutable globals.
Do not add custom tool changes, painted facets, painted brim points, modifier
regions, project materials, or wipe tool ordering. Give only the collector
entry point a temporary expectation while it has no non-test caller; Slice 11
owns its file and removes that expectation when orchestration first calls it.

- [ ] **Step 3: Verify and review Slice 10**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(candidates|usage)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for every supported source, exact dedup/wipe timing,
bounded type safety, and explicit omission of all deferred sources.

## Slice 11: Exact cold double-apply orchestration and fixture

**Upstream:** `PrintApply.cpp:1113-1194,1256-1283,1525-1768`;
`src/slic3r/GUI/PartPlate.cpp:3503-3510`.

**Production files:**

- Modify `crates/ares-core/src/project/effective_config.rs`
- Create `crates/ares-core/src/project/effective_config/phases.rs` if required
  to keep the root comfortably below 400 lines
- Modify `crates/ares-core/src/project/effective_config/types.rs`
- Modify `crates/ares-core/src/project/effective_config/grouping.rs`
- Modify `crates/ares-core/src/project/effective_config/candidates.rs`
- Modify `crates/ares-core/src/project/effective_config/usage.rs`
- Modify `crates/ares-core/src/project/effective_config/usage/roles.rs` and
  `crates/ares-core/src/project/effective_config/usage/support.rs` only if
  Slice 10 created them
- Modify `crates/ares-core/src/options.rs`
- Modify `crates/ares-core/src/options/project_fdm_normalization.rs`
- Delete `crates/ares-core/src/print_apply/apply_normalization_state.rs`
- Modify `crates/ares-core/src/print_apply/staged_modules_legacy.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/phases.rs`
- Create `crates/ares-core/src/project/tests/effective_config/fixture.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`
- Delete `crates/ares-core/src/print_apply/tests/apply_normalization.rs`
- Modify `crates/ares-core/src/print_apply/tests.rs`

- [ ] **Step 1: Add phase-order REDs**

Use synthetic sentinels to prove the exact 12-step spec order:

- one normalized unmaterialized clone; cold first `_2(0, 0)` no-op;
- first materialization occurs after `_1` and reads normalized retract/spiral
  values;
- first late usage is pre-region raw/support/wipe and excludes feature regions;
- first regions use first late-normalized state and only first sorted groups;
- second early usage includes first regions but its wipe predicate reads the
  previous first-late config;
- second `_2` runs before a fresh second materialization from the original
  normalized unmaterialized source;
- final pre-normalize usage reads second candidates/second materialized tower;
- second late `_2` runs once, returned usage is recomputed from final state,
  the reachable vector remains equal, and no convergence call occurs;
- an explicit by-object/multiple-PrintObject case proves second-early `_2`
  disables the tower before second materialization, final pre-normalize usage
  excludes the wipe selector, post-normalization recomputation remains equal,
  and the call ledger contains no third `_2`;
- the reverse-input two-Z-group sentinel proves the nonrepresentative group's
  selector is absent from both the used vector and every `_2` used-count input;
- preliminary candidates are discarded and final candidates/views are rebuilt
  exactly once from final state;
- `ProjectConfigViews` derives only from final full state;
- original `Project.settings()` and raw variant stride vectors are unchanged.

Add the committed-project test, reading only the 3MF, asserting logical count
two, one print-object group, one implicit `[0, f64::MAX]` candidate, one
ModelPart, bounded used set `[0]`, final tower false, independent support true,
final `resolution == 0.012`, unchanged raw settings, the already approved
full/runtime retract distinctions on the resolver-returned views, and
`[0]` exposed only through `BoundedProjectUsage` with
`usage.coverage == TypedConfigSourcesOnly`. It must not open or include the
reference G-code.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::(phases|fixture)/)'
```

Expected RED: pure helpers exist, but no source-ordered resolver composes them.

- [ ] **Step 2: Make orchestration/fixture GREEN**

Add `BoundedResolvedProjectConfig` and the crate-private resolver. Re-export
only the remaining crate-private normalization/materialization/view seams
needed across the `options`/`project` boundary; preserve the two Slice 9 region
seams without widening them. When the resolver first calls normalization,
grouping, candidate, and usage entry points, remove their now-unfulfilled
item-level expectations from the files owned by this slice. Do not add
expectations to the resolver or phase helpers because the existing propagated
`effective_config` module expectation covers that still-unreachable graph.
Retain only that outer module expectation and the
pre-existing `project_variants`/`project_config_views` allowances through this
slice because the resolver itself is not called by `project_slice.rs` until
Slice 12. Keep transient phase vectors private and absent from the returned
type.

- [ ] **Step 3: Replace obsolete normalization staging**

After orchestration tests are green, delete only the staged normalization state
module/test and their registrations. The new phase-order tests must remain
green and provide strictly stronger production coverage.

- [ ] **Step 4: Verify and review Slice 11**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_fdm_normalization|effective_config)/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_variants|project_config_views|object_options|region_options)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Require `VERDICT: APPROVE` for phase fidelity, fresh materialization, stable
final result, fixture derivation, no hardcoding, and complete staged replacement.

## Slice 12: Production caller and public incomplete boundary

**Upstream:** approved Ares caller boundary around the fixed `Print::apply`
rewrite slice; G-code generation remains deferred.

**Production files:**

- Modify `crates/ares-core/src/project_slice.rs`
- Modify `crates/ares-core/src/project.rs`
- Modify `crates/ares-core/src/options.rs`
- Modify `crates/ares-core/src/options/project_config_views.rs`
- Modify `crates/ares-core/src/project/effective_config/types.rs`
- Modify `crates/ares-core/src/project/domain.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/effective_config/caller.rs`
- Modify `crates/ares-core/src/project/tests/effective_config.rs`
- Do not modify `crates/ares-cli/tests/ksr_fdmtest_v4.rs` or its golden helper;
  it is an unchanged pre-existing regression audit only

- [ ] **Step 1: Add caller REDs**

Add at least one synthetic project archive whose materialized typed settings
violate a named cardinality rule and assert public `slice_project` returns the
specific compact `SliceError::InvalidInput` key before the current
`ProjectSlicingIncomplete`. Add a separate unsupported-feature archive only if
useful; it does not replace the cardinality case. Also assert a valid committed
3MF still reaches exactly `ProjectSlicingIncomplete` after resolution and
malformed archives still fail before resolution.

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/effective_config::caller/)'
```

Expected RED: `project_slice.rs` loads and immediately returns incomplete, so
the required keyed cardinality `InvalidInput` is hidden.

- [ ] **Step 2: Make the caller GREEN**

Call the crate-private resolver immediately after `load_project`; discard its
bounded result only because geometry/G-code is deferred, then preserve the
public incomplete error. Do not expose the bounded type, add output bytes, or
special-case the committed fixture. After this non-test caller is connected,
remove the `effective_config` module expectation, the existing
`project_variants`/`project_config_views` module-level dead-code allowances,
remaining domain/helper expectations, and every expectation whose reason is
`Task 19B.3 production caller lands in Slice 12`. No transform, normalization,
layer, grouping, occupancy, candidate, or usage entry-point expectation may
survive from an earlier first reference. Run warning-denying Clippy;
for output-only DTO/identity warnings only, add the narrow distinct-reason
expectations from the locked whitelist. Do not keep a module-wide expectation
or annotate a member Clippy does not report.

- [ ] **Step 3: Verify and review Slice 12**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(effective_config|project_import|project_settings)/)'
# Unchanged pre-existing golden regression; not Task 19B.3 expectation evidence.
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run -p ares-wasm
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
git diff --check
```

Require `VERDICT: APPROVE` for production reachability, error ordering, the
unchanged public incomplete boundary, WASM portability, and zero G-code scope.

## Whole implementation approval gate

After all 12 slice reviews approve, freeze one manifest containing every
tracked/untracked implementation and test path plus their SHA-256 values. Run
fresh pre-review verification:

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_fdm_normalization|effective_config|project_variants|project_config_views|object_options|region_options)/)'
# Unchanged pre-existing golden regression; not Task 19B.3 expectation evidence.
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run -p ares-wasm
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test
git diff --check
```

- [ ] Dispatch a fresh read-only whole-spec compliance reviewer against the
  approved spec, approved plan, exact manifest, diff, and verification. Require
  literal `VERDICT: APPROVE` and `ROLE: SPEC COMPLIANCE`.
- [ ] Dispatch a different fresh read-only code-quality reviewer against the
  same bytes. It checks source fidelity, correctness, performance, Rust
  ownership, simplicity, tests, LOC, and cleanup. Require literal
  `VERDICT: APPROVE` and `ROLE: CODE QUALITY`.
- [ ] Run the same bounded whole-diff review through a fresh OpenCode session.
  Require literal `VERDICT: APPROVE`.
- [ ] Any `REVISE` unfreezes implementation. Fix with a fresh Agent, rerun
  focused/cumulative/full verification, freeze a new manifest, and rerun all
  three whole reviewers. Tracked docs remain untouched until all approve the
  same implementation bytes.

## Post-approval documentation gate

After whole implementation approval, update only:

- `docs/architecture/option-parity-v4.md` with typed normalization, logical and
  physical cardinality, sorted/shared object candidate ownership, bounded
  usage, double-apply collapse, public incomplete boundary, and explicit
  deferred sources;
- `docs/roadmap.md` with Task 19B.3 completion evidence and Task 19C next;
- ignored `.superpowers/sdd` progress/release evidence with RED/GREEN commands,
  reviewer verdicts, hashes, and the persistent goal still open.

Freeze the tracked docs and dispatch a fresh documentation reviewer. Require
literal `VERDICT: APPROVE` with `ROLE: DOCUMENTATION`. Any tracked doc edit
invalidates only the documentation approval.

## Fresh release matrix and mandatory audits

Run from the frozen implementation plus approved docs:

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_fdm_normalization|effective_config|project_variants|project_config_views|object_options|region_options|project::tests::layer_config_ranges)/)'
# Unchanged pre-existing golden regression; not Task 19B.3 expectation evidence.
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run -p ares-wasm
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test
git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
```

Mandatory audits:

```powershell
# Direct Task 19B.3 integrity evidence is hash-only; the unchanged CLI golden regression is separate.
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf -Algorithm SHA256
# 698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode -Algorithm SHA256
# 10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3

# No fixture/source hardcoding, native I/O, process invocation, or erased config.
rg -n 'ksr_fdmtest_v4|\.gcode|8500fcdc|Ares-Orca|include_(str|bytes)!|std::fs|File::|Command::|std::process' crates/ares-core/src/options/project_fdm_normalization.rs crates/ares-core/src/options/project_fdm_normalization crates/ares-core/src/project/effective_config.rs crates/ares-core/src/project/effective_config crates/ares-core/src/project_slice.rs
rg -n 'serde_json::Value|serde_json::Map|RawValue|BTreeMap<String|HashMap<String' crates/ares-core/src/options/project_fdm_normalization.rs crates/ares-core/src/options/project_fdm_normalization crates/ares-core/src/project/effective_config.rs crates/ares-core/src/project/effective_config
rg -n 'SliceOptions|set_num_extruders|set_num_filaments|get_parameter_size|extend_extruder_variant' crates/ares-core/src/options/project_fdm_normalization.rs crates/ares-core/src/options/project_fdm_normalization crates/ares-core/src/project/effective_config.rs crates/ares-core/src/project/effective_config

# New tests do not depend on a live Orca checkout or reference G-code.
rg -n '8500fcdc|Ares-Orca|OrcaSlicer[/\\]src|std::process|Command::|ksr_fdmtest_v4\.gcode' crates/ares-core/src/options/tests/project_fdm_normalization.rs crates/ares-core/src/options/tests/project_fdm_normalization crates/ares-core/src/project/tests/effective_config.rs crates/ares-core/src/project/tests/effective_config crates/ares-core/src/tests/slice_error.rs

# Superseded staging is gone and production reachability removed its allowances.
Test-Path crates/ares-core/src/print_apply/apply_normalization_state.rs
Test-Path crates/ares-core/src/print_apply/tests/apply_normalization.rs
Test-Path crates/ares-core/src/print_apply/tests/layer_ranges.rs
Test-Path crates/ares-core/src/print_apply/tests/layer_range_lookup.rs
rg -n 'LayerConfigRangeInput|NormalizedLayerRange|staged_apply_normalization_prelude' crates/ares-core/src
rg -n 'allow\(dead_code' crates/ares-core/src/options/project_fdm_normalization.rs crates/ares-core/src/options/project_fdm_normalization crates/ares-core/src/options/project_config_views.rs crates/ares-core/src/project/effective_config.rs crates/ares-core/src/project/effective_config crates/ares-core/src/project/transform.rs crates/ares-core/src/project/domain.rs crates/ares-core/src/project_slice.rs
rg -n 'Task 19B\.3 production caller lands in Slice 12' crates/ares-core/src
rg -n -C 2 'expect\(dead_code|Task 19B\.3 bounded result consumer is deferred' crates/ares-core/src/options/project_fdm_normalization.rs crates/ares-core/src/options/project_fdm_normalization crates/ares-core/src/options/project_config_views.rs crates/ares-core/src/project/effective_config.rs crates/ares-core/src/project/effective_config crates/ares-core/src/project/transform.rs crates/ares-core/src/project/domain.rs crates/ares-core/src/project_slice.rs
$optionsRoot = Get-Content crates/ares-core/src/options.rs -Raw
if ($optionsRoot -match '(?ms)#\[cfg_attr\(not\(test\),\s*(?:allow|expect)\(dead_code.*?\)\)\]\s*(?:pub\(crate\)\s+)?mod project_(?:config_views|variants|fdm_normalization)') { throw 'Task 19B.3 options dead-code attribute remains' }
$projectRoot = Get-Content crates/ares-core/src/project.rs -Raw
if ($projectRoot -match '(?ms)#\[cfg_attr\(not\(test\),\s*(?:allow|expect)\(dead_code.*?\)\)\]\s*(?:pub\(crate\)\s+)?mod effective_config') { throw 'effective_config dead-code attribute remains' }

# All changed Rust files remain below 400 physical lines.
$rustPaths = git diff --name-only --diff-filter=ACMR -- '*.rs'
$rustPaths += git ls-files --others --exclude-standard -- '*.rs'
$rustPaths | Sort-Object -Unique | Where-Object { Test-Path $_ } | ForEach-Object {
    $lines = (Get-Content $_).Count
    if ($lines -ge 400) { throw "$_ has $lines lines" }
}
```

All hardcoding/dynamic/source-pinning/`allow(dead_code)`/temporary-reason `rg`
commands must return no result. All four staged `Test-Path` commands must print
`False`. Both root-attribute scripts must not throw. The output-only expectation
scan may show only Clippy-proven members from the locked whitelist, every hit
must carry exactly `Task 19B.3 bounded result consumer is deferred`, and the
whole code-quality reviewer freezes the exact subset. No expectation remains
on modules, helper functions, transform methods, domain accessors, or wiring.

## Commit, push, and exact-SHA Tier 1

- [ ] Recompute a final manifest containing the approved spec, approved plan,
  approved implementation/tests, approved tracked docs, and reviewed baseline
  changes. Confirm `git status` contains only intended paths and the bytes match
  the manifest.
- [ ] Apply the Conventional Commits skill, stage only that manifest, and create
  one commit:

```powershell
git commit -m "feat(project): resolve effective project config"
```

- [ ] Push the current branch:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

- [ ] Verify clean local/remote equality:

```powershell
git rev-parse HEAD
git rev-parse origin/codex/ksr-fdmtest-v4-parity
git status --short
```

- [ ] Wait for the exact pushed SHA's Tier 1 workflow and require all five jobs
  green: `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
  `windows-latest`. Do not call Task 19B.3 released while that run is pending.

Task 19B.3 completion still leaves Task 19C configuration serialization,
remaining dynamic-consumer migration, geometry slicing, toolpaths, G-code,
metadata/post-processing, and complete normalized `ksr_fdmtest_v4` golden
parity open. Do not mark the persistent goal complete at this milestone.
