# Lint remediation: behavior-preserving module slices

## Non-negotiable boundaries

- Preserve existing runtime behavior, comparator thresholds, mutation rejection and every test assertion. This is lint remediation, not a new slicing milestone or upstream-behavior change.
- Batch by owning module. Prefer small fill slices.
- Fix excessive nesting and argument lists at their owning module boundaries; do not raise lint thresholds or blanket-suppress warnings.
- Keep concurrent support-tip, Arachne-outline and tower/AdaptivePA/bridge work out of the current slice.
- For each slice: run full workspace nextest, clippy, fmt and WASM compilation.
- The final workspace gate is `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Intermediate module slices report remaining baseline warnings explicitly rather than claiming the workspace is warning-free.

## First slice: CLI semantic comparison

Destination: `crates/ares-cli/tests/stock_project_whole_file/semantic/{controls,parser,depositions,lifecycle,travel}.rs` and `semantic/depositions/leftovers/geometry.rs`.

ARD-0081 owns the boundary: semantic comparison remains a diagnostic utility, never a fallback for strict ordered-byte acceptance. Preserve existing control canonicalization, fan parsing/error order, geometric sampling arithmetic, matching predicates, and diagnostic output. Extract existing nested control/diagnostic operations or use equivalent flatter branches. Do not add probes, tolerances, fixtures, or assertions.

Verification uses the existing semantic mutation regressions before and after the refactor, then full workspace gates and WASM compilation. Existing lint debt is tracked separately from this slice's changed files.

First-slice results: 144 semantic regression cases passed before editing; after editing the full workspace passed all 7,589 cases (nine skipped). `cargo clippy -p ares-cli --test ksr_fdmtest_v4 --no-deps -- -D warnings` passed, eliminating the ten semantic nesting diagnostics. Full ordinary clippy, fmt and the `wasm32-unknown-unknown` build for both core and the WASM adapter passed. Remaining core/parity warnings are not suppressed and remain follow-up work.

## Scalar wire adapters

Destination: `ares-core/src/options/config_types/{scalar,scalar_bool_int}.rs`, the existing Rust serde wire adapters around `Config.hpp` scalar option concepts (`ConfigOptionFloat`, `ConfigOptionPercent`, `ConfigOptionFloatOrPercent`, `ConfigOptionBool`). No upstream behavior is added or deferred by this lint-only slice. A shared generic `drain_sequence` retains the same `IgnoredAny` reads after the first typed array element. Empty-array/first-element errors still occur first, malformed trailing input still propagates an error, and successful scalar values and integer-macro visitors are unchanged. Existing deserialization and full-workspace tests remain intact.

Scalar-slice validation: full workspace nextest passed 7,589 tests (nine skipped); ordinary workspace/all-targets/all-features clippy, fmt and the core/WASM target build passed. The four scalar diagnostics disappeared: core library warnings dropped from 253 to 249, and core test warnings from 269 to 265 including duplicates. Remaining workspace warnings are explicit follow-up work, not suppressed.

## Geometry slice

This slice covers all 43 geometry diagnostic locations: 36 excessive-nesting and seven argument-count warnings. Of these, 28 are in the production library and 15 in the existing test-only `edge_grid::simplify` module; its `cfg(test)` boundary is unchanged.

| Upstream ownership | Rust destination | Preserved operations |
| --- | --- | --- |
| `Support/SupportMaterial.cpp::{rasterize_polygons,contours_simplified,dilate_trimming_region,seed_fill_block}`, AGG `agg_rasterizer_cells_aa.h::{render_hline,line}` and `agg_rasterizer_scanline_aa.h::sweep_scanline` | `geometry/agg_raster/{cells,contours,grid_ops,mod}.rs`; signature-only caller edits in `geometry/support_grid.rs` | Signed remainder stepping; area/cover arithmetic; duplicate-x accumulation; conditional x increment; row/block/directional sweep order; boundary insertion and contour successor selection; float scaling and corner shrink order. |
| `EdgeGrid.cpp::{calculate_sdf,contours_simplified}` (`:600–900`, `:1283–1400`), `EdgeGrid.hpp::cell_inside_or_crossing` | `geometry/edge_grid/simplify{.rs,/field.rs}` | Row-major seeds; exact wedge/interior predicates and cast sites; frozen signs; Danielsson propagation; pre-fill snapshot; contour closure-before-successor selection; integer scaling. |
| `MarchingSquares.hpp` start-cell search and ring scanning | `geometry/marching_squares{.rs,/rings.rs}` | Word/bit scan order, loop bounds and residual ambiguous direction bits. |
| `AABBTreeLines.hpp::{point_outside_closed_contours,distance_from_lines_extra}`, `Line.hpp` segment distance | `geometry/line_distance_tree{.rs,/outside.rs,/query/mm.rs}` | Query point/scale pairing only; recursive child order, strict ties/pruning and floating arithmetic unchanged. |

Included work is limited to extracting existing algorithm stages, expressing coordinate/dimension pairs and borrowed seed/query state, and equivalent integer guards. Two existing local argument-count suppressions disappear because their signatures now meet the threshold. No new suppression, allocation, assertion change, DEBUG probe or algorithm is introduced. Adjacent upstream behavior remains deferred exactly as before; existing adapters are not expanded into an independent slicing pipeline. Functional Arachne, tower/AdaptivePA/bridge and FillBase-connect code stays outside this slice.

Validation retains every existing test and comparator. Full workspace gates cover the test-only EdgeGrid implementation; production-only clippy is insufficient to verify its 15 diagnostics. Existing gyroid/TPMSFK, support-grid, signed-distance and tree-query fixtures provide regression coverage, not a claim of exhaustive new upstream parity.

Geometry results: all 7,589 workspace tests passed (nine unchanged skips); ordinary all-target/all-feature clippy, fmt and core/WASM compilation passed. Geometry diagnostics fell from 43 to zero in the full workspace log. Core library warnings fell from 249 to 221; core test warnings from 265 to 222 (221 duplicates); CLI parity retains 20 warnings. All changed Rust files remain below 400 lines. No test or comparator source changed.

## Fill engines small slice

This slice targets 25 diagnostic locations: adaptive hooks/octree (5), multiline vertical rasterization (12), TPMS-D (5), honeycomb (1), line (1), and locked-zag (1). Lightning WIP is saved separately for a later slice. Exclude cooling, Grid path ordering, FillBase support-connect, and bridge-owned signatures; adjacent fill-entity edits only adapt the changed APIs.

Upstream boundaries are `Fill/FillTpmsD.cpp::make_waves`, `Fill/FillHoneycomb.cpp::_fill_surface_single`, `Fill/FillLine.cpp`, `Fill/FillRectilinear.cpp` vertical rasterization and locked-zag fill, and `Fill/FillAdaptive.cpp` hooks/octree. Destinations remain the existing `ares-core/src/fill` engines and their `project_slice/fill_entities` callers. Include only structural extraction, borrowed mesh/triangle grouping, removal of unused inputs, and equivalent guards. Defer all new upstream functionality; no compatibility scaffold grows into a new pipeline. Preserve integer division, unary negation, floating expressions/casts, traversal/tie order, error ordering and every assertion. Existing full-workspace and complete-output controls provide the pre-edit baseline. The initial extraction passed 907 existing fill-matching tests; the final slice must rerun complete gates and controls before publication.

Known upstream gap: `OrcaSlicer/src/libslic3r/Fill/FillAdaptive.cpp:1033–1046` inserts boundary segments into the R-tree so hooks do not cross the boundary. Ares had not implemented this boundary-collision logic: `connect_lines_using_hooks` accepted `_boundary` but never read it. Removing that unused parameter does not change current behavior or close the parity gap. When this upstream behavior is ported, restore the boundary input and implement the boundary insertion/collision checks.

Validation: all 25 targeted diagnostics are gone; 133 unique core locations remain (154 workspace locations, including 21 CLI locations). Full workspace nextest passed 7,595 tests with nine unchanged skips, and ordinary workspace/all-targets/all-features clippy, fmt and core/WASM compilation passed. Complete controls matched the baseline exactly: official 389 PASS / eight existing replay-environment FAIL; seeded 397/397 PASS. All Ares output hash multisets matched across 389 official and 400 seeded artifact labels.

## Lightning structural slice

Target 30 existing diagnostic locations in `ares-core/src/fill/lightning`; leave `Generator::new` and its cross-module arguments unchanged. The upstream boundaries are `Fill/Lightning/DistanceField.cpp` sampling/update, `TreeNode.cpp` realign/straighten/prune/propagate and junction trimming, and `Layer.cpp` grounding/reconnect. Rust destinations remain the corresponding existing Lightning modules, with grounding geometry moved into `layer_geom/geometry.rs` to keep changed files below 400 lines. Include only helper extraction, grouped borrowed context/settings, same-type cast removal and equivalent guards. Defer new upstream behavior; no compatibility shell or Ares-specific pipeline is introduced.

Non-negotiable invariants: preserve arithmetic/cast ordering, live post-move positions, RefCell borrow release points, tree traversal and strict grounding tie ordering, grid predicate ordering, swap-removal behavior and emitted bytes. Tests and comparator thresholds are unchanged. The integrated full-workspace run (7,597 PASS / nine existing skips) is the pre-edit baseline; existing focused Lightning tests and full gates/official plus seeded controls must validate this slice. Exclude `gcode_emit/processor.rs` and `ares-vgcode`.

Validation: 30 unique locations removed, leaving core 103 / workspace 124. Six focused Lightning tests and all 7,597 workspace tests passed (nine existing workspace skips); ordinary workspace/all-targets/all-features clippy, fmt and core/WASM compilation passed. Complete baseline/candidate controls have identical statuses: official 391 PASS / eight existing replay-environment FAIL, seeded 399/399 PASS. Ares output SHA-256 multisets are identical across 391 official and 402 seeded artifact labels. No assertions or comparator thresholds changed; every changed Rust file is below 400 lines.

## Arachne sandwich small slice

Scope is six diagnostic locations in `project_slice/perimeters/arachne/sandwich.rs` only. All motion, processor and vgcode files remain excluded, as do the tower candidate's classic-perimeter and raft files.

Upstream boundaries: `PerimeterGenerator.cpp:1987–2061` proximity ordering and `:2398–2456` sandwich resequencing. Include only equivalent match/loop guards and extraction of the repeated strict-largest-position scan. Defer new upstream behavior; no Ares-owned pipeline or compatibility shell is added. Preserve first-wins ties, length evaluation order, stable partitioning, sentinel/index arithmetic, clone order, assertions and all output bytes. Existing integrated workspace and new-seeded controls are the pre-edit baseline; do not add or weaken tests or comparator thresholds.

Six diagnostic locations removed; remaining core 97 / workspace 118. All 7,605 workspace tests passed with nine existing skips; ordinary all-targets/all-features clippy, fmt and core/WASM compilation passed. Candidate official controls: 399 PASS / eight unchanged replay-environment FAIL. Candidate new-seeded controls: 407/407 PASS, matching the baseline; all 402 Ares artifact-label hash multisets are identical. Official failure identities match the retained historical environment failures; this does not claim an official baseline/candidate byte comparison for this slice.

## Coherent fuzzy-skin noise small slice

Fresh ordinary workspace/all-targets/all-features clippy reports 118 unique workspace locations, 97 in core. Distribution includes motion 22, base_support 16, raft/fills 7, classic perimeters 7, and coherent noise 4. Choose only the four coherent-noise locations and their direct caller. Leave all motion/tower/tool-ordering, processor/vgcode/wasm, profiles/config, and fuzzy-skin main-entry signatures untouched.

Source boundary: `libslic3r/Feature/FuzzySkin/FuzzySkin.cpp::get_noise_module` uses the libnoise `noisegen.cpp::GradientNoise3D`, `Perlin::GetValue`, `Billow::GetValue`, and `Voronoi::GetValue` ports already in these Rust files. Include only grouped coordinate/lattice arguments and equivalent Voronoi grid traversal; defer new upstream behavior and do not add any pipeline or compatibility shell. Preserve f64 and wrapping-i32 expression order, all casts, octave accumulation, z/y/x visitation, strict first-wins nearest-point selection, and emitted bytes. Existing focused fuzzy-skin tests run before edits; full workspace and complete-output controls remain the acceptance mechanism, with no assertion or threshold changes.

Pre-edit focused tests: 33 PASS. Candidate workspace: 7,608 PASS / nine existing skips; ordinary workspace/all-targets/all-features clippy, fmt and core/WASM compilation pass. Four unique diagnostics removed, leaving core 93 / workspace 114. Candidate official controls: 399 PASS / the same eight replay-environment FAIL. Candidate new-seeded controls: 407/407 PASS; all 402 Ares artifact-label hash multisets match the sandwich slice output (a cross-revision output-equality check, not a fresh run of the immediate parent). No official baseline/candidate hash comparison is claimed. Both changed Rust files remain below 400 lines.

## Fill base_support small slice

Scope: 15 diagnostics in `fill/base_support/{extend,mark,take,mod}.rs`; `connect.rs` remains excluded (support-tip connect slice).

Upstream boundary: `Fill/FillBase.cpp` — `base_support_extend_infill_lines` (`:1834–1948`), `mark_boundary_segments_overlapping_infill` (`:1352–1435`), `take_limited` arc consuming (`:578–670`, `:2364–2432`), and the in/out split logic in the mod scanner. Include only symmetric-side helper extraction (`probe_extension_side`/`ProbeTarget`/`SideProbe`, `arc_inside_tube`/`ArcWalk`/`Direction`, `finish_arc`/`FinishArc`, `trim_neighbor`) and equivalent guards. Preserve arc direction (ccw/cw), first-endpoint sign flip, trim order, float comparisons, and traversal order; no assertion or threshold changes. `connect.rs` untouched; no new suppressions (the pre-existing `#[expect]` stays).

