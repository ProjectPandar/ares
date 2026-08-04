# Task 22O.19 — Single-region vertical-shell cache Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o19-vertical-shell-cache.md`

## Status

Implemented and validated from Ares baseline `b63c30c73dd0bc44b83bc3396f3180d8872d96c4` against pinned Orca `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen evidence: cache checksum `-114359197324258778780701398534712718623`, parent-bound successor checksum `148296943860974241781127169756103364063`, totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, spacings `[457079, 377079]`, 21 focused tests, 310 O10-O19 regressions, and 5,630 workspace passes with 2 skipped. Final independent six-dimensional and OpenCode rereviews both returned `VERDICT: APPROVE`.

## Validation contract

Port caller `PrintObject.cpp:595-596` and only cache declarations/gating plus single-region cache population in `PrintObject::discover_vertical_shells` at `PrintObject.cpp:2008-2027,2111-2149`, with directly reached `SurfaceCollection.cpp:45-60`, `ExPolygon.hpp:300-318`, `LayerRegion.cpp:21-28`, `PrintRegion.cpp:8-53`, `Flow.cpp:129-145,200-205`, `Flow.hpp:62-69`, `ClipperUtils.hpp:19-34,343`, `ClipperUtils.cpp:438-567`, and vendored Clipper 6 Paths offset/union semantics. Defer the multi-region branch `2028-2109` and stop before projection at `PrintObject.cpp:2153`.

## Gate 0

1. Independently and through OpenCode review the spec; any repair returns to both.
2. Review this plan with both reviewers against the approved spec; any repair returns to both.
3. Assert exact Ares/Orca commits before RED. Add no Rust until both plan reviewers approve.

## Task 1 — RED direct cache semantics and successor contract

Exact files/budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shells.rs` (module root/successor sidecar, at most 220 LOC);
- new children `vertical_shells/cache.rs`, `stage.rs`, `types.rs` (all cache/sidecar/successor types), and `cleanup.rs` (partial staged-cache error disposal, distinct from reused public-terminal object sinks), each at most 300 LOC;
- direct test root `vertical_shells/tests.rs` and shards `tests/cache.rs`, `tests/options.rs`, `tests/transaction.rs`, each at most 300 LOC;
- add `vertical_shells` to `prepare_infill.rs` (currently 2 LOC) beside existing declarations;
- borrowed-reference offset adapter in `geometry/clipper/offset/expolygon.rs` (114 LOC), re-export-only edits in `geometry/clipper/offset.rs` (122), `geometry/clipper.rs` (184), and `geometry.rs` (106), each below 400 LOC; it accepts stable `&[&ExPolygon]` and performs the existing per-expolygon raw/conditional-NonZero-Paths algorithm without source geometry clones;
- integration root `tests/prepare_infill/vertical_shells.rs` and shards `fixture.rs`, `ksr.rs`, `options.rs`, `ownership.rs`, `lifecycle.rs`, `cleanup.rs`, and `metamorphic.rs`, each at most 300 LOC; declare from `tests/prepare_infill.rs` (currently 2 LOC);
- `project_slice.rs` (currently 261 LOC) wiring only; reuse existing iterative child sinks and leave `incomplete_sink.rs` unchanged at 399 LOC;
- documentation paths: this spec/plan, `docs/architecture/option-parity-v4.md`, and `docs/roadmap.md`.

RED before successor implementation:

1. Pin `EnsureAll` active and Rust variants `None`, `CriticalOnly`, and `Moderate` geometry-inactive. Add a multi-region archive proving object-level `perimeters/preflight.rs:29-36` fails before O19 invocation.
2. Pin top-only, bottom-only, mixed/repeated kinds, empty, multiple expolygons, and holes. Require upstream `SurfaceCollection` pointer/filter order through Rust's equivalent borrowed typed-slice vector, per-surface contour-then-holes, stable surface order, one stable original-slice-order predicate matching `Bottom | BottomBridge` (never kind-major ordering), fill-boundary `ExPolygon` flattening, raw per-expolygon Paths order when no union runs, and conditional Paths-union result order.
3. Use a spacing above the `f32` exact-integer boundary to distinguish `(i64 as f32) * 0.05_f32` from f64/reordered alternatives. Pin miter `3.0` and conditional positive union with source-shaped overlapping inputs.
4. Pin exact object→slot→top-offset→bottom-offset→holes call order for active records. Top failure must emit `[Top]` with bottom absent; bottom failure must emit `[Top, Bottom]`. Require `InvalidInput("vertical-shell cache geometry is outside the supported Clipper range")`, minimal/range-error coordinates, no partial cache, stage-before-move, miter/3.0, raw contour/hole orientation, empty behavior, conditional NonZero Paths union, and no PolyTree.
5. Add alignment tests before writes, prove O18 `None` slots remain `None` without shifting populated indices, snapshot all O18 allocations, and prove no source/cache aliasing. Add zero-O19-invocation witnesses for global spiral, counterbore, multi-region, O17 interface-shell/active-extra-bridge, and an O17 geometry failure. Require the consuming O19 `prepare` path for top and bottom errors plus separate direct-success disposal and public-incomplete consumption on constrained test stacks (64 KiB on Unix, 256 KiB on Windows) with both traversal/hierarchy trees at depth 10,000, weak/drop-probe confirmation, exact errors, no successor, and no partial cache.
6. Add real-archive ensure-mode, `internal_solid_infill_line_width`, model-part override, ZIP repack, non-slicing metadata, and component-scale mutations before GREEN. Pin the aligned prelude `solid_infill_spacing` bits first, including layer-zero first-layer override. For a selected rectangular source/cache pair, require `cache_x_span - source_x_span == 2 * rounded_expansion` before and after scale mutation, equal expansion under unchanged options, and changed source/cache spans.
7. Add independent-twice KSR characterization with predecessor checksum `-126362407653399901571400348049652748978` and O18 totals `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294, 113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011, 0, 0]`. Define delimiters for objects, slots/`None`, O18 predecessor checksum, cache fields in top/bottom/holes order, every path/count/point/coordinate, and a parent-capture marker. Parse twice independently and record RED.
8. First add a compiling API shell. Then add and independently record RED with `cargo nextest run -p ares-core --no-fail-fast <filter>` for each exact filter: `vertical_shells::tests::cache`, `vertical_shells::tests::options`, `vertical_shells::tests::transaction`, `project_slice::tests::prepare_infill::vertical_shells::options`, `::ownership`, `::metamorphic`, `::lifecycle`, `::cleanup`, and `::ksr`. Expected failures are missing behavior or parent-capture mismatch—not compile errors or another group's failure. Rerun each identical filter GREEN before O18-O10 regressions.

## Task 2 — GREEN source-shaped cache and transactional project stage

1. Add `VerticalShellCache` with ordered `Vec<Polygon>` top, bottom, and holes. The successor owns the exact boxed traversal predecessor and exact `Vec<PreparedSurfaceTypeObject>` unchanged plus an aligned `Vec<VerticalShellCacheObject>` sidecar containing `Vec<Option<VerticalShellCache>>`; do not rebuild larger record vectors.
2. Validate all object/record/prelude slot identities first, including object-level one-region invariants and stable `None` slots. Stage every cache while borrowing O18. Resolve ensure mode through `input_object.region_options(input)` and spacing through aligned `ClassicPreludeRecord.solid_infill_spacing` only.
3. For active records, compute `(spacing as f32) * 0.05_f32`; stable-filter O18 typed slices into borrowed `&ExPolygon` references; call the borrowed source-shaped adapter with `JoinType::Miter, 3.0`; preserve pinned conditional union rather than Ares's unconditional `offset_expolygons` sibling; flatten fill boundaries without union. Map errors to the exact O19 text. Pin spacing `16_777_217`: cast operand `16_777_216.0_f32`, expansion `838_860.8125_f32`, bits `0x494c_cccd`;  define `rounded_expansion` through the pinned Clipper fixed-round result used by selected rectangular offsets.
4. For inactive modes return empty caches without invoking geometry. Do not sort, canonicalize, mutate source records, or add defensive copies.
5. Only after every stage succeeds, destructure O18 and move its predecessor/object vector verbatim beside the staged cache sidecar. On error, consume unchanged O18 objects and deep predecessor iteratively through existing sinks.
6. Wire public slicing through O19 once, reuse iterative sinks, and stay `ProjectSlicingIncomplete`.
7. Run every RED GREEN and O18 focused regressions.

