# Task 22M Implementation Plan: Single-Region Elephant-Foot Compensation and Slice Ordering

## Status, fixed points, and success condition

This plan is a draft. No production or tracked-test implementation is
authorized until the fixed-source oracle is complete and the exact
specification and plan bytes receive independent fixed-source/specification and
current-Ares/plan approval.

The fixed Ares baseline is commit
`fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`; exact-SHA Tier-1 run
`29718329104` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with exact blobs and ranges in the
Task 22M specification.

Success means Ares ports the complete enabled single-region elephant-foot
compensation and `Layer::make_slices()` slice-ordering boundary after released
Task 22L, consumes only typed Options from each supplied 3MF, preserves exact
upstream f32/scale and ordered geometry semantics, uses production EdgeGrid
rather than a full scan, proves fixed two-pass union, metadata reset and raw
`lslices` restoration, rejects deferred activated paths before mutation, and
passes native, WASM, Chromium, six-axis review, and exact-SHA Tier-1 gates.

Task 22M does not emit G-code or claim normalized KSR parity.

## Immutable implementation ledger

1. Task 22M runs once after released Task 22L and before later segmentation,
   classification, perimeter, fill, support, extrusion, and G-code stages.
2. It consumes `PostRegionPrintObject`, ordered resolved object contexts,
   global typed print/project Options, and the selected `CoordinateScale`.
3. Its output is `PostCompensationPrintObject { post_regions, lslices }`; the
   released post-region type is not widened.
4. Every retained nonempty object must have exactly one region; zero-region
   empty objects remain empty; valid multi-region input fails with
   `UnsupportedProjectFeature("multi_region_layer_slices")` before mutation.
5. Nonzero XY hole then contour compensation fail with exact feature keys
   before mutation. They are not clamped, ignored, or approximated.
6. Positive and negative nonzero real-project raft remain rejected by the
   released capability gate before M validation; the pure stage treats either
   sign as disabling compensation without an unsigned conversion.
7. Object compensation must be finite and nonnegative; configured compensation
   layers must be strictly positive.
8. All resolved objects, structural gates, and required layer Flow records are
   preflighted transactionally before the first mutation.
9. Flow uses actual `PlannedLayer.height` and first-layer ID zero.
10. Initial width is selected only when first-layer and raw numeric value is
    strictly positive; selected raw zero then falls back to object line width.
11. Raw percent/absolute identity survives until Flow; percent zero is not
    absolute-zero auto width.
12. The 1-based logical outer selector indexes the nozzle vector directly and
    never passes through `filament_map`; zero, underflow, and out-of-range
    indices use element-zero fallback.
13. Selected nozzle converts to f32 before percent evaluation; width, height,
    spacing, and minimum width stay f32 in source operation order.
14. Only final non-percent width `<= 0` selects `1.125f * nozzle`; spacing must
    be finite and positive.
15. Compensation scales directly to f32, ramps in f32 by layer vector index,
    and passes through the source f32 unscale/rescale round trip.
16. Dynamic normal/large-bed scale and `SCALED_EPSILON` are retained exactly.
17. Tiny ExPolygons use only the source bbox/area no-op gate.
18. Non-tiny input uses the released closed-contour simplifier, 0.5 mm
    resampling, source EdgeGrid spatial enumeration, exact filtered distance,
    distance mapping, and three-pass banded smoothing.
19. Production EdgeGrid is mandatory; the independent oracle's exhaustive scan
    is not a permitted production shortcut.
20. Variable inner offset uses per-vertex deltas and fixed miter limit 2; it is
    not a constant offset or averaged approximation.
21. The only identity fallback is the source result-count rule when variable
    offset returns a count other than one. Actual errors propagate.
22. Compensated ExPolygons are unioned with the released NonZero two-pass API:
    first to Paths, then from those Paths to a fresh PolyTree.
23. The sibling/hole/nested fixed ordering is `[left, nested, right]`; a direct
    one-pass mutant produces the forbidden `[right, left, nested]` ordering.
24. A compensated region is rebuilt as Internal surfaces with exact default
    metadata; a disabled layer preserves existing surface metadata.
25. Every retained layer runs single-region `make_slices`, including disabled
    and post-ramp layers.
26. `chain_points` retains source nearest/tie, KD-tree, mutable-priority, and
    output-index behavior; an unstable sort or unrelated nearest library is not
    equivalent.
