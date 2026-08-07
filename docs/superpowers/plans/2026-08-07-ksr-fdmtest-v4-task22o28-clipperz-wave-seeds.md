# Implementation Plan

## Goal

Implement approved Task 22O.28 as a source-cited, crate-private Rust rewrite of pinned OrcaSlicer `Algorithm::wave_seeds`, extending only the existing ARD-0024 indexed Clipper kernel with private Z provenance while leaving project lifecycle and public APIs unchanged.

Execution is complete. Final documented-state independent and default-model
OpenCode reviews returned `VERDICT: APPROVE`; implementation commit `7eb0d27`
and documentation commit `be33437` were pushed; exact-SHA Tier-1 run
`31156094839` passed at `be334375be871eb12ca98c98d889b65a92d13a37`.

## Source and Scope Contract

- Approved spec: `docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o28-clipperz-wave-seeds.md`
- Exact Ares predecessor: `f361bb73b558b4e50bfa4fa712afcd63df44ba9f`
- Pinned OrcaSlicer commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`
- Primary upstream boundary:
  - `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.hpp:38-68`
  - `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.cpp:88-391`
  - `OrcaSlicer/src/libslic3r/ClipperZUtils.hpp:14-139`
  - `OrcaSlicer/src/libslic3r/Polyline.hpp:232-250`
  - `OrcaSlicer/deps_src/clipper/clipper.hpp:46-47,99-135,230-279,441-479,500-533`
  - `OrcaSlicer/deps_src/clipper/clipper.cpp:78-113,472-479,1617-1683,2002-2040,2284-2314,2588-2643,2694-2760,2927-3015,4121-4166`
  - `OrcaSlicer/src/libslic3r/AABBTreeIndirect.hpp:37-210,221-236,940-987`
  - `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:33-39`
- No source-taking propagation overload, external-surface processing, project checkpoint, CLI/WASM export, option implementation, or G-code behavior enters this milestone.
- `ProjectSlicingIncomplete` remains the public project-slicing result at the existing O26/O27 boundary.
- Any production touchpoint not explicitly allowed by the approved spec requires a spec amendment and renewed independent plus default-model OpenCode approval before editing.

## Tasks

1. **Freeze the approved predecessor, source evidence, and implementation boundary**
   - Files:
     - `docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o28-clipperz-wave-seeds.md`
     - `docs/superpowers/plans/2026-08-07-ksr-fdmtest-v4-task22o28-clipperz-wave-seeds.md`
   - Changes:
     - Verify `HEAD == origin/main == f361bb73b558b4e50bfa4fa712afcd63df44ba9f`.
     - Verify the OrcaSlicer checkout is exactly `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
     - Record the green predecessor Tier-1 runs `31127440442` and `31126818275`.
     - Confirm no staged files and exclude `.pi-subagents/` and `target/parity/` from all commits.
     - Record physical LOC for every allowed existing touchpoint, especially:
       - `clipper/ordering.rs`, currently near 400 LOC;
       - `clipper/output/join_points.rs`, currently near 400 LOC;
       - `clipper/types.rs`.
     - Search all uses of `Edge.current/bottom/top`, `IntersectionNode.point`, `OutPoint.point`, `Join.offset`, and `GhostJoin.offset`.
     - The approved audits permit mechanical constructor/predicate/join-offset conversions in `clipper/minima.rs` and `clipper/strictly_simple.rs`; preliminary implementation review additionally permits only the pinned direction-sensitive closed-horizontal `SetZ` fill in `horizontals.rs` and only pinned type-3 `SetZ(candidate, previous, current)` before its two output writes/join in `strictly_simple.rs`. It requires `clipper/active_edges.rs`, `clipper/bounds.rs`, `clipper/winding.rs`, `clipper/output/open_fixup.rs`, `clipper/output/joins.rs`, and `clipper/output/simple.rs` to compile unchanged through `KernelPoint`'s XY-only access/equality seams. Any other production touchpoint requires a spec amendment and dual re-review before editing.
     - Obtain literal `VERDICT: APPROVE` from an independent plan reviewer and default-model OpenCode before production changes.
   - Acceptance:
     - The base SHA, pinned source SHA, clean staging state, allowed touchpoint inventory, LOC baseline, and both plan approvals are archived under `/tmp/task22o28-*`.
     - The implementation allowlist is fixed before RED tests.

