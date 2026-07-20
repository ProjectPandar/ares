# Task 22L Implementation Plan: Post-Region Conical Overhang

## Status, fixed points, and success condition

This plan is a draft. No production or tracked test implementation is
authorized until the fixed-source oracle is complete and the exact
specification and plan bytes receive independent fixed-source/specification and
current-Ares/plan approval.

The fixed Ares baseline is commit
`7f71ed8068102772d54346ac08184ef6b0bcd79b`, tree
`4e3a7445d340bd1dc22bdb184fbca6f2bad17521`; exact-SHA Tier-1 run
`29704298779` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with exact blobs and ranges in the
Task 22L specification.

Success means Ares ports the complete uncancelled
`PrintObject::apply_conical_overhang()` geometry stage after Task 22K, consumes
only typed Options resolved from each 3MF, preserves exact upstream f32/scale
boundaries, supports arbitrary ExPolygons, holes and multiple regions, proves
enabled and disabled real-project behavior, preserves plan and complete
sidecars, remains publicly incomplete, and passes native, WASM, Chromium,
six-axis review, and exact-SHA Tier-1 release gates.

Task 22L does not emit G-code or claim normalized KSR parity.

## Immutable implementation ledger

1. Task 22L runs once after released Task 22K and before all later segmentation
   or compensation stages.
2. It consumes `PostRegionPrintObject`, ordered resolved object contexts, and
   the already selected `CoordinateScale`.
3. It reads only existing object/region Options resolved from the supplied
   3MF.
4. After Task 22K, validate all resolved object configs transactionally in
   vector order, angle before hole, before any L mutation; exact raw errors are
   `invalid Orca option make_overhang_printable_angle` and
   `invalid Orca option make_overhang_printable_hole_size` even for empty,
   zero-region, disabled, or angle-90 objects.
5. The pure stage order is empty return, exact-90 return, exact f64/f32
   derivation and finite check, then pair gates.
6. Empty and single-layer objects are accepted; empty does not derive, while a
   single layer derives before discovering that it has no layer pair.
7. Pair iteration is top-down and changes cascade into the next lower pair.
8. Upper-layer emptiness uses only surface-vector cardinality across regions.
9. The lower-layer gate is cross-region and checks current surfaces plus the
   current region switch.
10. Nominal object `layer_height`, never actual planned-layer height, sets the
    conical distance.
11. Scaled distance and hole area retain the upstream f32 conversion without
    prior integer quantization.
12. `SCALED_EPSILON` is 100 at normal scale and 10 at large scale.
13. `Layer::merged` filters regions by the four printable-content fields,
    offsets each participating region first, then performs NonZero union.
14. Hole size zero skips protection; positive thresholds use strict `<` and
    require full coverage by intersection plus XOR.
15. Upper footprint shrink uses Miter join and miter limit 3.
16. Region candidates come from pair-start upper-region surfaces intersected
   with the shrunken complete upper footprint; these may already contain the
   previous higher pair's cascade result.
17. Candidate islands fully covered by the fixed original current footprint
    are removed; partially covered islands remain whole.
18. Same-region output concatenates current then candidate ExPolygons as
   Subject paths, unions them, and rebuilds exact default Internal surfaces.
19. Other regions subtract candidates with raw per-path 10-coordinate safety
    expansion; all rebuilt surfaces use exact metadata
    `(Internal, -1.0, 1, -1.0, 0)`.
20. Region processing follows existing order and current footprint is not
    recomputed between regions.
21. Planned layers, IDs, Options, object order, and complete volume sidecars do
    not change.
22. Nonfinite derived f32 values and Clipper errors map exactly once to
    `project conical overhang geometry is nonfinite or outside the supported Clipper range`;
    there is no identity fallback.
23. The old rectangle compatibility helper is not called or extended.
24. KSR resolves object values `0.2 / 55 / 0` and region values
    `false / 3 / 5 / 15% / 2`; its disabled L body therefore equals released
    K while exercising real merged-footprint eligibility.
