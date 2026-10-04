# Lint remediation: behavior-preserving module slices

## Non-negotiable boundaries

- Preserve existing runtime behavior, comparator thresholds, mutation rejection and every test assertion. This is lint remediation, not a new slicing milestone or upstream-behavior change.
- Batch subsequent work by owning module, targeting 60–120 diagnostics per slice rather than repeating full gates and review for a handful of warnings.
- Fix excessive nesting and argument lists at their owning module boundaries; do not raise lint thresholds or blanket-suppress warnings.
- Work in an isolated worktree. Keep concurrent support-tip, Arachne-outline and tower/AdaptivePA/bridge work out of the current slice.
- For each slice: run full workspace nextest, clippy, fmt and WASM compilation; push the topic branch, print `REVIEW-REQUEST <commit-hash> <summary>` in the terminal, and stop for coordinator Claude's `APPROVED <hash>` or revision requests. Do not invoke external reviewers or start new changes on the same branch while waiting. Only after approval: fetch/rebase latest `origin/main`, rerun the gates, and fast-forward main without force pushing.
- The final workspace gate is `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Intermediate module slices report remaining baseline warnings explicitly rather than claiming the workspace is warning-free.

## First slice: CLI semantic comparison

Destination: `crates/ares-cli/tests/ksr_fdmtest_v4/semantic/{controls,parser,depositions,lifecycle,travel}.rs` and `semantic/depositions/leftovers/geometry.rs`.

ARD-0081 owns the boundary: semantic comparison remains a diagnostic utility, never a fallback for strict ordered-byte acceptance. Preserve existing control canonicalization, fan parsing/error order, geometric sampling arithmetic, matching predicates, and diagnostic output. Extract existing nested control/diagnostic operations or use equivalent flatter branches. Do not add probes, tolerances, fixtures, or assertions.

Verification uses the existing semantic mutation regressions before and after the refactor, then full workspace gates and WASM compilation. The prior full baseline is commit `9a7b46fc`: 7,589 tests passed, nine skipped; existing lint debt is tracked separately from this slice's changed files.

First-slice results: 144 semantic regression cases passed before editing; after editing the full workspace passed all 7,589 cases (nine skipped). `cargo clippy -p ares-cli --test ksr_fdmtest_v4 --no-deps -- -D warnings` passed, eliminating the ten semantic nesting diagnostics. Full ordinary clippy, fmt and the `wasm32-unknown-unknown` build for both core and the WASM adapter passed. Remaining core/parity warnings are not suppressed and remain follow-up work.

## Scalar wire adapters

Destination: `ares-core/src/options/config_types/{scalar,scalar_bool_int}.rs`, the existing Rust serde wire adapters around `Config.hpp` scalar option concepts (`ConfigOptionFloat`, `ConfigOptionPercent`, `ConfigOptionFloatOrPercent`, `ConfigOptionBool`). No upstream behavior is added or deferred by this lint-only slice. A shared generic `drain_sequence` retains the same `IgnoredAny` reads after the first typed array element. Empty-array/first-element errors still occur first, malformed trailing input still propagates an error, and successful scalar values and integer-macro visitors are unchanged. Existing deserialization and full-workspace tests remain intact.

Scalar-slice validation: full workspace nextest passed 7,589 tests (nine skipped); ordinary workspace/all-targets/all-features clippy, fmt and the core/WASM target build passed. The four scalar diagnostics disappeared: core library warnings dropped from 253 to 249, and core test warnings from 269 to 265 including duplicates. Remaining workspace warnings are explicit follow-up work, not suppressed. Publication follows the coordinator approval protocol above.

## Geometry batch

Baseline: `0d6dfe1f`, the approved scalar slice rebased onto `7ac55555` (thin-tip repair); full workspace nextest, ordinary clippy, fmt and WASM compilation passed before geometry edits. This batch covers all 43 geometry diagnostic locations: 36 excessive-nesting and seven argument-count warnings. Of these, 28 are in the production library and 15 in the existing test-only `edge_grid::simplify` module; its `cfg(test)` boundary is unchanged.

| Upstream ownership | Rust destination | Preserved operations |
| --- | --- | --- |
| `Support/SupportMaterial.cpp::{rasterize_polygons,contours_simplified,dilate_trimming_region,seed_fill_block}`, AGG `agg_rasterizer_cells_aa.h::{render_hline,line}` and `agg_rasterizer_scanline_aa.h::sweep_scanline` | `geometry/agg_raster/{cells,contours,grid_ops,mod}.rs`; signature-only caller edits in `geometry/support_grid.rs` | Signed remainder stepping; area/cover arithmetic; duplicate-x accumulation; conditional x increment; row/block/directional sweep order; boundary insertion and contour successor selection; float scaling and corner shrink order. |
| `EdgeGrid.cpp::{calculate_sdf,contours_simplified}` (`:600–900`, `:1283–1400`), `EdgeGrid.hpp::cell_inside_or_crossing` | `geometry/edge_grid/simplify{.rs,/field.rs}` | Row-major seeds; exact wedge/interior predicates and cast sites; frozen signs; Danielsson propagation; pre-fill snapshot; contour closure-before-successor selection; integer scaling. |
| `MarchingSquares.hpp` start-cell search and ring scanning | `geometry/marching_squares{.rs,/rings.rs}` | Word/bit scan order, loop bounds and residual ambiguous direction bits. |
| `AABBTreeLines.hpp::{point_outside_closed_contours,distance_from_lines_extra}`, `Line.hpp` segment distance | `geometry/line_distance_tree{.rs,/outside.rs,/query/mm.rs}` | Query point/scale pairing only; recursive child order, strict ties/pruning and floating arithmetic unchanged. |

Included work is limited to extracting existing algorithm stages, expressing coordinate/dimension pairs and borrowed seed/query state, and equivalent integer guards. Two existing local argument-count suppressions disappear because their signatures now meet the threshold. No new suppression, allocation, assertion change, DEBUG probe or algorithm is introduced. Adjacent upstream behavior remains deferred exactly as before; existing adapters are not expanded into an independent slicing pipeline. Functional Arachne, tower/AdaptivePA/bridge and FillBase-connect code stays outside this batch.

Validation retains every existing test and comparator. Full workspace gates cover the test-only EdgeGrid implementation; production-only clippy is insufficient to verify its 15 diagnostics. Existing gyroid/TPMSFK, support-grid, signed-distance and tree-query fixtures provide regression coverage, not a claim of exhaustive new upstream parity.

Geometry results: all 7,589 workspace tests passed (nine unchanged skips); ordinary all-target/all-feature clippy, fmt and core/WASM compilation passed. Geometry diagnostics fell from 43 to zero in the full workspace log. Core library warnings fell from 249 to 221; core test warnings from 265 to 222 (221 duplicates); CLI parity retains 20 warnings. All changed Rust files remain below 400 lines. No test or comparator source changed. Reproducible gate logs for this run are `/tmp/ares-lint-geometry-{nextest,clippy,wasm}.log`; main integration remains governed by coordinator approval and rebase-time revalidation.

## G-code post-processing and adjacent geometry batch

Baseline: `b64b1d90`. Remaining diagnostics are counted once per primary file/line/column across all workspace targets: 243 locations, led by `gcode_emit` (86), `fill` (73), and adjacent `pressure_equalizer` (12). This batch targets 64 locations: FanMover26, cooling9, pressure equalizer12, brim3, processor3, object ordering5 and curl6.

Upstream boundaries are `GCode/FanMover.cpp` (buffering, delayed fans and kickstart), `GCode/CoolingBuffer.cpp` (fan/comment/feedrate rewriting), `GCode/PressureEqualizer.{cpp,hpp}` (parsing, limiting and split emission), `Brim.cpp`, `GCode/GCodeProcessor.cpp` (arc accounting), `ShortestPath.cpp` (instance chaining), and `SupportSpotsGenerator.cpp` / `ExtrusionProcessor.hpp` (curl estimates). Destinations stay inside their existing `project_slice/gcode_emit` and `project_slice/pressure_equalizer` modules. Included changes only extract existing stages or group source-owned arguments; all adjacent upstream behavior remains deferred as before, and no scaffold is expanded into a new Ares algorithm. Timelapse, travel lifecycle, AdaptivePA dispatch, Arachne and bridge-direction logic are excluded.

Preserve numerical expression/cast order, invalid-command partial state, exact comments and newlines, duplicate fan suppression, forward/backward phase distinctions, contour tie choices and every existing assertion. The pre-edit full workspace and official/seeded control receipts are the completed geometry integration receipts above. Tests and strict-output controls must be rerun after refactoring; no warning suppression or comparison tolerance change is permitted.

G-code batch results against `b64b1d90`: all 64 owned diagnostics are gone, leaving **179 unique primary diagnostic locations** across workspace targets (243 before). Ordinary clippy reports core library 157, core tests 158 including 157 duplicates, and CLI parity 21. Full workspace nextest passed 7,594 tests with nine unchanged skips; fmt and core/WASM compilation passed. Every changed Rust file remains below 400 lines. The only test edits group the same endpoint arguments at two curl call sites; all assertions and comparator sources remain unchanged.

Complete CPU0 controls retain exactly the baseline status sets: official 388 PASS / eight existing replay-environment FAIL (`ORCA_REPLAY_*` absent), seeded 396/396 PASS. The eight official failures are not semantic passes. Actual Ares G-code SHA-256 multisets also match the baseline for all artifact labels: 383 labels with output in official mode and 394 in seeded mode. Gate logs, source hashes, complete-output artifacts, `lint-counts.json`, `comparison.json`, and `actual-hash-comparison.json` are in `/tmp/ares-lint-gcode-controls.6tcgfM/`. These are pre-review receipts; approval, latest-main rebase and revalidation remain required before integration.