2. **Generate and freeze the offline pinned C++ oracle**
   - Files:
     - No repository files; all harnesses and output remain under `/tmp/task22o28-oracle-*`.
   - Changes:
     - Build focused debug and `NDEBUG` C++ harnesses from the pinned files above, using the bundled ClipperZ source only as an offline test oracle. For every path affected by comparator-equivalent ordering, compile and run on the audited MSVC compatibility target `_MSVC_STL_VERSION=143`, `_MSVC_STL_UPDATE=202503L`, toolset directory `14.44.35207`, after verifying SHA-256 `e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7` for `algorithm` and `56c6be67b7c0ff9b3ffb7d48943c1ec01728f41f0663dca2c49c296f492bf619` for `__msvc_heap_algorithms.hpp`. Archive the exact compiler commands, macro probe, toolset path, header hashes, exits, and outputs under `/tmp/task22o28-oracle-*`; a host `std::sort` result is not ordering evidence.
     - Record complete ordered data, not counts or areas:
       - expanded/opened Z paths;
       - intersection table pairs;
       - PolyTree-flattened paths before merge;
       - paths after exact split reconciliation;
       - final `WaveSeed` IDs and XY point sequences.
     - Include direct crossing, shared vertices, split contour, contour/hole, closed fallback, multiple IDs, overlapping-boundary fallback, and more-than-32 equal-key sort vectors.
     - Record executable debug and `NDEBUG` compile/run commands so release-only behavior can be distinguished from debug assertions.
     - Do not check in C++, raw/generated oracle output, source text, source hashes, line-pinning tests, or binary oracle payloads. Human-reviewed Rust literal expectations transcribed from the ordered captures are permitted in ordinary test shards; they must name behavior rather than pin source text, lines, hashes, or a serialized oracle blob.
   - Acceptance:
     - Human-readable ordered oracle captures exist in `/tmp`.
     - The harness has no production, build-script, FFI, subprocess, fixture-identity, or runtime dependency.

3. **Add compiling RED scaffolding for private Z paths without changing ordinary 2-D output**
   - Files:
     - `crates/ares-core/src/geometry/clipper.rs`
     - `crates/ares-core/src/geometry/clipper/types.rs`
     - `crates/ares-core/src/geometry/clipper/input.rs`
     - `crates/ares-core/src/geometry/clipper/engine.rs`
     - New `crates/ares-core/src/geometry/clipper/z.rs`
     - New test roots and shards listed under “New Files”.
   - Changes:
     - Declare `geometry/clipper/z.rs` and test modules.
     - Introduce geometry-private `KernelPoint`/`ZPath` shells and narrow Z add/execute test seams, initially projecting through the existing 2-D path so provenance assertions fail while code compiles.
     - Keep `Point`, `Polygon`, and `Polyline` structurally unchanged.
     - Add compiling RED tests for:
       - Z-bearing closed/open input and range validation;
       - XY-only equality, cleanup, closure, and immediate output dedup;
       - unchanged complete ordered 2-D outputs;
       - output Z retention, endpoint priority, callback bypass, self-intersection, negative table indexing, clear/reuse, both direction-sensitive closed-horizontal fills, and strictly-simple type-3 fill.
     - Put release-only cases in explicit `release` test modules so the approved `cargo nextest run --release ...::release` filters execute real tests.
     - Capture every RED command, exit code, and assertion excerpt under `/tmp/task22o28-red-*`.
   - Acceptance:
     - REDs are compiling assertion failures, not missing imports or unresolved symbols.
     - Existing 2-D Clipper tests remain green before the internal migration.
     - No Z type is reachable through `lib.rs`, CLI, WASM, or public geometry records.

4. **Migrate the single indexed kernel from internal `Point` records to `KernelPoint`**
   - Files:
     - `crates/ares-core/src/geometry/clipper/z.rs`
     - `crates/ares-core/src/geometry/clipper/types.rs`
     - `crates/ares-core/src/geometry/clipper/predicates.rs`
     - `crates/ares-core/src/geometry/clipper/input.rs`
     - `crates/ares-core/src/geometry/clipper/input/path.rs`
     - `crates/ares-core/src/geometry/clipper/input/bounds.rs`
     - `crates/ares-core/src/geometry/clipper/intersections.rs`
     - `crates/ares-core/src/geometry/clipper/intersections/open.rs`
     - `crates/ares-core/src/geometry/clipper/intersections/top.rs`
     - `crates/ares-core/src/geometry/clipper/horizontals.rs`
     - `crates/ares-core/src/geometry/clipper/minima.rs`
     - `crates/ares-core/src/geometry/clipper/strictly_simple.rs`
     - `crates/ares-core/src/geometry/clipper/output/rings.rs`
     - `crates/ares-core/src/geometry/clipper/output/fixup.rs`
     - `crates/ares-core/src/geometry/clipper/output/append.rs`
     - `crates/ares-core/src/geometry/clipper/output/join_points.rs`
     - `crates/ares-core/src/geometry/clipper/output/ownership.rs`
   - Changes:
     - Define `KernelPoint { xy: Point, z: i64 }` with `pub(in crate::geometry)` visibility.
     - Implement ordinary `PartialEq`/`Eq` as XY-only, matching `clipper.cpp:109-110`.
     - Add explicit full-XYZ equality and X/Y/Z lexicographic helpers only; do not derive `Ord`.
     - Convert `Edge.current/bottom/top`, `IntersectionNode.point`, `OutPoint.point`, `Join.offset`, and `GhostJoin.offset` to `KernelPoint`. Keep cfg(test) `EdgeSnapshot`/`InputSnapshot` fields as `Point` by projecting `.xy`, preserving all existing snapshot tests.
     - Keep existing 2-D input adapters assigning `z = 0`; keep existing outputs projecting only `.xy`.
     - Route geometry predicates through explicit `.xy` projections so Z never affects cleanup, closure, range, slopes, winding, area, point-in-polygon, joins, or output ordering.
     - Preserve complete metadata through whole-record copies:
       - edge promotion retains complete metadata automatically;
       - horizontal reversal swaps endpoint X and Z together;
       - scanbeam X projection retains current Z;
       - top updates choose top Z, bottom Z, or zero exactly by `top_y`;
       - output allocation and duplication copy complete XYZ;
       - immediate XY dedup retains the already-stored node’s Z;
       - fixup removes only the selected node and transfers no Z;
       - join copy/replacement copies or overwrites full XYZ from the exact source node.
     - Keep changes to `ordering.rs` line-neutral where possible; it is not part of this migration.
     - Do not modify unlisted files merely to simplify conversions. If an unlisted edit is genuinely required, invoke the spec-amendment gate from Task 1.
   - Acceptance:
     - Kernel-focused tests prove every metadata survivor/copy/overwrite rule.
     - Existing ordered 2-D Boolean, open-path, strict-simple, offset, PolyTree, and O27 tests are byte-for-byte/order-identical.
     - Every touched Rust file remains below 400 physical lines; `join_points.rs` is split only after an approved spec amendment if it cannot stay below the limit.