## Task 3 — Freeze KSR and active option evidence

1. Freeze literal active transition counts/checksums from exact package edits: in `Metadata/project_settings.config`, replace `"ensure_vertical_shell_thickness": "ensure_all"` with each literal `none`, `ensure_critical_only`, and `ensure_moderate` and `"internal_solid_infill_line_width": "0.42"` with literal `"internal_solid_infill_line_width": "0.52"`; separately prove fallback by replacing solid width `"0.42" → "0"` and generic `"line_width": "0.42" → "0.52"`. In `Metadata/model_settings.config`, replace exactly `<part id="1" subtype="normal_part">` with that line followed by `<metadata key="ensure_vertical_shell_thickness" value="ensure_all"/>` and `<metadata key="internal_solid_infill_line_width" value="0.52"/>`, while global values remain unchanged. Pin layer-zero spacing to `initial_layer_line_width = 0.5`, later spacing to reached solid width/fallback, and rely on named O18/prelude guards for filament/nozzle selection unless explicit selectors are mutated. Compare complete unrelated O18 state/allocation identity.
2. Freeze O19 KSR checksum/totals only after clean implementation output; remove diagnostics.
3. Repack all identical ZIP entry bytes in exact reverse entry order using Stored compression and Unix permissions, and in `Metadata/model_settings.config` replace both literal `value="ksr_fdmtest_v4.drc"` names with `value="task22o19_renamed"`;  require identical O19. In `3D/3dmodel.model`, replace exactly the component attribute `transform="1 0 0 0 1 0 0 0 1 0 0 0"` with `transform="2 0 0 0 1 0 0 0 1 0 0 0"`. Use object index `0`, populated slot `0`, bottom source expolygon/path `0`, and `cache.bottom_surfaces[0]` as the named rectangular pair; freeze source/cache spans and Clipper-rounded expansion satisfying the spec relation. Keep reference G-code unread.

## Task 4 — Full gates, review, docs, ship

1. Update architecture, roadmap, spec, and plan with boundary, exact cast/offset/Paths order, option/flow provenance, ownership, checksums/totals/counts, and next source line `PrintObject.cpp:2153`.
2. Run `cargo nextest run -p ares-core --no-fail-fast vertical_shells`, `cargo nextest run -p ares-core --no-fail-fast -E 'test(/classic|layer_region|surface_type_detection|fill_surfaces|vertical_shells/)'` for O10-O19 regressions, `cargo nextest run --workspace --no-fail-fast`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo check -p ares-wasm --target wasm32-unknown-unknown`, `cargo fmt --all -- --check`, and `git diff --check`.
3. Run exact audits: `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 >= 400 {print; bad=1} END {exit bad}'`; both `git diff --unified=0 -- '*.rs' | rg '^\+[^+].*(unsafe|include!|include_bytes!|allow\()'` and `git diff --unified=0 -- '*.rs' | rg '^\+[^+].*(ksr_fdmtest_v4.*(hash|layer|geometry)|reference.*gcode|OrcaSlicer.*(Command|FFI))'` must be empty, so established fixture embedding is not a false positive; `git diff -- Cargo.toml Cargo.lock` empty; the addition/deletion audit `git diff --unified=0 -- '*test*.rs' | rg '^[+-][^+-].*(OrcaSlicer/src|8500fcdc|source.*line|source.*hash)'` empty; and `git diff --cached --name-only | rg '^(\.pi-subagents/|target/parity/)'` empty.
4. Record no public API/persistence/dependency/migration/compatibility layer; rollback removes only O19 and restores O18 terminal.
5. Run independent six-dimensional and OpenCode implementation reviews against the same diff/evidence. Main thread applies findings, reruns affected plus full gates, and returns the identical updated diff/evidence to both until approval.
6. Synchronize status, make small Conventional Commits, push main, verify clean `HEAD == origin/main`.

## Stop condition

Stop O19 only when the exact single-region cache exists transactionally from typed aligned inputs, every ordering/cast/offset/ownership invariant is proven, public slicing reaches O19 once and remains incomplete before projection, all gates pass, both final reviewers approve, docs are synchronized, and commits are pushed.