25. A real stepped 3MF false/true pair differs only in the switch and proves
    enabled production behavior.
26. Public slicing executes L and still returns
    `ProjectSlicingIncomplete`.
27. The K browser feature is replaced by L; no alias or legacy export remains.
28. Default WASM has no Task 22 export; feature WASM has exactly L input/output.
29. Cancellation and all later slicing/G-code stages remain deferred.
30. Task 22L tracked tests never inspect Orca source identity and production
    never reads fixtures, hashes, names, reference G-code, environment, or
    filesystem data.

## Exact planned tracked manifest

No tracked path outside this 34-path list may change without a plan amendment
and fresh exact-byte document approval. Every listed path must change; missing,
extra, or substituted paths block implementation closure.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22l-conical-overhang.md`
- `docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22l-conical-overhang.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature and Clipper kernel

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/clipper.rs`
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs`
- `crates/ares-core/src/geometry/tests/clipper.rs`
- `crates/ares-core/src/geometry/tests/clipper/conical_overhang.rs`

### Project stage, orchestration, and native tests

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/conical_overhang.rs`
- `crates/ares-core/src/project_slice/conical_overhang/geometry.rs`
- `crates/ares-core/src/project_slice/region_slices.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/gates.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/geometry.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/holes.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/regions.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/oracle.rs`
- `crates/ares-core/src/project_slice/tests/conical_overhang/fixture.rs`
- `crates/ares-core/src/project_slice/tests/region_fixture/checkpoint.rs`
- `crates/ares-core/src/project_slice/tests/region_slices.rs`
- `crates/ares-core/src/project_slice/tests/region_slices/complex.rs`

### WASM, browser, and Tier-1

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/project-slice-page.mjs`
- `crates/ares-wasm/tests/browser/task22l-vectors.mjs`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `crates/ares-wasm/tests/browser/server.mjs`
- `.github/workflows/tier1.yml`

Ignored evidence ledgers, the fixed-source oracle, temporary targets, bindgen
output, Playwright output, and generated in-memory archives are never staged.
The two committed KSR fixtures and `Cargo.lock` remain unchanged.

## Module and line budgets

Every changed Rust production and test file remains below 400 physical LOC:

- `crates/ares-core/src/lib.rs`: at most 280;
- `geometry.rs`: at most 70;
- `geometry/clipper.rs`: at most 180;
- `geometry/clipper/boolean_ex.rs`: at most 150;
- `geometry/tests/clipper.rs`: at most 20;
- `geometry/tests/clipper/conical_overhang.rs`: at most 320;
- `project_slice.rs`: at most 375;
- `project_slice/conical_overhang.rs`: at most 260;
- `project_slice/conical_overhang/geometry.rs`: at most 260;
- `project_slice/region_slices.rs`: at most 350;
- `project_slice/tests.rs`: at most 40;
- `tests/conical_overhang.rs`: at most 280;
- each new `tests/conical_overhang/*.rs`: at most 390;
- `tests/region_fixture/checkpoint.rs`: at most 350;
- `tests/region_slices.rs`: at most 350;
- `tests/region_slices/complex.rs`: at most 365;
- `ares-wasm/src/lib.rs`: at most 160.

Browser budgets are `index.html` at most 30 physical lines,
`project-slice-page.mjs` at most 390, `task22l-vectors.mjs` at most 300, and
`project-slice.spec.mjs` at most 350. `server.mjs` is at most 70 lines. The
current inline page implementation is moved, not duplicated, into the real
imported page module, and the explicit server route table serves both new
modules. If any budget is insufficient, amend and reapprove the manifest before
editing. Rust source splitting with `include!`, `include_bytes!`, or related
macros is forbidden.

## Working protocol

Packages 1 through 5 change executable behavior or an executable adapter and
therefore use strict RED/GREEN TDD. For each implementation package:

1. freeze its allowed paths and exact acceptance vectors;
2. add package-owned tests in real modules;
3. run the smallest nextest/browser command and record attributable RED
   evidence in `.superpowers/sdd/task22l-evidence.md`;