5. **Implement exact `SetZ`, collector ownership, and Z execution**
   - Files:
     - `crates/ares-core/src/geometry/clipper/z.rs`
     - `crates/ares-core/src/geometry/clipper.rs`
     - `crates/ares-core/src/geometry/clipper/engine.rs`
     - `crates/ares-core/src/geometry/clipper/intersections.rs`
     - `crates/ares-core/src/geometry/clipper/intersections/open.rs`
     - `crates/ares-core/src/geometry/clipper/intersections/top.rs`
     - `crates/ares-core/src/geometry/clipper/horizontals.rs`
     - `crates/ares-core/src/geometry/clipper/strictly_simple.rs`
   - Changes:
     - Add narrow Z-path input methods to the same `Clipper`; do not create an alternate engine.
     - Before every open/closed intersection branch:
       1. preserve an already nonzero candidate Z;
       2. compare candidate XY to first bottom, first top, second bottom, second top;
       3. copy the first matching endpoint Z;
       4. invoke collector logic only if no endpoint matches.
     - Before closed horizontal output, fill crossing `Curr` in place with direction-sensitive edge order (horizontal/crossing for left-to-right, crossing/horizontal for right-to-left), then write that filled point.
     - At strictly-simple type-3 top touch, fill one copied candidate with previous/current edge order and use that same complete point for both outputs and the join.
     - Implement the fixed four-value insertion sort and numeric dedup:
       - one unique label copies directly;
       - two or more labels debug-assert exactly two;
       - release still stores the first two sorted labels;
       - emitted Z is `-(table.len() as i64)`.
     - Recover table indices using signed `-z - 1` before conversion to `usize`.
     - Make each Z execution own and return a fresh `Vec<(i64, i64)>`; no borrowed closure, global, thread-local, or retained callback state.
     - Make `Clipper::clear` remove all Z paths, output sidecars, collector/table state, and active execution state.
   - Tests:
     - endpoint-priority branch witnesses for all four endpoints, both direction-sensitive closed-horizontal output fills, and strictly-simple type-3 fill;
     - nonzero callback bypass;
     - same-label self-intersection;
     - sorted two-label table and negative index;
     - debug panic for three/four distinct labels;
     - release first-two behavior for three/four labels;
     - horizontal, top, promotion, output, fixup, duplication, join, and reuse behavior.
   - Acceptance:
     - Focused debug and release Z tests pass.
     - No stale labels/table entries survive reuse.
     - No new public error or validation layer is added.

6. **Retain Z in the existing PolyTree and flatten it in exact preorder**
   - Files:
     - `crates/ares-core/src/geometry/clipper/types.rs`
     - `crates/ares-core/src/geometry/clipper/polytree.rs`
     - New `crates/ares-core/src/geometry/clipper/polytree/z_paths.rs`
   - RED first:
     - Before production changes for this task, add the listed PolyTree tests as compiling assertion failures, run their exact focused filter, and archive command/exit/failure excerpts under `/tmp/task22o28-red-polytree-*`; after implementation archive the matching GREEN command.
   - Changes:
     - Extend `PolyNodeRecord` with an optional Z vector parallel to its existing contour.
     - Ordinary execution stores `None` and preserves current tree/order/allocation behavior.
     - Z execution stores zipped XY/Z without changing contour orientation, start point, closure, ownership, or child order.
     - Flatten the existing tree root-left-to-right in preorder, emitting every nonempty open and closed contour without filtering or canonicalization, matching `PolyTreeToPaths`.
   - Tests:
     - mixed open/closed tree;
     - nested children and sibling order;
     - empty contours omitted only where the existing tree already omits them;
     - exact zipped Z point order;
     - ordinary `into_expolygons` and `into_open_polylines` unchanged.
   - Acceptance:
     - Pinned oracle PolyTree paths match point-for-point and Z-for-Z.
     - No second tree or post-hoc XY provenance reconstruction exists.