27. Positive-ramp layers save uncompensated ExPolygons before surface rebuild,
    run normal `make_slices`, then receive independently ordered raw backups as
    final `lslices`.
28. Planned layers, object/region IDs, Options, object order, and complete
    occurrence sidecars do not change.
29. Geometry/nonfinite errors map exactly once to the Task 22M geometry error;
    they never become identity output.
30. Existing G-L and new M checkpoint entrypoints live in a real
    `project_slice/checkpoints.rs` module so `project_slice.rs` remains below
    400 LOC.
31. M framing is exact L object/sidecar/region framing plus ordered `lslices`
    directly after each retained layer's regions.
32. KSR L input is 2,008,706 bytes / `7a71db29...4b07`; M output is 3,008,346
    bytes / `91f6943a...8d19` and is produced by actual production code.
33. Public slicing executes M and still returns `ProjectSlicingIncomplete`.
34. The L browser feature and vector module are replaced by M; no alias or
    stale export remains.
35. Default WASM has no Task 22 export; feature WASM has exactly M input/output.
36. Native and browser tests use real serialized Options and semantic archive
    identities, never a filename/hash behavior switch.
37. Tracked tests do not inspect Git or Orca source. Fixed identities and the
    C++ oracle remain ignored evidence only.
38. Every Rust production/test file is below 400 physical lines, tests use real
    modules, and source splitting macros are forbidden.
39. No unsafe code, new native dependency, filesystem access, legacy fallback,
    or speculative later-stage abstraction is introduced.
40. Cancellation, painted/MMU/fuzzy/interlocking segmentation, XY and
    multi-region algorithms, and every later slicing/G-code boundary remain
    explicit deferrals.

## Exact planned tracked manifest

No tracked path outside this 49-path list may change without a plan amendment
and fresh exact-byte document approval. Every listed path must change; missing,
extra, or substituted paths block implementation closure.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
- `docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature and geometry production

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/clipper.rs`
- `crates/ares-core/src/geometry/clipper/variable_offset.rs`
- `crates/ares-core/src/geometry/edge_grid.rs`
- `crates/ares-core/src/geometry/edge_grid/raster.rs`
- `crates/ares-core/src/geometry/chain_points.rs`
- `crates/ares-core/src/geometry/chain_points/kd_tree.rs`
- `crates/ares-core/src/geometry/chain_points/priority_queue.rs`

### Geometry tests

- `crates/ares-core/src/geometry/tests.rs`
- `crates/ares-core/src/geometry/tests/clipper.rs`
- `crates/ares-core/src/geometry/tests/clipper/variable_offset.rs`
- `crates/ares-core/src/geometry/tests/edge_grid.rs`
- `crates/ares-core/src/geometry/tests/edge_grid/raster.rs`
- `crates/ares-core/src/geometry/tests/chain_points.rs`
- `crates/ares-core/src/geometry/tests/chain_points/kd_tree.rs`
- `crates/ares-core/src/geometry/tests/chain_points/priority_queue.rs`

### Project stage, checkpoint, and orchestration

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/checkpoints.rs`
- `crates/ares-core/src/project_slice/task22m_oracle.rs`
- `crates/ares-core/src/project_slice/compensation.rs`
- `crates/ares-core/src/project_slice/compensation/flow.rs`
- `crates/ares-core/src/project_slice/elephant_foot.rs`
- `crates/ares-core/src/project_slice/elephant_foot/distance.rs`
- `crates/ares-core/src/project_slice/elephant_foot/profile.rs`
- `crates/ares-core/src/project_slice/slice_ordering.rs`

### Project-stage tests

- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/compensation.rs`
- `crates/ares-core/src/project_slice/tests/compensation/flow.rs`
- `crates/ares-core/src/project_slice/tests/compensation/gates.rs`
- `crates/ares-core/src/project_slice/tests/compensation/fixture.rs`
- `crates/ares-core/src/project_slice/tests/elephant_foot.rs`
- `crates/ares-core/src/project_slice/tests/elephant_foot/distance.rs`
- `crates/ares-core/src/project_slice/tests/elephant_foot/profile.rs`
- `crates/ares-core/src/project_slice/tests/elephant_foot/oracle.rs`
- `crates/ares-core/src/project_slice/tests/slice_ordering.rs`