## Seam KD-tree small slice

Scope: three argument-count diagnostics in `project_slice/seam_placement/kdtree.rs`. Existing seam tests run before editing. Exclude all motion/tower/tool-ordering, processor/vgcode/wasm and profiles/config boundaries.

Source boundary: `libslic3r/KDTreeIndirect.hpp::build_recursive` (`:81–108`), `partition_input` (`:111–160`), and `visit_recursive`/nearest visitor (`:169–190,200–251,266–271`), used by `SeamPlacer.hpp`'s candidate tree. Include only tuple grouping of node/axis and inclusive left/right indices in private function parameters/call sites. Preserve median arithmetic, partition comparisons/swaps, recursion order, distance arithmetic and last-visited equal-distance selection. No new upstream behavior, pipeline, compatibility shell, assertion or threshold changes.

Three diagnostics removed, leaving core 90 / workspace 111. Focused pre-edit seam tests: 58 PASS. Candidate workspace: 7,608 PASS / nine existing skips; ordinary workspace/all-targets/all-features clippy, fmt and core/WASM compilation pass. Candidate official controls: 399 PASS / the same eight replay-environment FAIL. Candidate new-seeded controls: 407/407 PASS and all 402 Ares artifact-label hash multisets match the coherent-noise slice output — a cross-revision output-equality check, not a fresh parent baseline. No official baseline/candidate hash comparison is claimed.