7. **Implement expanded/opened source construction and exact split reconciliation**
   - Files:
     - `crates/ares-core/src/geometry/region_expansion.rs`
     - New `crates/ares-core/src/geometry/region_expansion/wave_seeds.rs`
     - New `crates/ares-core/src/geometry/region_expansion/wave_seeds/splits.rs`
   - RED first:
     - Before production behavior, add expanded-path and split tests as compiling assertion failures, run their exact focused filters, and archive command/exit/failure excerpts under `/tmp/task22o28-red-{expanded,splits}-*`; archive matching GREEN reruns after implementation.
   - Changes:
     - Add the exact crate-private signature and function-pointer assertion:
       `wave_seeds(&[ExPolygon], &[ExPolygon], f32, bool, CoordinateScale) -> Result<Vec<WaveSeed>, ClipperError>`.
     - Assert `tiny_expansion > 0.0` before checking either empty input.
     - Return `Ok([])` immediately when either side is empty.
     - Add/range-check boundary contour then holes, one Z ID per `ExPolygon`, before source expansion.
     - Reuse one 2-D `ClipperOffset` configured with `JoinType::Square` and `f64::from(tiny_expansion) * 0.005`.
     - Clear before each contour/hole; apply positive contour and negative hole expansion; preserve all output paths in emitted order.
     - After offsetting, attach the current source Z and append one exact full-XYZ copy of the first point.
     - Increment source Z once per source `ExPolygon`, including zero/multiple-output cases.
     - Add sources as open `Subject`, boundaries as closed `Clip`, and execute `Intersection` with `NonZero/NonZero`.
     - Build split records `(endpoint, -1)` and order them by explicit X/Y/Z comparison using fixed MSVC control flow.
     - Implement exact front-first lower-bound and XY-only final endpoint matching.
     - Implement all four `polylines_merge` direction cases, duplicate-junction retention, last-pop handling, and middle swap-pop/reprocess.
     - Do not use a map, grouped merge, stable erase, post-merge canonicalization, or unconditional index increment.
   - Tests:
     - outer/hole signs and exact shortest-edge threshold;
     - contour/hole shared source ID;
     - zero/multiple offset outputs and per-ExPolygon increments;
     - exact repeated full endpoint and source order;
     - all four merge directions;
     - front-before-back lookup;
     - X/Y/Z lower-bound with XY-only final match;
     - last pop, middle move/reprocess, moved-fragment merge, closed no-op, duplicate junction, and exact final vector order.
   - Acceptance:
     - Expanded paths, split registry order, pre-merge paths, and post-merge paths match the pinned C++ oracle exactly.
     - Boundary-add errors precede all source offset/add errors.

8. **Implement the lazy source-compatible boundary AABB**
   - File:
     - New `crates/ares-core/src/geometry/region_expansion/wave_seeds/aabb.rs`
   - RED first:
     - Before AABB production code, add every listed AABB/containment/laziness witness as a compiling assertion failure, run its focused filter, and archive command/exit/failure excerpts under `/tmp/task22o28-red-aabb-*`; archive the matching GREEN rerun.
   - Changes:
     - Keep all O28 AABB and exact containment code local to this shard; do not modify `expolygon.rs` or introduce a generic/public tree.
     - Build one leaf per boundary `ExPolygon` using only its outer contour bbox.
     - Inflate inclusively by 100 units for `CoordinateScale::Normal` and 10 for `LargeBed`.
     - Allocate `2 * next_power_of_two(n) - 1` implicit nodes.
     - Union internal ranges and choose the longest axis, X on ties.
     - Preserve centroid arithmetic as signed `min + max / 2`, including negative truncation behavior.
     - Port the exact median-of-three/QuickSelect swaps and comparisons without index tie-breaks.
     - Traverse inclusive boxes left-first and stop on the first containing leaf.
     - Implement containment locally with the existing Clipper point-in-polygon predicate:
       - outer result must be nonzero;
       - a hole excludes only positive interior;
       - a hole boundary remains contained.
     - Construct the tree only on the first Branch 1 or Branch 4 fallback request.
   - Tests:
     - both epsilon scales;
     - outer-contour-only bbox despite holes;
     - longest-axis X tie;
     - negative centroid arithmetic;
     - QuickSelect order;
     - overlapping first-hit order;
     - inclusive bbox;
     - outer interior, hole interior rejection, and hole boundary acceptance;
     - proof that direct-ID paths never build the tree.
   - Acceptance:
     - Overlapping-boundary and containment oracle witnesses select the exact pinned ID.
     - No R-tree, hash lookup, eager build, lowest-index scan, or right-first traversal is used.