### WASM, browser, and Tier-1

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/task22l-vectors.mjs` (delete)
- `crates/ares-wasm/tests/browser/task22m-vectors.mjs` (add)
- `crates/ares-wasm/tests/browser/project-slice-page.mjs`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `crates/ares-wasm/tests/browser/server.mjs`
- `.github/workflows/tier1.yml`

Ignored evidence ledgers, fixed-source oracle files, object/executable output,
temporary targets, bindgen output, Playwright output, and generated in-memory
archives are never staged. `region_slices.rs`, the typed Option declarations,
the released simplifier, existing conical-overhang modules, browser
`index.html` and package files, `Cargo.lock`, and both committed KSR fixtures
remain unchanged.

## Module and line budgets

Every changed Rust production and test file remains strictly below 400
physical LOC. Planned upper budgets are:

- `crates/ares-core/src/lib.rs`: 290;
- `geometry.rs`: 80;
- `geometry/clipper.rs`: 190;
- `geometry/clipper/variable_offset.rs`: 360;
- `geometry/edge_grid.rs`: 300;
- `geometry/edge_grid/raster.rs`: 280;
- `geometry/chain_points.rs`: 260;
- `geometry/chain_points/kd_tree.rs`: 300;
- `geometry/chain_points/priority_queue.rs`: 280;
- geometry test roots: 40;
- every geometry leaf test: 390;
- `project_slice.rs`: 300 after checkpoint extraction;
- `project_slice/checkpoints.rs`: 260;
- `project_slice/task22m_oracle.rs`: 260;
- `project_slice/compensation.rs`: 300;
- `project_slice/compensation/flow.rs`: 260;
- `project_slice/elephant_foot.rs`: 360;
- `project_slice/elephant_foot/distance.rs`: 390;
- `project_slice/elephant_foot/profile.rs`: 330;
- `project_slice/slice_ordering.rs`: 220;
- project test roots: 50;
- every project leaf test: 390;
- `crates/ares-wasm/src/lib.rs`: 160.

Browser budgets are `project-slice-page.mjs` at most 390 physical lines,
`task22m-vectors.mjs` at most 350, `project-slice.spec.mjs` at most 350, and
`server.mjs` at most 80. If any budget is insufficient, split by responsibility
through another real module, amend the exact manifest, and reapprove the
documents before editing. Rust `include!`, source-organizing `include_bytes!`
or `include_str!`, and textual source inclusion are forbidden.

## Working protocol

Packages 1 through 6 change executable behavior or an executable adapter and
must use strict TDD:

1. add the named test module and focused failing behavior first;
2. run the focused nextest/check command and record the expected RED;
3. add the minimum production code for that behavior;
4. rerun GREEN, format, strict clippy for affected targets, LOC, macro, unsafe,
   hardcoding, and exact-path checks;
5. freeze the package diff and obtain independent read-only review before the
   next package depends on it.

An unresolved import or missing module is an acceptable compile RED only when
the production API truly does not exist. Once a helper exists, later REDs must
exercise wrong behavior, not delete or disable production code. Expected
vectors must come from the approved fixed oracle or released predecessor before
the corresponding GREEN; production output is never copied back as its own
expected value.

Temporary `const _: fn(...) = helper;` assertions may keep a new crate-private
production seam warning-clean until its next planned caller lands. Do not use
`allow(dead_code)`, test-only production implementations, feature-gated native
shortcuts, broad fallbacks, or partial mutating stages that can escape through
the public API.

The main thread owns integration files shared across packages:
`geometry.rs`, `geometry/clipper.rs`, `project_slice.rs`, test roots, Cargo
features, WASM exports, workflow, architecture, and roadmap. Parallel workers
may edit only disjoint leaf modules assigned by the main thread. Every worker
returns exact changed paths, commands, output summaries, LOC, and open risks.
Whenever a package lists allowed leaf paths, the main thread may additionally
edit only the manifest-listed registration roots needed to declare those leaf
modules and keep normal compilation warning-clean. That permission does not
extend to unrelated implementation in a shared root.

## Package 0: fixed-source oracle and exact document approval

The corrected ignored oracle cleanly refreezes the following approval
candidates:

- synthetic binary 10,351 bytes /
  `c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d`;
- synthetic text 17,407 bytes /
  `daa902bf4d1bf93d16e8c1b22427432ffe37d0c5d73728967f08bcf7a5d57e72`;
- KSR binary 3,008,346 bytes /
  `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`;
- KSR text 2,528,073 bytes /
  `abbe1ce7bdfdda06f4e9e6e581c2e08b4ff29051322bf22c92c5daaf62e79833`.

Local clean compilation deleted all Task 22M obj/exe output, rebuilt the fixed
Clipper translation unit with `/W0` and every oracle-owned translation unit with
`/W4 /WX` under VS2022 C++20 `/O2 /fp:precise /DNDEBUG`, reran synthetic and KSR
twice, and matched each pair byte-for-byte. Reviewers must independently verify
at least
`ClipperUtils.cpp:303-344,634-668,737-739,813-816,1019-1031`,
`ClipperUtils.hpp:34,183-222,548-550`, `ExPolygon.cpp:50-56,229-254`,
`Polygon.cpp:52-68`, `MultiPoint.cpp:164-230`, `EdgeGrid.cpp:28-334`,
`EdgeGrid.hpp:15-356`, `ShortestPath.cpp:83-419,1000-1011,1106-1115`,
`ElephantFootCompensation.cpp:20-28,233-447,465-532,544-644`,
`Utils.hpp:305-408`, and `libslic3r.h:299-303`. The tracked Rust metadata-reset
test remains mandatory because the C++ wire excludes non-type Surface metadata.

After these two documents exist, compute one deterministic content frame from
sorted relative paths, byte lengths, and SHA-256 values. Dispatch two read-only
reviewers in parallel:

1. fixed Orca boundary, exact arithmetic, oracle independence, scope and
   deferral reviewer;
2. current Ares types/APIs, manifest completeness, package feasibility, TDD,
   WASM/browser, and LOC reviewer.

Both must approve the same exact frame. Any document change invalidates both
approvals and requires a new frame and two new reviews. Production and tracked
test code remain forbidden until the dual gate is green.

## Package 1: external Flow resolver TDD

Allowed leaf paths:

- `project_slice/compensation.rs`;
- `project_slice/compensation/flow.rs`;
- `project_slice/tests/compensation.rs`;
- `project_slice/tests/compensation/flow.rs`;
- `project_slice/tests/compensation/gates.rs`.

Register compile-RED tests for an absent raw typed Flow resolver and validated
Task 22M config record. Freeze:

- first-layer initial selection versus later outer selection;
- initial absolute zero falling to outer, selected outer zero falling to
  object, then final absolute zero/negative auto width;
- percent values, percent zero spacing failure, and negative percent failure;
- selected nozzle f32 conversion before percent arithmetic;
- selectors 0, 1, 2, -1, `i32::MIN`, and out-of-range fallback, with no
  `filament_map` parameter or lookup in the pure resolver;
- width/height/spacing/minimum-width exact f32 bits from the approved oracle;
- per-layer height 0.2 versus 0.3 and first-layer identity by ID;
- empty nozzle, selected zero/negative nozzle, invalid height, and nonpositive
  spacing exact errors;
- compensation/layer raw validation for zero, negative one, and `i32::MIN`,
  public positive/negative raft gate ordering, and XY feature-key ordering; and
- pure all-config validation returning all records or one error without mutating
  state; structural and complete per-layer Flow transactionality remains the
  Package 5 orchestration gate.

GREEN introduces only immutable validated records and a crate-private pure
resolver. It pattern-matches `FloatOrPercent` exhaustively and preserves raw
variant identity. It does not call the old general extrusion-width
compatibility layer, precompute geometry, mutate regions, or wire public
slicing. Keep it warning-clean through an exact typed function assertion until
Package 5.

Focused gate:

```text
cargo nextest run -p ares-core task22m_flow
```

## Package 2: chain-points and single-region slice ordering TDD

Allowed leaf paths:

- `geometry/chain_points.rs`;
- `geometry/chain_points/kd_tree.rs`;
- `geometry/chain_points/priority_queue.rs`;
- `geometry/tests/chain_points.rs`;
- `geometry/tests/chain_points/kd_tree.rs`;
- `geometry/tests/chain_points/priority_queue.rs`;
- `project_slice/slice_ordering.rs`;
- `project_slice/tests/slice_ordering.rs`.

Begin with absent-module REDs and source-derived ordered vectors for:

- empty, singleton, nearest-neighbor, equal-distance, duplicate-point, negative
  and large i64 coordinate cases;
- mutable-priority insert/update/remove ordering and equal-key tie behavior;
- KD-tree capacity, next-power-of-two growth, partition/search behavior, and
  deterministic index selection;
- multi-island first-point ordering with holes left attached;
- one empty retained layer, multiple retained layers, and exact per-layer
  output length; and
- uncompensated backup chaining independent of current compensated surfaces.

GREEN ports the minimum fixed-source point-only KD and mutable-priority support
required by `chain_points`, not generic public data structures. Add one
single-region `make_slices` helper that consumes existing RegionSurface
geometry through existing accessors and returns ordered `Vec<ExPolygon>`.
Do not yet introduce the wrapper or mutate project state. No third-party
nearest-neighbor dependency is added.

Focused gates:

```text
cargo nextest run -p ares-core task22m_chain_points
cargo nextest run -p ares-core task22m_slice_ordering
```

## Package 3: released two-pass union regression and variable inner offset TDD

Allowed leaf paths:

- `geometry/clipper/variable_offset.rs`;
- `geometry/tests/clipper/variable_offset.rs`.

Before production changes, rerun the released Task 22F two-pass union regression
whose sibling/hole/nested output is `[left, nested, right]` and whose direct
one-pass mutant is `[right, left, nested]`. Then add behavioral REDs for:

- fixed-source constant-delta and varying-delta variable inner offsets;
- convex, concave, collinear, acute-miter, limited-miter, contour/hole,
  erosion, split and empty repair cases;
- exact normal and large-scale coordinates; and
- actual Clipper failures propagating rather than returning the input.

GREEN adds only the variable-offset real module. Reuse the released
`union_expolygons`/`union_ex` two-pass behavior and existing Clipper types; do
not add a new union helper or alter the earlier union implementation. The
variable-offset API accepts exact per-contour per-vertex delta vectors and miter
limit and returns a Result. It does not implement the elephant-foot result-count
fallback; that belongs in Package 4.

Focused gate:

```text
cargo nextest run -p ares-core task22f_union_ex_uses_paths_then_fresh_polytree_and_exact_recursive_order
cargo nextest run -p ares-core task22m_variable_offset
```

## Package 4: EdgeGrid and full elephant-foot kernel TDD

Allowed leaf paths:

- `geometry/edge_grid.rs`;
- `geometry/edge_grid/raster.rs`;
- `geometry/tests/edge_grid.rs`;
- `geometry/tests/edge_grid/raster.rs`;
- `project_slice/elephant_foot.rs`;
- `project_slice/elephant_foot/distance.rs`;
- `project_slice/elephant_foot/profile.rs`;
- `project_slice/tests/elephant_foot.rs`;
- `project_slice/tests/elephant_foot/distance.rs`;
- `project_slice/tests/elephant_foot/profile.rs`;
- `project_slice/tests/elephant_foot/oracle.rs`.

Build REDs in increasing scope:

- grid bbox, resolution, dimensions, contour/segment ownership, horizontal,
  vertical, diagonal, boundary-touching and corner-crossing raster insertion;
- query boxes outside, inside, and across grid boundaries, with exact visited
  cell/candidate order and no hidden full scan;
- f64 closest-foot projection, inward direction, strict nearest replacement,
  contour/hole orientation, same-contour near reject, far accept, convex and
  concave corner predicates, the exact strict
  `distance < search_radius + SCALED_EPSILON` boundary, bulge threshold, and
  search-radius cap;
- closed simplification seam and orientation through the released helper;
- exact 0.5 mm resampling, source indices, interpolated flags, step lengths,
  accumulated curve parameters, and coordinate casts;
- three exact smoothing passes, band interpolation, cyclic walks, and the
  negative-compensation monotonic rule;
- tiny bbox/area no-op, distance-to-delta thresholds, normal/large scale,
  minimum width from Flow, and f32 compensation round trip;
- variable-offset count 1 success and count 0/>1 source fallback, while actual
  errors propagate; and
- all ordered 19-case oracle outputs or bounded exact case projections,
  including the two-pass-union/direct-one-pass mutant kill and repeatability.

Expected vectors are copied from the approved oracle before GREEN. GREEN ports
the production uniform EdgeGrid and exact filtered-distance/profile/kernel
semantics. The kernel consumes validated Flow/scaling records, processes every
input ExPolygon, calls the Package 3 variable offset and released two-pass
union, and owns
only the source result-count identity fallback. No full-scan production branch,
parallel scheduler, cancellation shim, or broad error fallback is added.

Focused gates:

```text
cargo nextest run -p ares-core task22m_edge_grid
cargo nextest run -p ares-core task22m_elephant_foot
cargo nextest run -p ares-core task22m_oracle
```

## Package 5: transaction, wrapper, real 3MF, and M checkpoint TDD

Allowed leaf paths:

- `project_slice/checkpoints.rs`;
- `project_slice/task22m_oracle.rs`;
- `project_slice/compensation.rs`;
- `project_slice/tests/compensation.rs`;
- `project_slice/tests/compensation/gates.rs`;
- `project_slice/tests/compensation/fixture.rs`;
- `project_slice/tests/elephant_foot/oracle.rs`;
- `project_slice/tests/slice_ordering.rs`.

The main thread first moves existing G-L checkpoint entrypoints from
`project_slice.rs` into `checkpoints.rs` without changing bytes, tests or
feature visibility. Run every Task 22G-L checkpoint test and compare the
released KSR L input identity before adding M behavior.

Then add absent-boundary REDs for the complete project apply API and wrapper:

- raw config validation before zero-layer, zero-region, disabled and later
  invalid objects, with earlier objects unchanged;
- exact XY and multi-region feature keys before mutation;
- signed raft values `-1` and `1` disable pure compensation but still compute
  ordered current `lslices` without unsigned conversion;
- compensation-layer clamp and f32 ramp across actual shorter/longer objects;
- compensated surface metadata reset from a seeded nondefault tuple to exact
  Internal defaults;
- disabled and post-ramp surface metadata preservation;
- compensated surfaces versus ordered uncompensated `lslices`;
- plan, sidecar, region Option/ID, object order and layer-count preservation;
- empty object/wrapper behavior and exact per-layer `lslices` cardinality;
- one later invalid Flow preserving the pre-mutation transaction, and one
  geometry failure returning no observable partial wrapper/output;
- complete synthetic `ARES22M` frame matching the approved oracle;
- released Task 22L input unchanged; and
- truncation/trailing/EOF parser behavior.

Before GREEN, generate in-memory real 3MF archives whose semantic entries
differ only in embedded compensation/width/nozzle/map/XY/region-count Options.
Freeze their loaded typed values, ordered semantic entry hashes, L inputs and
expected fixed-source M outputs before the production stage exists. The
disabled/enabled pair must observe real metadata and raw-backup behavior, not
only output size. A dedicated anti-map pair uses
`nozzle_diameter = [0.4, 0.6]`, 125 percent width, and outer selector one; its
archives differ only between `filament_map = [1, 2]` and `[2, 1]`. Both exact
widths are 0.5 mm and both M frames are byte-identical; incorrect pre-mapping
makes the second width 0.75 mm. This pair must pass through effective config
and project orchestration so an incorrect logical-to-physical pre-map cannot
hide behind the pure resolver.

GREEN adds a consuming, transactional wrapper stage. It first validates every
raw config, structural gate, and needed Flow record across the project. It then
owns the input objects while building outputs, so a later geometry error drops
the incomplete owned result and exposes no partially mutated state without
copying the full geometry tree. For every positively
compensated layer it moves/copies raw ExPolygons, resolves its preflighted Flow,
runs the full kernel, rebuilds Internal surfaces, and saves the raw vector. It
then runs `make_slices` for all layers and restores ordered raw backups. Public
orchestration calls this stage after `prepare_post_conical_overhang` and
destructures the wrapper only at the existing incomplete sink.

The M encoder writes exact L identity/sidecars, then for each retained layer
writes layer index, all regions and surfaces, followed immediately by that
layer's `lslices`. Register the committed KSR vectors before GREEN:

- L input: 2,008,706 bytes /
  `7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`;
- M output: 3,008,346 bytes /
  `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`.

The real KSR test also proves one object/occurrence, 460 retained layers, one
region per layer, loaded effective Options, six raw first-layer islands,
compensated first-layer surfaces, uncompensated first-layer `lslices`, later
ordered current slices, unchanged sidecars/plan, repeatability, and public
`ProjectSlicingIncomplete`.

Focused gates:

```text
cargo nextest run -p ares-core task22m
cargo nextest run -p ares-core task22l
cargo nextest run -p ares-core task22k
```

## Package 6: feature transition and real browser

Allowed paths are the manifest's core/adapter Cargo and lib files, deletion of
`task22l-vectors.mjs`, addition of `task22m-vectors.mjs`, the existing browser
page/spec/server, and Tier-1 workflow.

Replace `task22l-browser-oracle` with `task22m-browser-oracle`; do not retain an
alias. Feature exports are exactly `task22mBrowserInputOracle` and
`task22mBrowserOracle`; default exports contain no Task 22 hook. Native Task
22L helpers remain only under `cfg(test)`.

Browser RED/GREEN requirements are:

1. independent small L and M parser KATs execute before fixture fetch;
2. the two-pass-union discriminant and one raw-backup layer are parsed with exact
   ordered coordinates;
3. truncated/trailing frames fail and valid frames reach exact EOF;
4. default and feature export sets are exact;
5. Chromium loads real KSR bytes and matches registered L/M sizes, hashes, and
   complete summaries;
6. Chromium builds semantically identical disabled/enabled Option-only 3MF
   archives with `fflate`, freezes their browser ZIP and shared semantic-entry
   identities, and matches native stage semantics;
7. enabled region surfaces change, raw `lslices` stay uncompensated, disabled
   surfaces stay unchanged, and plan/sidecars remain identical;
8. public slicing remains incomplete;
9. every archive and frame repeats; and
10. Chromium runs twice from fresh optimized bindgen output.

Add explicit server routes for the M vectors module. Do not add wildcard
filesystem serving or change browser package dependencies. Run default and
M-feature wasm32 checks for core/adapter, isolated release builds,
wasm-bindgen 0.2.121, generated-export audit, Node syntax checks, locked
dependency install, and two Playwright Chromium runs.

## Package 7: documentation, full matrix, and release

Only after Packages 1-6 receive package review approval, update:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

Record the implemented source boundary, raw Option/Flow semantics, exact
f32/scale decisions, EdgeGrid versus oracle independence, two-pass-union mutant
kill, wrapper and backup lifecycle, native/browser evidence, explicit
deferrals, next fixed Orca boundary, and continuing
`ProjectSlicingIncomplete` status. Do not claim G-code equality.

Final native verification includes:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo nextest run -p ares-core task22m
cargo nextest run -p ares-core task22l
cargo nextest run -p ares-core task22
cargo nextest run -p ares-core
cargo nextest run --workspace
```