4. implement only the fixed-source behavior required to make that RED green;
5. run focused predecessor and package regressions;
6. run rustfmt, strict Clippy, LOC, macro, unsafe, hardcoding, and diff checks;
7. freeze a path-sorted `<path><NUL><lowercase SHA-256>` content frame;
8. obtain independent specification and quality review before advancing.

Package 0 is an analysis/document gate and Package 6 is documentation/release.
No later package starts while an earlier package has a P0-P3 finding.

## Package 0: fixed-source oracle and exact document approval

Tracked paths:

- the Task 22L specification;
- this implementation plan.

Create an ignored source oracle under `.superpowers/sdd/task22l-oracle/` that:

- verifies the fixed Orca commit/tree objects and all 16 cited
  `PrintObjectSlice`, `Layer`, `Surface`, `PrintConfig`, `Polygon`,
  `ExPolygon`, `ClipperUtils`, and modified Clipper blobs before compilation,
  plus the two live Clipper blobs that the build directly consumes;
- compiles every oracle-owned translation unit independently with
  `/std:c++20 /O2 /fp:precise /W4 /WX /DNDEBUG`; fixed `clipper.cpp` is a
  separate translation unit using `/W0` only for its unchanged C4244 at line
  3762 and is linked rather than textually included;
- mechanically implements the complete non-cancellation source sequence;
- emits ordered binary and text cases covering every specification family;
- records exact f32 bit patterns, uses the fixed `Polygon::area()` cross
  accumulation, both union passes, and all-Subject concatenating binary union;
- models all four `Layer::merged` eligibility fields instead of collapsing
  them into a precomputed boolean;
- keeps each oracle source file below 400 physical lines and splits case
  families into real translation units rather than textual inclusion;
- runs repeatedly with identical bytes;
- can consume the released KSR K/J-body checkpoint with audited effective
  Options and produce disabled and enabled source outputs.

The frozen source-oracle rows, independently reproduced twice, are:

| Mode | Kind | Bytes | SHA-256 |
|---|---|---:|---|
| synthetic | binary | 23,615 | `7acbe44192edf030fb4b93cdab3593d83dde5800a5faa62bdb8d12002d5c8779` |
| synthetic | text | 59,481 | `6dc0fbd639b4eec91b8af2dba9fe953262ace6e550abfa9982125de03979e9a8` |
| stepped disabled | binary | 490 | `0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790` |
| stepped disabled | text | 1,405 | `576047f25c1b781477c5aff12c7d738f91710370401c109192544d71f928cf8b` |
| stepped enabled | binary | 554 | `33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505` |
| stepped enabled | text | 1,478 | `a8301caa9cd3a5a504b60eaa830379cabcf0bc88dc644b4a0996c5161409ba21` |
| KSR disabled | binary | 2,008,706 | `7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07` |
| KSR disabled | text | 2,795,812 | `6ac1b174fa012c46b43d537f42a6f35977497b244183b995fa3343c8b7f33d2c` |
| KSR enabled | binary | 2,370,813 | `46ac3ce00c40e2ba812d4f9589ce8d996949ab1e97a301243c28131865d834dc` |
| KSR enabled | text | 3,256,773 | `d77bedca008e1b54b3c32e2e746be47d51edcea5bad5e725967f964078ee2ef1` |

The source oracle and generated bytes remain ignored and are never a runtime
or tracked-test dependency. Tracked tests copy only independently frozen small
inputs, expected ordered coordinates, and hashes.

Dispatch two read-only document reviewers:

1. fixed Orca semantics, arithmetic, and oracle independence;
2. current Ares feasibility, manifest completeness, TDD isolation, and LOC
   budgets.

Both must approve the exact same spec/plan content frame. Any document change
invalidates both approvals. Production and tracked test implementation remain
forbidden until this gate is green.

## Package 1: exact Clipper operations TDD

Allowed paths:

- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/clipper.rs`;
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs`;
- `crates/ares-core/src/geometry/tests/clipper.rs`;
- `crates/ares-core/src/geometry/tests/clipper/conical_overhang.rs`.

Add compile-RED tests for crate-private operations with no production boundary:

- union of one ExPolygon vector through the existing two-pass NonZero
  Paths/PolyTree ordering, plus current-then-candidate concatenation with every
  path in the Subject role;
- XOR of arbitrary ExPolygons, including a clockwise hole path used as a solid
  contour exactly as upstream constructs it, with unchanged point order and a
  negative signed area before boolean execution;
- arbitrary ExPolygon offset with fractional f32 delta, Miter join, and limit
  3 at both scales;
- raw per-path safety expansion by exactly 10 coordinates before difference;
- overlapping clips, holes, disjoint islands, empty inputs, and coordinate
  errors;
- ordered fixed-source outputs, not only area equality.

The GREEN implementation reuses the existing `execute_ex` and raw-offset
kernel. It exposes only `union_expolygons`, `xor_ex`,
`difference_ex_with_safety_offset`, `offset_expolygons`, and the minimum path
offset seam required internally. It does not add a second boolean engine,
third-party dependency, fallback, or public API. Exact typed
`const _: fn(...) = helper;` assertions keep every new crate-private seam
reachable under normal non-test library compilation until its first production
caller lands; do not use `cfg(test)` production helpers or
`allow(dead_code)`.

Focused gate:

```text
cargo nextest run -p ares-core task22l_clipper
```

## Package 2: parameters, gates, and merged-footprint helpers TDD

Allowed paths:

- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/conical_overhang.rs`;
- `crates/ares-core/src/project_slice/conical_overhang/geometry.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang/gates.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang/geometry.rs`.

Start with tests that import absent pure helper boundaries and cover:

- raw angle NaN/infinities/below zero/above 90 and raw hole
  NaN/infinities/below zero, with exact error equality, resolved-vector order,
  and angle-before-hole precedence in the pure validation helper;
- separate pure validation and pure stage helpers proving the specified raw
  rules and empty/angle-90/derivation order without claiming an integrated
  mutation transaction before the complete apply API exists;
- pure-stage empty then 90 then derive ordering across zero layers, one layer,
  zero regions, upper empty, and all-disabled lower regions;
- upper empty, current all empty, current all disabled, and cross-region gate
  combinations through a pure pair-classification helper;
- a surface container holding an empty ExPolygon remains cardinality-nonempty;
- exact normal/large epsilon, fractional distance, and hole-area f32 bit
  patterns;
- finite `layer_height=1e34` distance and `hole_size=1e30` overflow at both
  scales, only after the empty and angle-90 returns;
- raw negative-zero angle and hole acceptance, with angle `+0` producing
  distance bits `0x80000000`, angle `-0` producing `0x00000000`, and hole
  `-0` preserving `0x80000000` after scaling;
- nominal layer height despite deliberately different planned-layer heights;
- the four merge-eligibility fields independently and all-zero exclusion;
- arbitrary non-rectangular footprint union and complete offset erosion.

Expected RED is the absent production module declaration and unresolved helper
imports. Register `mod conical_overhang;` in `project_slice.rs`, then introduce
the two real production modules only far enough to validate raw Options into a
record without precomputing scaled fields, compute fixed arithmetic, classify
one immutable layer pair, and build one merged footprint. Until Package 3 calls
them, explicit typed `const _: fn(...) = helper;` assertions keep these real
helpers warning-clean under `-D warnings`; do not use `cfg(test)` production
helpers or `allow(dead_code)`. Package 2 does not declare or implement
`apply_project_conical_overhang`, mutate a `PostRegionPrintObject`, or expose a
partial stage. The complete mutating stage lands atomically in Package 3 after
every hole/cascade/multi-region RED exists.

Focused gate:

```text
cargo nextest run -p ares-core task22l_stage
```

## Package 3: complete holes, cascade, and region ownership TDD

Allowed paths:

- the two production stage modules from Package 2;
- `crates/ares-core/src/project_slice/region_slices.rs`;
- `tests/conical_overhang.rs`;
- `tests/conical_overhang/gates.rs`;
- `tests/conical_overhang/geometry.rs`;
- `tests/conical_overhang/holes.rs`;
- `tests/conical_overhang/regions.rs`;
- `tests/conical_overhang/oracle.rs`.

First add an unresolved-import compile RED for the absent
`apply_project_conical_overhang` boundary, then add fixed-source vectors before
each behavior:

- hole-size positive and negative zero (both skipping protection), strict
  less-than, equality, full cover, partial cover, uncovered, multiple, and
  non-rectangular holes;
- three-layer top-down cascade and an interior-empty propagation barrier;
- fully covered, partially covered, and empty candidate islands;
- enabled and disabled upper-region ownership with two or more regions;
- an enabled-empty current region receiving its projection when another
  enabled nonempty current region keeps the pair-wide gate active; this RED
  must also observe the receiving region's write and the exact safety notch in
  the other region, so a per-region `current_region.is_empty()` skip fails;
- two or more enabled regions whose pair-start upper surfaces include a prior
  cascade mutation, locking ordered ownership transfer;
- same-region union and other-region raw 10-coordinate safety subtraction;
- normal and large scales, including physical safety-distance difference;
- exact default Internal surface reconstruction after same- and other-region,
  nonempty- and empty-candidate paths, plus preservation on skipped pairs and
  the never-modified top layer;
- raw validation before empty, zero-region, disabled, and angle-90 integrated
  stage gates, plus a later invalid resolved object leaving all earlier post-K
  objects byte-for-byte unchanged;
- one finite-derived-delta Clipper coordinate failure mapped to the exact
  unified stage error without identity fallback;
- unchanged plans, IDs, Options, object order, and complete sidecars;
- repeatability of ordered coordinates.

GREEN adds the complete mutating stage and exact source sequence in one change.
It maps object/instance contexts in released flattened order and maps one
Clipper error at the stage boundary. Hole coverage uses the fixed
intersection plus XOR operations. Region writes happen in order against a
fixed `current_poly`; later regions see earlier current-layer ownership
transfers exactly as upstream. No enabled input is rejected as unsupported and
no non-rectangular input is preserved as a fallback. An exact typed
`const _: fn(...) = apply_project_conical_overhang;` assertion keeps the new
stage reachable in normal non-test library compilation until Package 4 wires
its production orchestration caller; do not use `cfg(test)` or
`allow(dead_code)`.

Add only a narrow `#[cfg(test)]` constructor in `region_slices.rs` that keeps
the kind Internal while accepting thickness, thickness-layer, bridge-angle,
and extra-perimeter metadata. It exists solely to prove reset from
`(0.37, 7, 1.25, 9)` to `(-1.0, 1, -1.0, 0)` on all four mutation paths; fields
remain private and no new surface kind is introduced.

Focused gates:

```text
cargo nextest run -p ares-core task22l_stage
cargo nextest run -p ares-core task22l_oracle
```

## Package 4: orchestration, real 3MF, and exact checkpoints TDD

Allowed paths:

- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang/oracle.rs`;
- `crates/ares-core/src/project_slice/tests/conical_overhang/fixture.rs`;
- `crates/ares-core/src/project_slice/tests/region_fixture/checkpoint.rs`;
- `crates/ares-core/src/project_slice/tests/region_slices.rs`;
- `crates/ares-core/src/project_slice/tests/region_slices/complex.rs`.

Add `prepare_post_conical_overhang` after
`prepare_post_top_empty_layers`, make it the public `slice_project` path, and
first collect validated configs for every resolved object in vector order,
angle before hole, before applying L to any post-K object. Only after that
transaction succeeds, expand configs in the released flattened print-instance
order and mutate objects. This boundary does not claim precedence over loader,
effective-resolution, planning, or Tasks 22B-K errors. Then
add native test oracles:

- `task22l_browser_input_oracle`: complete K bytes;
- `task22l_browser_oracle`: post-L bytes with `ARES22L\0` magic.

Task 22K native oracles remain under `cfg(test)`. The shared parser adds L magic
without changing J/K bytes. The existing private ten-object synthetic producer
changes only to crate-test visibility. Its parent router changes the declaration
to `pub(in crate::project_slice::tests) mod complex;`, and the producer uses the
same scoped visibility, so sibling `tests/conical_overhang/oracle.rs` can name
it without making it crate-wide. That sibling owns the resolved contexts,
executes the actual L stage against the producer, and freezes the complete
transition without growing the 330-line complex-region module toward the
400-line limit.

Before orchestration GREEN, build the real stepped false/true 3MF pair in the
new fixture test module. It uses `KsrArchive` only as a deterministic ZIP and
profile container, explicitly replaces the source profile angle `55` with
`45`, replaces mesh/model/settings entries, and then differs only in the
serialized region switch. Tests assert the Rust-writer archive identities,
loaded Options, shared ordered semantic identities, identical K input, fixed
false/true L output, unchanged upper/plan/sidecar state, repeatability, exact
`-0.1`/`90.1` angle and `-0.1` hole errors under a false switch, and public
incomplete behavior. The committed KSR test also asserts all four
merged-footprint eligibility values resolved from its project settings.

Package 5 builds the same sorted entries with `fflate` and locks separate
browser-writer archive identities. Rust `zip` and `fflate` physical bytes are
not compared to each other; their semantic frames and K/L outputs must match.

Register before GREEN:

- disabled KSR L: 2,008,706 bytes /
  `7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`;
- changed L known-answer vector: 554 bytes /
  `33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505`;
- native Rust-writer stepped archives: 181,446 /
  `ee928a255109b491b0640da279b86d9282c573ec49a400e3cc4529eac915030e`
  and 181,447 /
  `be286d7abb2bef8ab5e8b650657b114ea35c4dcff3a1463eba1a0dd278a89faa`;
- browser `fflate` stepped archives: 190,380 /
  `c4c0ea05709a6fadd8b2d0d6d34dab1cad5420865c5993b58b9d8e91a8f73313`
  and 190,381 /
  `130260c5c63846759aa66d25e68ff9bb07cf5aeec86ef7da9476c12761f3836d`;
- their shared ordered semantic entries: 1,020,460 /
  `ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef`
  and 1,020,460 /
  `f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9`;
- both stepped K inputs: 490 bytes /
  `c6668cfbc56b20abe71606d59d2e28abf08ebb8b22f3ecebb3058d63ba05b44f`;
- stepped disabled L: 490 bytes /
  `0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790`;
- stepped enabled L: 554 bytes /
  `33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505`;
- ten-object L transition: the released K checkpoint is 5,848 bytes /
  `037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07`;
  before orchestration GREEN, a test-only expected encoder changes only its
  magic to `ARES22L\0` and freezes 5,848 bytes /
  `fe46d60251dcf95590c71a3e55cafdf81e0fc6af5b3cb95d58d6c39ea693b264`
  without invoking Task 22L production.

Expected RED is the absent orchestration/oracle boundary. Expected constants
must already come from the source oracle or released predecessor, never from
Task 22L production output.

Focused gates:

```text
cargo nextest run -p ares-core task22l
cargo nextest run -p ares-core task22k
```

## Package 5: feature transition and real browser

Allowed paths:

- `crates/ares-core/Cargo.toml`;
- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-wasm/Cargo.toml`;
- `crates/ares-wasm/src/lib.rs`;
- `crates/ares-wasm/tests/browser/index.html`;
- `crates/ares-wasm/tests/browser/project-slice-page.mjs`;
- `crates/ares-wasm/tests/browser/task22l-vectors.mjs`;
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`;
- `crates/ares-wasm/tests/browser/server.mjs`;
- `.github/workflows/tier1.yml`.

Replace `task22k-browser-oracle` with `task22l-browser-oracle`; do not retain an
alias. Feature exports are exactly `task22lBrowserInputOracle` and
`task22lBrowserOracle`; default exports contain no Task 22 hook.

Move the current inline browser logic into `project-slice-page.mjs`, preserving
all parser and exact-record checks. Put independent K/L KAT construction and
stepped archive entries in `task22l-vectors.mjs`; the Playwright specification
imports those tracked data builders without duplicating them. Add explicit
JavaScript routes for both modules to `server.mjs`; wildcard filesystem serving
is not introduced.

Browser RED/GREEN requirements:

1. independent K and the 554-byte /
   `33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505`
   changed-L parser KATs execute before fixture fetch;
2. truncated/trailing streams fail and all valid streams reach exact EOF;
3. public KSR path remains incomplete;
4. exact feature export set is enforced;
5. KSR K input and disabled L output match registered identities and complete
   summaries;
6. Chromium constructs the semantically identical false/true stepped 3MF pair,
   matches its registered `fflate` ZIP identities, and matches native semantic
   and fixed-source K/L identities;
7. lower retained geometry changes only when enabled while upper, plan, and
   sidecars remain identical;
8. all archives and outputs repeat;
9. Chromium runs twice from fresh optimized bindgen output.

Run default and L-feature wasm32 checks for core and adapter, isolated release
builds, wasm-bindgen 0.2.121, generated-export audit, locked dependency
installation, syntax checks, and two Playwright runs.

## Package 6: documentation, full matrix, and release

Allowed documentation paths:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

Only after Packages 1-5 are independently approved, update architecture and
roadmap with the implemented source boundary, arithmetic/ownership decisions,
exact native/browser evidence, explicit cancellation and later-stage
deferrals, next source audit boundary, and continuing
`ProjectSlicingIncomplete` status. Do not claim normalized G-code parity.

Final verification includes:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo check --workspace --all-targets --all-features`;
- `cargo nextest run -p ares-core task22l`;
- all Task 22A-L tests;
- `cargo nextest run -p ares-core`;
- `cargo nextest run --workspace`;
- core and adapter wasm32 checks for default and L feature;
- isolated default and L-feature release WASM builds and bindgen;
- generated export audit and two real Chromium runs;
- exact Option-only semantic archive diff;
- per-file LOC, no-new-source-macro, unsafe, hardcoding, fixture identity,
  planned-manifest, stale-feature/export, and `git diff --check` audits.

