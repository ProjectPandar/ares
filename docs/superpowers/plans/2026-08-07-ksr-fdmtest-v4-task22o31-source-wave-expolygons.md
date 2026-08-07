# Task 22O.31 Implementation Plan

## Execution state

O31 is released as implementation commit `7113f7c` and documentation commit
`1f89dd3`. Exact-SHA Tier-1 run `31196271880` passed all five jobs at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`. Focused debug/release pass 5/5,
RegionExpansion 69/69, workspace Nextest 6,010 with 2 skipped, and all required
native/WASM/static/rollback gates passed. The CI WASM job installed Chromium
runtime dependencies and passed the browser suite twice; the local host could
not launch Chromium because those libraries were unavailable. Both final
reviewers returned literal `VERDICT: APPROVE`.

## Goal and baseline

Implement the approved source/scalar `RegionExpansionEx` composition from
pinned `Algorithm/RegionExpansion.cpp:506-520` and
`RegionExpansion.hpp:94-100` as one crate-private Rust wrapper. Exact released
predecessor is `6ccb145dbb1867e5724538fb071795a7fd4179f0`; Tier-1 run
`31184069746` is green at that SHA; Orca remains pinned to
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

One writer owns implementation/test/docs and repair; reviewers are read-only.
Allowed Rust files are `region_expansion/propagate.rs`,
`region_expansion.rs`, `geometry.rs`, the RegionExpansion test root, and new
`tests/region_expansion/expolygon_composition.rs`. Allowed docs are O31
spec/plan, roadmap, option parity, and O30 spec/plan release corrections. Stop
for spec amendment if another file is required.

## Tasks

### 1. Baseline and released-predecessor audit

Verify HEAD/origin/O30 run/Orca SHA, empty staging, only O31 docs plus
`.pi-subagents/` untracked, LOC, and no unexpected changes. Run RegionExpansion
64/64, O30 focused 6/6, PolyTree 6/6, and O26 lifecycle 3/3. Archive under
`/tmp/task22o31-baseline-*`.

### 2. Oracle vectors and compiling RED

Use existing pinned-source O29/O30 oracle evidence under `/tmp` and, if needed,
a disposable scalar-composition oracle. Freeze compact complete
`(src,boundary,contour,holes)` literals for singleton, natural hole,
multi-ID/order, Normal, and LargeBed results. Oracle code/output remains
uncommitted.

Add the exact six-argument function signature, narrow reasoned
`#[expect(clippy::too_many_arguments)]`, private reexports/type assertions, and
ordinary test module with an `Ok(Vec::new())` stub. Tests must cover the
approved empty/precondition, sorted explicit pipeline, topology/order,
dual-scale, discovery error, and valid-discovery-before-propagation-error
matrix. Capture assertion RED, not compile failure. Keep shard <=300 LOC.

### 3. Implement literal source composition

Replace only the stub with:

```rust
let params = RegionExpansionParameters::build(
    full_expansion,
    expansion_step,
    max_nr_expansion_steps,
    scale,
);
let seeds = wave_seeds(src, boundary, params.tiny_expansion, true, scale)?;
propagate_waves_ex(&seeds, boundary, &params)
```

Build/discover/delegate exactly once and in that order. Same scale reaches both
calls; values are not rescaled. Preserve literal sorted `true`, complete seeds,
original boundary, direct `ClipperError`, and all assertion/error order. Add no
shortcut, overload emulation, mapping, validation, fallback, sort, or duplicate
pipeline.

Run focused debug/release and complete RegionExpansion/O30/PolyTree filters.
Any vector mismatch is investigated at first differing ID/point/hole/order; do
not normalize.

### 4. Mutations and structural audit

In a disposable worktree, kill/restore applicable mutations one at a time:
builder removed/duplicated/moved; full/step swap; max-step substitution; value
rescale; literal true false; wrong tiny expansion; mismatched scale on build or
discovery; discovery bypass; O30 bypass; dropped/reordered seeds/output; error
swallow/map; early empty; signature shape. Record runtime kills, compiler
rejections, equivalent survivors, and final restored GREEN under
`/tmp/task22o31-mutation-manifest.txt`.

Structurally prove one build, one sorted discovery, one O30 delegation, same
scale, no direct O27/union call, no clone/rescale/shortcut. Do not call
behaviorally equivalent inlining a mutation kill.

### 5. Documentation

Update O31 spec/plan, roadmap, option parity, and O30 release state. Record O30
commits `0a19939`/`6ccb145`, exact run `31184069746`, and exact SHA. State O31
adds no public/lifecycle/Option/checkpoint/G-code/ARD behavior and public slicing
still returns `ProjectSlicingIncomplete`. Next boundary is
`expand_expolygons` declaration hpp:102-108 / implementation cpp:522-534.

### 6. Exact candidate verification

Run focused debug/release, RegionExpansion, O30 focused, PolyTree, O26
lifecycle, workspace Nextest, workspace all-target check, warning-denying
workspace Clippy, and rustfmt. Run four wasm32 checks, separate optimized
default/feature builds, wasm-bindgen export and JavaScript syntax audits, and
two complete Playwright executions.

Static audit exact allowlist, LOC (<400, shard <=300), empty staging, no
manifest/lock/dependency/ARD/public/lifecycle/adapter change, no forbidden
patterns/lint allow/source pinning/reference G-code/oracle payload, and literal
function structure. Rehearse byte-identical candidate application and rollback
in a disposable O30 worktree; after removal require RegionExpansion 64/64, O30
6/6, PolyTree 6/6, lifecycle 3/3 and unchanged primary digest.

### 7. Dual final review and ship

Independent six-dimensional and default-model OpenCode reviewers compare the
same final documented diff/evidence to the approved spec/plan and must both
return literal `VERDICT: APPROVE`. Repair only within the allowlist. After any
review-requested repair, rerun affected checks and the complete exact-candidate
native/release/WASM/browser/static/rollback suite, refresh documentation and
evidence truthfully, then rerun both reviews against that same repaired and
verified diff.

Stage only approved files; create Conventional Commits and push. Confirm
HEAD==origin/main. O31 is released only after Tier-1 run `headSha` equals exact
pushed documentation SHA and format, WASM/browser, Linux, Windows, and macOS
all pass. Keep `.pi-subagents/`, `/tmp`, target output, bindings, and oracle
artifacts unstaged.

## Completion and rollback

Completion requires exact source composition, complete tests/mutations and all
local/review/CI gates. Mechanical rollback removes only O31 wrapper,
reexports/assertions, shard/registration, O31 docs, and O30 release corrections,
retaining exact released O27-O30 and O26 lifecycle. Full KSR G-code parity
remains deferred.