9. **Implement the four recovery branches and optional seed sorting**
   - Files:
     - `crates/ares-core/src/geometry/region_expansion/wave_seeds.rs`
     - `crates/ares-core/src/geometry/region_expansion/wave_seeds/aabb.rs`
     - `crates/ares-core/src/geometry/clipper/ordering.rs`
   - RED first:
     - Before recovery or sorting production behavior, add every listed debug/release recovery and ordering witness as compiling assertion failures, run both exact focused debug and release filters, and archive command/exit/failure excerpts under `/tmp/task22o28-red-{recovery,sort}-*`; archive matching GREEN reruns.
   - Changes:
     - Process reconciled paths in current vector order.
     - Implement Branch 1:
       - only for open paths with both endpoint Z values nonnegative;
       - scan for first valid source and boundary labels;
       - drop if source missing;
       - emit direct pair if both found;
       - otherwise lazily sample front XY and emit only on success;
       - omit upstream’s unused `iseed`.
     - Implement rare repair:
       - trigger on XY-equal endpoints with `front.z < idx_boundary_end`;
       - replace local front/back for every `z >= idx_boundary_end`;
       - preserve last-match-wins and the absent source upper-bound guard.
     - Implement negative intersection recovery:
       - validate boundary/source pair ranges;
       - try front before back;
       - emit signed-subtracted/narrowed `u32` IDs.
     - Implement closed fallback:
       - retain debug assertions for XY closure/source range/containment;
       - release drops only a failed final containment.
     - Preserve the three deliberate drop sites and add no defensive fallback.
     - For `sorted=true`, sort a `Vec<usize>` permutation with fixed MSVC comparator `(boundary, src)` only, then move seeds accordingly.
     - Widen `fixed_msvc_sort_by` only to `pub(in crate::geometry)` as needed; keep its existing `Copy` implementation and use copyable indices rather than making it sort owned seeds.
     - Add no geometry/index tie-break and do not use host sorting.
   - Tests:
     - both-positive direct IDs;
     - source-only fallback success/failure;
     - missing-source drop;
     - last-source rare repair;
     - front-negative precedence;
     - debug-valid front/back recovery;
     - release invalid-front/valid-back continuation;
     - closed fallback success and release-only failure drop;
     - unchanged XY paths;
     - unsorted output;
     - sorted output;
     - equal-key group over 32 exposing fixed MSVC behavior.
   - Acceptance:
     - Final ordered IDs and XY paths match every pinned oracle vector.
     - `sorted=false` differs only through prescribed swap-pop and drops.
     - `sorted=true` has no stable, host, geometry, or index tie-break behavior.

10. **Prove compatibility with unchanged O27 propagation and lifecycle**
    - Files:
      - Focused test shards only.
      - No project lifecycle production file.
    - RED first:
      - Before adding compatibility test seams or any missing private glue, add the O27 handoff, entry/error-precedence, and lifecycle assertions as compiling failures against the incomplete O28 boundary; capture exact focused RED commands and excerpts under `/tmp/task22o28-red-compat-*`, then archive matching GREEN reruns.
    - Changes:
      - Add a geometry-only test that sends discovered sorted seeds into the existing O27 `propagate_waves`.
      - Assert ordered seed annotations and propagated polygons.
      - Add entry/error-precedence tests:
        - zero, negative, and NaN expansion panic before either empty shortcut;
        - each empty side avoids offsetter, Clipper, and AABB work;
        - boundary range/add error occurs before source expansion/add error.
      - Add KSR configuration/caller attestation only if it remains ignored or non-wired and reads the 3MF through existing test support; do not inspect reference G-code or branch on fixture identity.
      - Run lifecycle/checkpoint assertions proving no new stage consumes `wave_seeds`.
    - Acceptance:
      - O27 ordered/error/rerun tests remain unchanged and green.
      - Project slicing still terminates at `ProjectSlicingIncomplete`.
      - No changes exist in `project_slice*`, CLI, WASM adapter, manifests, lockfile, browser API, or G-code code.

11. **Complete all 23 independent mutation campaigns**
    - Files:
      - No persistent mutation files; scripts/logs live under `/tmp/task22o28-mutations-*`.
    - Changes:
      - Run one compiling mutation at a time, capture the named failing focused test and excerpt, restore production, and rerun GREEN.
      - Required mutations:
        1. ordinary XYZ equality;
        2. endpoint priority removal/reorder;
        3. callback at coincident endpoint;
        4. zero-based intersection index;
        5. unsorted labels;
        6. wrong fixup/copy/overwrite survivor Z;
        7. junction dedup;
        8. moved-slot increment;
        9. grouped split merge;
        10. first rare-repair source;
        11. back-before-front recovery;
        12. eager AABB;
        13. hole-inclusive leaf bbox;
        14. missing/wrong epsilon;
        15. mathematical midpoint;
        16. right-first/lowest-index fallback;
        17. wrong hole offset sign;
        18. per-contour/output source increment;
        19. stable/host seed sort;
        20. geometry/index tie-break;
        21. stale Z after clear;
        22. emptiness before assertion;
        23. source validation before boundary validation.
   - Acceptance:
     - `/tmp/task22o28-mutation-manifest.txt` names every mutation, command, failing test, failure excerpt, restoration, and restored GREEN.
     - All 23 compiling mutations are killed; any survivor blocks review.

