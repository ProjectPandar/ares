# Task 22O.20 — Single-region vertical-shell projection gather Plan

Spec: `docs/superpowers/specs/2026-08-04-ksr-fdmtest-v4-task22o20-vertical-shell-projection.md`

## Status

Implemented and validated from Ares baseline `059d26db8b91d6867ffdb3b2045469fe0caa8459` against pinned Orca `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen evidence: parent-bound O20 checksum `-106767561006193260948265111057697183253`, totals `[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, event totals `[1830, 917, 1539, 749, 0, 0, 0, 0]`, 45 focused tests, 355 O10-O20 regressions, and 5,678 workspace passes with 2 skipped. Strict Clippy, native all-target, both WASM, formatting, diff, LOC, forbidden-pattern, dependency, source-pinning, and staging gates pass. The final independent six-dimensional and OpenCode rereviews both approve the identical implementation diff; post-push Tier-1 evidence remains the last release gate.

## Validation contract

Port only the release-observable projection gather in `PrintObject::discover_vertical_shells` at `PrintObject.cpp:2153-2278`, using the directly reached Layer/Flow/ExPolygon/Clipper sources listed in the spec. Stop before debug/no-op lines `2279-2333` and internal-surface trimming at `2334`. O19 and O20 are crate-private temporary compatibility representations of the upstream cache and local `shell`/`holes`, consumed by the next source slice; preserve O19 exactly and do not grow an Ares-owned pipeline.

## Gate 0

1. Independently and through OpenCode review the source ledger and behavioral boundary in the spec; any repair returns to both.
2. Review this plan with both reviewers against the approved spec and current O19 seam; any repair returns to both.
3. Before RED run `test "$(git rev-parse HEAD)" = 059d26db8b91d6867ffdb3b2045469fe0caa8459`, `test "$(git rev-parse origin/main)" = 059d26db8b91d6867ffdb3b2045469fe0caa8459`, and `test "$(git -C OrcaSlicer rev-parse HEAD)" = 8500fcdccaa10b5099ac20d252af3a7c560046f1`. Add no Rust until both plan reviewers approve.

## Task 1 — RED projection and successor contract

Exact files/budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shell_projection.rs` (module root/successor, at most 180 LOC);
- new children `vertical_shell_projection/types.rs`, `stage.rs`, `gather.rs`, `cleanup.rs`, each at most 300 LOC;
- direct test root `vertical_shell_projection/tests.rs` and shards `combine.rs`, `windows.rs`, `anchors.rs`, `transaction.rs`, each at most 300 LOC;
- add the module declaration to `prepare_infill.rs`;
- new `geometry/clipper/boolean_paths.rs` for source-shaped NonZero Paths union/intersection, with re-export-only edits in `geometry/clipper.rs` and `geometry.rs`; direct geometry tests live in a real `geometry/tests/clipper/boolean_paths.rs` module declared by an explicit `mod boolean_paths;` edit in `geometry/tests/clipper.rs`, each file below 300 LOC;
- integration root `project_slice/tests/prepare_infill/vertical_shell_projection.rs` and shards `fixture.rs`, `ksr.rs`, `options.rs`, `ownership.rs`, `lifecycle.rs`, `cleanup.rs`, and `metamorphic.rs`, each at most 300 LOC; declare from `tests/prepare_infill.rs`;
- `project_slice.rs` wiring only; reuse iterative O19 disposal without growing `incomplete_sink.rs` beyond 399 LOC;
- documentation paths: this spec/plan, `docs/architecture/option-parity-v4.md`, and `docs/roadmap.md`.

RED before production behavior:

1. Add a compiling API shell, then direct combine tests for all source lambda branches: holes empty-left/empty-right/intersection; shell empty-source/empty-next/append-then-NonZero-union. Freeze exact holed, overlapping, repeated, and disjoint Paths order.
2. Add direct top and bottom window tables. Pin integer-count boundaries, thickness-only continuation, equality at `thickness - 1e-4` as excluded, one representable step below as included, asymmetric `print_z` versus `bottom_z = print_z - height`, top-before-bottom event order, first/last layer behavior, and zero/negative counts with neither scan nor anchor. Prove current `None` is an explicitly deferred dead transient at the next trim boundary. Prove an interior neighboring `None` is still visited, clears holes, contributes no shell, continues the window, and suppresses the anchor.
3. Before production behavior, run `cargo nextest run -p ares-core --no-fail-fast task22g_raw_wrapper` to guard existing `geometry::tests::clipper::offset::execute::{task22g_raw_wrapper_runs_internal_cleanup_and_preserves_round_order,task22g_raw_wrapper_negates_cw_delta_then_reverses_each_result}` evidence for shortest-edge/CCW Positive and CW Negative cleanup/outer-removal/reversal. Add an O20 anchor RED using a current-cache contour plus CW hole that freezes exact raw per-path and final NonZero-union order, empty offset input, stopped-index object-lslice contour-then-hole order, current-index spacing and implicit `coord_t -> f32` bits, miter `3.0`, offset-before-intersection order, and no hole combination. Add both top/bottom count-`1`, thickness-`0` anchors. A first-layer current record and later stopped neighbor must carry distinguishable spacing values. The reused offset characterization may already pass, but the containing O20 anchor group must RED on missing O20 behavior. Inactive ensure modes emit empty projections and no events.
4. Add every geometry failure site and exact error `vertical-shell projection geometry is outside the supported Clipper range`. Whole-project tests require validation before events, source object→slot order, no partial successor, and stage-before-move.
5. Add ownership/alignment tests for object/record/cache/input/prelude/plan/lslice counts, `Some`/`None`, source identity, region identity, and planned index. Snapshot the outer O19 predecessor/object/cache/record vector buffers, every nested expolygon/polygon point buffer, and source tree allocations before/after success; projection paths are fresh and nonaliasing against both O19 caches and stopped-index object lslices.
6. Add zero-O20-invocation precedence with exact unchanged errors for spiral, counterbore, multi-region, interface shells, active extra bridge, O17 geometry, and separate O19 Top-offset and Bottom-offset failures. Add 64-KiB/depth-10,000 two-tree/drop-probe witnesses separately for each O20 failure, direct success disposal, and public incomplete consumption.
7. Add exact independent real-archive replacements before GREEN in `Metadata/project_settings.config`: `"top_shell_layers": "5" -> "1"` together with `"top_shell_thickness": "1" -> "0"` for the top-anchor case; `"bottom_shell_layers": "3" -> "1"` while bottom thickness remains `0` for the bottom-anchor case; separate `"bottom_shell_thickness": "0" -> "1"` for the thickness window; and `"outer_wall_line_width": "0.42" -> "0.52"` with changed aligned external spacing/projection geometry. For model-part precedence after exactly `<part id="1" subtype="normal_part">`, independently insert: both `<metadata key="top_shell_layers" value="1"/>` and `<metadata key="top_shell_thickness" value="0"/>` for the top anchor; `<metadata key="bottom_shell_layers" value="1"/>` for the bottom anchor; `<metadata key="bottom_shell_thickness" value="1"/>` for its window; or `<metadata key="outer_wall_line_width" value="0.52"/>` for spacing, while global values remain unchanged. Also require reverse-entry Stored/Unix repack; `ksr_fdmtest_v4.drc -> task22o20_renamed`; exact component X transform `1 0 0 0 1 0 0 0 1 0 0 0 -> 2 0 0 0 1 0 0 0 1 0 0 0`.
8. Add independent-twice KSR characterization guarded by O19 successor checksum `148296943860974241781127169756103364063` and O19 totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`. Define an O20 parent marker and delimit objects, slots/`None`, shell, holes, paths, points, and coordinates. Record parent-capture RED before freezing literals.
9. After a compiling shell, record distinct RED with `cargo nextest run -p ares-core --no-fail-fast` followed by each exact filter: `geometry::tests::clipper::boolean_paths`; `project_slice::prepare_infill::vertical_shell_projection::tests::combine`, `::windows`, `::anchors`, `::transaction`; and `project_slice::tests::prepare_infill::vertical_shell_projection::options`, `::ownership`, `::lifecycle`, `::cleanup`, `::metamorphic`, `::ksr`. Expected RED is missing O20 behavior or parent checksum—not compile errors or another group's failure. Preserve RED logs outside Git, then rerun every identical command/filter GREEN.

## Task 2 — GREEN source-shaped projection and transaction

1. Add `VerticalShellProjection { shell, holes }`, aligned projection-object sidecars, and a successor owning the exact O19 predecessor/objects/caches unchanged.
2. Add source-shaped `union_polygons_paths` and `intersection_polygons_paths` using `Clipper::execute_paths`, `ClipOperation::{Union,Intersection}`, `FillRule::NonZero` for subject and clip, Paths output, original insertion order, and no PolyTree/sort/canonicalization. Retain the existing source-shaped anchor offset pipeline: per-path `ShortestEdgeLength = abs(delta * 0.005)`; CCW `+delta` with Positive cleanup; CW `-delta` with temporary outer polygon/reverse solution/Negative cleanup/outer removal followed by reversing results back; then NonZero Paths union. Implement against the Task 1 contour-plus-CW-hole, intermediate/final-order, shortest-edge, and empty-input RED evidence; do not defer those tests to GREEN.
3. Validate the complete project alignment before geometry. Read region options, planned z/height, O19 caches, external spacing, and object lslices only through aligned predecessor records. Preserve current O19 synthetic `None` as projection `None` under the spec's dead-transient proof; treat neighboring `None` as a planned, visited source empty cache that clears holes and suppresses anchors without terminating windows.
4. Implement holes/shell combination exactly. Use a stable event hook only under tests. Map all Clipper errors once to the exact O20 geometry error.
5. Implement top forward and bottom backward scans with source operand/cast order and strict `1e-4` comparisons. Use f64 planned z values and derive bottom z by subtraction at each comparison.
6. Implement anchor fallback only inside the respective positive layer-count block, when no planned neighbor was visited and the stopped index exists. Expand current-index cache paths using current-index `external_spacing as f32`; flatten exact post-compensation object lslices at the stopped index; call existing positive `offset_paths` with `JoinType::Miter`, `3.0`, then source-shaped Paths intersection and shell combine.
7. For non-`EnsureAll`, stage empty shell/holes without geometry. Do not compute regularization-only `solid_infill_spacing * 1.05f`.
8. Stage all projections while borrowing O19. Only after success move O19 fields verbatim beside projections. On failure dispose O19 iteratively. Add direct successor disposal, wire public slicing through O20 once, and remain `ProjectSlicingIncomplete`.
9. Run every RED GREEN, then focused O19 and O18 regressions.

## Task 3 — Freeze KSR and active provenance

1. Freeze literal baseline projection counts/checksums and the exact number/order of top/bottom visits, hole intersections, shell unions, and anchor calls.
2. Freeze exact results for count/thickness mutations, top-only and bottom-only anchor activation, external-spacing width change, and model-part precedence. Compare every unrelated O19 allocation and parent checksum before/after O20.
3. Freeze reverse Stored/Unix ZIP and non-slicing rename invariance. For the exact object `0` and chosen populated slot/path indices, freeze component-scale source/projection spans while option-derived visit windows remain unchanged.
4. Parse KSR independently twice, remove parent-capture diagnostics, and freeze the O20 full-successor checksum/totals only after clean implementation output.

## Task 4 — Full gates, review, docs, ship

1. Update architecture, roadmap, spec, and plan with boundary, exact windows/order/geometry provenance, ownership, KSR checksums/totals/counts, and next source line `PrintObject.cpp:2334`.
2. Run `cargo nextest run -p ares-core --no-fail-fast vertical_shell_projection`, `cargo nextest run -p ares-core --no-fail-fast -E 'test(/classic|layer_region|surface_type_detection|fill_surfaces|vertical_shells|vertical_shell_projection/)'`, `cargo nextest run --workspace --no-fail-fast`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo check -p ares-wasm --target wasm32-unknown-unknown`, `cargo fmt --all -- --check`, and `git diff --check`.
3. Run the exact LOC audit `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 >= 400 {print; bad=1} END {exit bad}'`. Require both added-Rust audits to produce no matches: `git diff --unified=0 -- '*.rs' | rg '^\+[^+].*(unsafe|include!|include_bytes!|allow\()'` and `git diff --unified=0 -- '*.rs' | rg '^\+[^+].*(ksr_fdmtest_v4.*(hash|layer|geometry)|reference.*gcode|OrcaSlicer.*(Command|FFI))'`. Require `git diff -- Cargo.toml Cargo.lock` empty. Require the addition/deletion source-pinning audit `git diff --unified=0 -- '*test*.rs' | rg '^[+-][^+-].*(OrcaSlicer/src|8500fcdc|source.*line|source.*hash)'` empty. Require the staged-artifact audit `git diff --cached --name-only | rg '^(\.pi-subagents/|target/parity/)'` empty before each commit.
4. Record no public API/persistence/dependency/migration/compatibility layer; rollback removes only O20, its Paths helper, and restores the O19 terminal.
5. Run independent six-dimensional and OpenCode implementation reviews against the identical diff/evidence. Main thread applies findings, reruns affected plus full gates, and returns the updated identical diff to both until approval.
6. Synchronize final docs, create small Conventional Commits, push main, verify clean `HEAD == origin/main`, then require the O20 commit's `.github/workflows/tier1.yml:18-30,41-80` Windows/macOS/Linux matrix and complete browser-WASM job (generated-export audit plus Playwright included) to pass.

## Stop condition

Stop O20 only when projected shell/hole sidecars are transactionally derived from typed aligned O19 inputs with every window/order/cast/anchor/ownership invariant proven, public slicing reaches O20 once and remains incomplete before internal trimming, all gates pass, both final reviewers approve, docs are synchronized, and commits are pushed.