## G-code post-processing and adjacent geometry slice

Remaining diagnostics are counted once per primary file/line/column across all workspace targets: 243 locations, led by `gcode_emit` (86), `fill` (73), and adjacent `pressure_equalizer` (12). This slice targets 64 locations: FanMover26, cooling9, pressure equalizer12, brim3, processor3, object ordering5 and curl6.

Upstream boundaries are `GCode/FanMover.cpp` (buffering, delayed fans and kickstart), `GCode/CoolingBuffer.cpp` (fan/comment/feedrate rewriting), `GCode/PressureEqualizer.{cpp,hpp}` (parsing, limiting and split emission), `Brim.cpp`, `GCode/GCodeProcessor.cpp` (arc accounting), `ShortestPath.cpp` (instance chaining), and `SupportSpotsGenerator.cpp` / `ExtrusionProcessor.hpp` (curl estimates). Destinations stay inside their existing `project_slice/gcode_emit` and `project_slice/pressure_equalizer` modules. Included changes only extract existing stages or group source-owned arguments; all adjacent upstream behavior remains deferred as before, and no scaffold is expanded into a new Ares algorithm. Timelapse, travel lifecycle, AdaptivePA dispatch, Arachne and bridge-direction logic are excluded.

Preserve numerical expression/cast order, invalid-command partial state, exact comments and newlines, duplicate fan suppression, forward/backward phase distinctions, contour tie choices and every existing assertion. Tests and strict-output controls must be rerun after refactoring; no warning suppression or comparison tolerance change is permitted.