Freeze the exact 34-path content frame after the matrix. No validation evidence
survives a candidate-byte change.

## Mandatory independent review and repair loop

Dispatch one dedicated read-only reviewer on the frozen frame with six explicit
sections:

1. requirement completeness;
2. fixed-source logical correctness;
3. boundary and edge cases;
4. code quality and module structure;
5. test coverage and oracle independence;
6. actual native, WASM, and browser execution.

The reviewer returns P0-P3 findings, a concrete repair checklist, and an
approve/reject verdict without editing. The main thread repairs every finding,
reruns affected and full gates, freezes a new frame, and sends it back to the
same review thread for revalidation. Repeat until every list is empty.

Then obtain fresh final specification, quality, anti-hardcoding/default-model,
and documentation approvals on the unchanged frame. Any finding reopens the
same loop.

## Commit, push, and exact-SHA Tier-1

After all approvals:

1. verify the worktree diff contains exactly the 34 planned paths;
2. stage exactly those paths, excluding ignored evidence and generated output;
3. verify cached diff, LOC, macro, fixture, and no-outside-path gates;
4. create one Conventional Commit without amend or squash;
5. push the current branch normally, never force-push;
6. verify local HEAD, upstream tracking ref, and direct remote readback agree;
7. monitor the new Tier-1 run for that exact SHA through format, Ubuntu,
   Windows, macOS, and WASM/browser completion;
8. repair and repeat review/release if any job fails;
9. begin the next source-cited slicing slice only after exact-SHA Tier-1 is
   fully green.