12. **Run complete native, release, WASM, browser, static, and rollback verification**
   - Commands:
     ```text
     cargo fmt --all -- --check
     cargo nextest run -p ares-core geometry::tests::clipper::z
     cargo nextest run -p ares-core geometry::tests::region_expansion::wave_seeds
     cargo nextest run --release -p ares-core geometry::tests::clipper::z::release
     cargo nextest run --release -p ares-core geometry::tests::region_expansion::wave_seeds::release
     cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::region_expansion/)'
     cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::clipper/)'
     cargo nextest run --workspace
     cargo nextest run --workspace --no-fail-fast
     cargo check --workspace --all-targets
     cargo clippy --workspace --all-targets -- -D warnings
     cargo clippy --workspace --all-targets --all-features -- -D warnings
     cargo check -p ares-core --target wasm32-unknown-unknown
     cargo check -p ares-core --target wasm32-unknown-unknown --features task22n-browser-oracle
     cargo check -p ares-wasm --target wasm32-unknown-unknown
     cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22n-browser-oracle
     cargo build -p ares-wasm --target wasm32-unknown-unknown --release --target-dir target/wasm-default
     cargo build -p ares-wasm --target wasm32-unknown-unknown --release --features task22n-browser-oracle --target-dir target/wasm-task22n
     git diff --check
     ```
   - Changes:
     - Run `wasm-bindgen` for default and feature builds and repeat the workflow’s export audit.
     - Run the browser package syntax checks and `npm --prefix crates/ares-wasm/tests/browser test` twice.
     - If local Chromium requires an environment wrapper, keep that environment-only repair out of the repository.
     - Archive all outputs under `/tmp/task22o28-*`.
     - Audit:
       - every Rust file `<400` physical lines;
       - each new production/test shard `<=300`;
       - no `include!` or `include_bytes!`;
       - no `unsafe`, FFI, filesystem, native thread, custom/native allocator, TBB, platform branch, mutable global, alternate/legacy geometry fallback, or new dependency; the required lazy AABB and closed recovery branches are not prohibited fallbacks;
       - no manifest/lockfile diff;
       - no source-text/hash/line pinning tests or binary oracle;
       - no reference-G-code reads or fixture identity/name/hash/geometry branches;
       - no lifecycle/checkpoint/incomplete-sink movement;
       - no deferred source-taking propagation or external-surface symbol;
       - only the approved file allowlist changed.
     - Before rollback rehearsal, record the primary worktree's tracked diff, untracked approved-file list, staged state, and content digests. Perform the mechanical rollback only in a disposable copy/worktree populated with that exact O28 state: remove O28 modules/sidecars/APIs/docs, restore internal `Point` records, and verify its tracked/untracked diff against predecessor `f361bb73b558b4e50bfa4fa712afcd63df44ba9f` is empty. Delete the disposable copy, then prove the primary worktree was never mutated by matching its pre-rehearsal diff, file list, staging state, and digests, followed by focused and full GREEN reruns.
   - Acceptance:
     - Every command and audit passes.
     - Linux, macOS, Windows, and WASM use the same pure-Rust implementation.
     - Rollback requires no migration, compatibility shim, persisted-state change, or manifest change.

13. **Run independent code review, update documentation, re-review, commit, push, and verify exact-SHA CI**
   - Files:
     - `docs/architecture/ard-0024-safe-indexed-clipper6-kernel.md`
     - `docs/architecture/option-parity-v4.md`
     - `docs/roadmap.md`
     - O28 spec and plan status/evidence sections.
   - Changes:
     - First obtain literal implementation `VERDICT: APPROVE` from:
       - a fresh independent read-only reviewer;
       - default-model OpenCode.
     - Reviews compare the complete implementation against the approved spec and assess:
       - requirement completeness;
       - logic correctness;
       - boundary cases;
       - code quality;
       - test coverage;
       - actual execution evidence.
     - The parent/implementer is the only fix writer. Convert all findings to a repair list, search for sibling defects, repair, rerun affected and full gates, and resubmit the same revised state to both reviewers until both approve.
     - After preliminary code approval:
       - narrowly amend ARD-0024 to record optional geometry-private Z metadata in the same indexed kernel;
       - add O28 source boundary, behavior, tests, mutations, limitations, rollback, next boundary, and local review/verification status to `docs/roadmap.md`;
       - add the same bounded parity status to `docs/architecture/option-parity-v4.md`;
       - finalize O28 spec/plan local evidence without claiming external-surface, G-code, or not-yet-observed CI parity.
     - Rerun all required verification after documentation changes, then unconditionally rerun both six-dimensional read-only reviewers against that final documented/restored worktree. Repair and repeat both reviews until both outputs end in literal `VERDICT: APPROVE`.
     - Use Conventional Commits, separating implementation and documentation evidence where practical. Never stage `.pi-subagents/`, `target/parity/`, `/tmp` evidence, C++ oracle sources, or generated WASM/browser outputs.
     - Push `main`, verify `HEAD == origin/main`, identify the Tier-1 run whose `headSha` exactly equals the pushed commit, and wait for format, Ubuntu, macOS, Windows, and WASM success. Archive the run ID, URL, exact `headSha`, and job conclusions under `/tmp` and report them as ship evidence; do not create a circular post-CI documentation commit that would require a new exact-SHA run.
   - Acceptance:
     - Both final review gates approve with no blocker.
     - Intended documentation is committed.
     - The exact pushed SHA has a completely successful Tier-1 matrix.
     - O28 is not declared shipped while CI is pending or attached to another SHA.

## Files to Modify