Final WASM/browser verification includes default and M-feature wasm32 checks
for core/adapter, isolated release builds, bindgen 0.2.121, exact generated
exports, Node syntax checks, `npm ci`, and two real Chromium runs.

Final structural verification includes:

- exact 49-path tracked manifest and no ignored/generated staging;
- every changed Rust production/test file below 400 physical LOC;
- no source-splitting macro, textual source include, new unsafe, or broad
  warning allowance;
- no tracked executable source pin, Git/Orca runtime inspection, filename/hash
  branch, fixture/reference G-code access, or production constant copied from
  KSR inventory;
- fixed project and reference fixture hashes unchanged;
- exact G-L predecessor bytes unchanged;
- no stale L browser feature, export, vector file, or workflow flag;
- semantic archive diff limited to intended Options;
- repeated KSR L/M output identity; and
- `git diff --check`.

Freeze one exact sorted 49-path content frame after the complete matrix. No
validation or review evidence survives a candidate-byte change.

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
reruns affected focused gates and the complete matrix, freezes a new frame, and
sends it back to the same review thread. Repeat until all six lists are empty
and the verdict is approve.

Then obtain fresh independent final specification, quality,
anti-hardcoding/default-model, and documentation approvals on the unchanged
frame. Any finding or byte change reopens the same repair/revalidation loop.

## Commit, push, and exact-SHA Tier-1

After all approvals:

1. verify the worktree diff contains exactly the 49 planned paths;
2. stage exactly those paths, excluding ignored oracle evidence and generated
   output;
3. verify cached diff, LOC, macro, unsafe, fixture, stale-feature, and
   no-outside-path gates;
4. create one Conventional Commit without amend or squash;
5. push the current branch normally, never force-push;
6. verify local HEAD, upstream tracking ref, and direct remote readback agree;
7. monitor the new Tier-1 run for that exact SHA through format, Ubuntu,
   Windows, macOS, and WASM/browser completion;
8. repair, rerun the full review loop, recommit, repush, and remonitor if any
   job fails; and
9. begin the next source-cited slicing slice only after exact-SHA Tier-1 is
   fully green.