G-code slice results: all 64 targeted diagnostics are gone, leaving **179 unique primary diagnostic locations** across workspace targets (243 before). Ordinary clippy reports core library 157, core tests 158 including 157 duplicates, and CLI parity 21. Full workspace nextest passed 7,594 tests with nine unchanged skips; fmt and core/WASM compilation passed. Every changed Rust file remains below 400 lines. The only test edits group the same endpoint arguments at two curl call sites; all assertions and comparator sources remain unchanged.

Complete controls retain exactly the baseline status sets: official 388 PASS / eight existing replay-environment FAIL (`ORCA_REPLAY_*` absent), seeded 396/396 PASS. The eight official failures are not semantic passes. Actual Ares G-code SHA-256 multisets also match the baseline for all artifact labels: 383 labels with output in official mode and 394 in seeded mode.

## travel retract/lift helpers (slice)

- `start_travel::retract::decide` duplicated the `G92 E0` reset, the
  `GCode.cpp:7678-7681` lazy-lift gate, and the wipe-then-replan
  sequence (`GCode.cpp:7436-7443`) across its Hilbert branches.
- Extracted `travel::reset_e` reuse, `lift::schedule_retract_tail`
  (shared `schedule_at` gate), `wipe_sequence::emit_retract_e` /
  `gcode_comment`, and `emit_z_restate` in `restate.rs`; bundled the
  >5-arg signatures into `RetractContext` / `FirstTravel` structs.
- Behavior-preserving: identical predicates and emit order; verified by
  the ares-core suite and the orca_parity control set.