- `crates/ares-core/src/geometry/clipper.rs` — declare private Z support and retain collector state in the one existing kernel.
- `crates/ares-core/src/geometry/clipper/types.rs` — migrate internal edge/intersection/output/join points to `KernelPoint`; add optional PolyTree Z sidecar.
- `crates/ares-core/src/geometry/clipper/engine.rs` — initialize/reset Z execution state and preserve ordinary execution behavior.
- `crates/ares-core/src/geometry/clipper/predicates.rs` — keep all geometry predicates explicitly XY-only.
- `crates/ares-core/src/geometry/clipper/input.rs` — add narrow Z adapters; existing adapters assign zero Z.
- `crates/ares-core/src/geometry/clipper/input/path.rs` — normalize/range-check `KernelPoint` using XY-only semantics.
- `crates/ares-core/src/geometry/clipper/input/bounds.rs` — preserve Z through horizontal endpoint reversal.
- `crates/ares-core/src/geometry/clipper/intersections.rs` — compute candidate metadata, call exact `SetZ`, preserve scanbeam Z.
- `crates/ares-core/src/geometry/clipper/intersections/open.rs` — accept the metadata-bearing candidate without changing open clipping logic.
- `crates/ares-core/src/geometry/clipper/intersections/top.rs` — implement exact top/bottom/zero current-Z updates.
- `crates/ares-core/src/geometry/clipper/horizontals.rs` — preserve Z and apply only the pinned direction-sensitive closed-output SetZ fill at horizontal crossings.
- `crates/ares-core/src/geometry/clipper/minima.rs` — apply only mechanical KernelPoint predicate/join-offset conversions.
- `crates/ares-core/src/geometry/clipper/strictly_simple.rs` — apply mechanical KernelPoint conversions plus only the pinned type-3 SetZ fill before its two output writes/join.
- `crates/ares-core/src/geometry/clipper/output/rings.rs` — allocate, deduplicate, duplicate, and expose complete XYZ output points.
- `crates/ares-core/src/geometry/clipper/output/fixup.rs` — preserve exact survivor identity while projecting ordinary paths to XY.
- `crates/ares-core/src/geometry/clipper/output/append.rs` — carry full intersection metadata into output records.
- `crates/ares-core/src/geometry/clipper/output/join_points.rs` — preserve complete XYZ through exact join copies and replacements.
- `crates/ares-core/src/geometry/clipper/output/ownership.rs` — project to XY for ownership predicates.
- `crates/ares-core/src/geometry/clipper/polytree.rs` — materialize optional Z sidecars without changing ordinary tree behavior.
- `crates/ares-core/src/geometry/clipper/ordering.rs` — widen fixed-sort visibility only; retain `Copy` by sorting indices.
- `crates/ares-core/src/geometry/region_expansion.rs` — declare/re-export within the module and freeze the exact private function signature.
- `crates/ares-core/src/geometry/tests/clipper.rs` — declare Z tests.
- `crates/ares-core/src/geometry/tests/region_expansion.rs` — declare `wave_seeds` tests.
- `docs/architecture/ard-0024-safe-indexed-clipper6-kernel.md` — record the same-kernel private metadata extension.
- `docs/architecture/option-parity-v4.md` — record bounded O28 parity and deferrals.
- `docs/roadmap.md` — record O28 scope, evidence, rollback, and next boundary.
- `docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o28-clipperz-wave-seeds.md` — finalize approval/implementation/ship evidence.
- `docs/superpowers/plans/2026-08-07-ksr-fdmtest-v4-task22o28-clipperz-wave-seeds.md` — record reviewed execution evidence.

## New Files

- `crates/ares-core/src/geometry/clipper/z.rs` — geometry-private `KernelPoint`, Z paths, exact `SetZ`, and collector logic; at most 300 LOC.
- `crates/ares-core/src/geometry/clipper/polytree/z_paths.rs` — exact preorder Z-path flattening; at most 300 LOC.
- `crates/ares-core/src/geometry/region_expansion/wave_seeds.rs` — bounded `Algorithm::wave_seeds` orchestration and recovery; at most 300 LOC.
- `crates/ares-core/src/geometry/region_expansion/wave_seeds/splits.rs` — split registry, four-direction merge, and swap-pop reconciliation; at most 300 LOC.
- `crates/ares-core/src/geometry/region_expansion/wave_seeds/aabb.rs` — lazy source-compatible AABB and local ExPolygon containment; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/clipper/z.rs` — Z test module root; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/clipper/z/input_fill.rs` — Z input, equality, `SetZ`, collector, debug/release tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/clipper/z/output.rs` — output/fixup/duplication/join/PolyTree tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/clipper/z/lifecycle.rs` — clear/reuse and unchanged 2-D tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds.rs` — wave-seed test root; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds/expanded.rs` — offset signs, IDs, endpoint duplication, and order tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds/splits.rs` — exact merge/swap-pop tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds/recovery.rs` — four recovery branches and release-only tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds/aabb_order.rs` — AABB build, traversal, containment, and sorting tests; at most 300 LOC.
- `crates/ares-core/src/geometry/tests/region_expansion/wave_seeds/oracle.rs` — checked-in Rust vectors derived from human-reviewed oracle output, without C++ source/blob/hash pinning; at most 300 LOC.

## Explicitly Unchanged

- `Cargo.toml`, `Cargo.lock`, and every crate manifest.
- `crates/ares-core/src/lib.rs`.
- `crates/ares-core/src/geometry.rs`.
- `crates/ares-core/src/geometry/expolygon.rs`.
- All `project_slice*` production files and lifecycle/checkpoint/incomplete-sink code.
- `crates/ares-cli/**`.
- `crates/ares-wasm/src/**` and browser exports.
- Existing O27 propagation and offset metadata behavior.
- Public 2-D `Point`, `Polygon`, and `Polyline` semantics.
- Reference 3MF/G-code fixtures.

## Dependencies

- Task 1 blocks every production or test implementation edit; the spec/plan review documents themselves are the only pre-gate repository edits.
- Task 2 must finish before oracle-derived expected vectors are frozen.
- Task 3 establishes compiling REDs before Tasks 4–6 alter the kernel.
- Tasks 4–6 are sequential because they share the indexed kernel and must preserve a compilable migration boundary.
- Task 7 depends on the Z kernel and PolyTree flattening.
- Task 8 depends on only existing 2-D geometry and may be implemented after Task 7’s module structure is established.
- Task 9 depends on Tasks 7 and 8.
- Task 10 depends on the complete `wave_seeds` result and unchanged O27 propagation.
- Task 11 begins only after all focused GREEN tests pass.
- Task 12 begins after every mutation is restored and killed.
- Task 13 begins after complete verification; documentation, final reviews, commit/push, and exact-SHA CI are strictly ordered.
- Fresh implementation subagents should execute Tasks 3–10 sequentially with explicit file ownership. Do not run concurrent writers against shared Clipper files.

## Risks

- **Kernel-wide type migration:** A `Point`→`KernelPoint` internal change may reveal an unlisted compile touchpoint. The implementation must stop for spec amendment and dual re-review rather than silently edit it.
- **LOC pressure:** `ordering.rs` and `output/join_points.rs` are already near 400 lines. Keep changes line-neutral; any new shard not already approved requires a spec amendment.
- **Accidental Z-sensitive geometry:** Derived equality/order or implicit conversions could make Z affect predicates. Require explicit XY projections and mutation coverage.
- **Release divergence:** Three/four-label collector behavior and invalid-front recovery differ under debug assertions. Both release-focused nextest commands are mandatory.
- **Borrow/state design:** A callback closure borrowing the Clipper or global state would violate the spec. Collector/table ownership must remain inside one execution and be taken/reset afterward.
- **Ordering parity:** Host sort, stable sort, extra tie-breaks, canonicalization, or changed swap-pop iteration will alter output. Oracle and over-32 equal-key tests must compare full order.
- **AABB parity:** The upstream centroid expression intentionally differs from a mathematical midpoint; overlapping boundaries expose build/traversal order.
- **False lifecycle progress:** The KSR caller reaches this upstream path, but O28 must not wire it into Ares project slicing or claim G-code parity.
- **Insufficient evidence:** Geometry counts, area, or bounds are not acceptance evidence. Tests and oracle captures must compare complete ordered IDs and point sequences.
- **External CI:** The pushed commit's exact `headSha` required a fully successful Tier-1 matrix; run `31156094839` satisfied this ship gate.

## Execution evidence

- The implementation uses only the approved same-kernel production boundary;
  public adapters, manifests, project lifecycle, fixtures, and O27 behavior are
  unchanged.
- Focused nextest results are 25/25 Z, 39/39 wave seeds, 211/211 Clipper, and
  53/53 RegionExpansion. Both final-state workspace commands pass 5,994 tests
  with 2 skipped. Release filters pass 1/1 Z and 3/3 recovery tests.
- Workspace all-target check, strict all-feature Clippy, formatting, four
  wasm32 checks, two optimized WASM builds, export/syntax audit, and two 11/11
  Playwright runs exit zero.
- Pinned debug and `NDEBUG` C++ diagnostics record full ordered paths and IDs
  for inside, crossing, hole, split, multiple-ID, overlap, and release-only
  shared-vertex cases. The fixed over-32 comparator literals reuse unchanged
  accepted ARD-0024 MSVC STL 14.44 control flow; local proprietary MSVC
  execution is unavailable, and exact-SHA Windows Tier-1 job `92795799169`
  supplied the required platform evidence in run `31156094839`.
- All 23 required mutations plus a strict-shortest-edge mutation are killed,
  production is restored, and each restored witness reruns GREEN. Original RED
  chronology is unavailable and is disclosed rather than recreated.
- The disposable rollback rehearsal copied the exact O28 state, returned clean
  to predecessor `f361bb73b558b4e50bfa4fa712afcd63df44ba9f`, and preserved
  the primary diff, file list, staging state, and content digests.
- Final documented-state independent six-dimensional and default-model
  OpenCode reviewers both returned literal `VERDICT: APPROVE` after the repair
  loop. Implementation commit `7eb0d27` and documentation commit `be33437`
  were pushed; exact-SHA Tier-1 run `31156094839` passed at
  `be334375be871eb12ca98c98d889b65a92d13a37`.