# Task 22O.18 — Fill-surface shell preparation Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o18-fill-surface-preparation.md`

## Status

Implemented from Ares baseline `3d5a2546d7cdf17509f15406b8fd01548495c0a2` after approved independent/OpenCode specification and plan reviews; pinned Orca HEAD is `8500fcdccaa10b5099ac20d252af3a7c560046f1`. The KSR O18 checksum remains `-126362407653399901571400348049652748978`; totals are `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294, 113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011, 0, 0]`. Seventeen focused O18 tests, 209 O10-O18 regressions, and 5,607 workspace tests with 2 skipped pass with all required native, strict Clippy, WASM, formatting, diff, LOC, forbidden-pattern, dependency, pinning-removal, and staging gates. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Frozen validation contract

First repair the early capability gate so `resolved.views.full.process.print.spiral_mode` cannot bypass O17's deferred spiral detection/corrections through false record-local flags. Then port only the slicing-state mutation in `PrintObject.cpp:587-592` and `LayerRegion::prepare_fill_surfaces` at `LayerRegion.cpp:935-973`, stopping before `discover_vertical_shells` at `PrintObject.cpp:595-596`.

Directly reached source contracts are `Surface.hpp:8-114`, `libslic3r.h:52` (`EPSILON = 1e-4`), and the static-false `PrintObject::infill_only_where_needed` definition at `PrintObjectSlice.cpp:22`. Pass 1 therefore retags `Top` to `Internal`, never `InternalVoid`; pass 2 retags `Bottom|BottomBridge` to `Internal`; pass 3 retags `Internal` to `InternalSolid`. Cancellation, status/logging, and debug SVG infrastructure remain explicitly deferred.

## Gate 0 — Freeze review inputs

1. Require literal approval from independent and OpenCode spec reviewers.
2. Review this plan with both reviewers against the approved spec; any finding returns to both.
3. Before RED, assert the Ares and Orca commits above. Add no Rust until both plan reviewers approve the same plan.

## Exact file manifest and budgets

Production/current tests:

- `crates/ares-core/src/project_slice/capabilities.rs` (currently 58 LOC), `planning.rs` (108), and `tests/capabilities.rs` (306): global spiral gate, call-site arguments, precedence tests; each stays below 400 LOC.
- `crates/ares-core/src/project_slice/region_slices.rs` (366) and `region_slices/tests/surface_kind.rs` (17): `RegionSurfaceKind::InternalSolid = 5`, non-bridge match, and in-place retag; both stay below 400 LOC.
- `crates/ares-core/src/project_slice/prepare_infill.rs` (1): add `fill_surfaces` declaration only.
- new `prepare_infill/fill_surfaces.rs` (at most 220 LOC), `fill_surfaces/tests.rs` (at most 50), and shards `tests/retag.rs`, `tests/ownership.rs`, and `tests/alignment.rs` (each at most 300).
- `prepare_infill/surface_type_detection/stage.rs` (188): replace the exhaustive kind-to-step match with the fixed four `(kind, step)` pairs so O17 never accepts `InternalSolid`.
- `tests/prepare_infill/surface_type_detection/ksr.rs` (197): add an explicit impossible-`InternalSolid` test-invariant arm while preserving the literal `[usize; 24]` O17 totals schema and values; the checksum already hashes numeric kind.
- `tests/prepare_infill/surface_type_detection/options.rs` (212): add the same explicit impossible-O17 arm, preserving its four-bucket O17 result.

Integration/lifecycle:

- `crates/ares-core/src/project_slice/tests/prepare_infill.rs` (1): add `mod fill_surfaces;`.
- new `tests/prepare_infill/fill_surfaces.rs` (at most 60) and shards `fixture.rs`, `options.rs`, `ownership.rs`, `ksr.rs`, and `lifecycle.rs` (each at most 300).
- `tests/prepare_infill/surface_type_detection/cleanup.rs` (265): move the existing public constrained-stack (64 KiB on Unix, 256 KiB on Windows) / depth-10,000 terminal witness through O18; keep below 400 LOC.
- `crates/ares-core/src/project_slice.rs` (261): O18 public preparation/consumer wiring, below 400 LOC.
- `incomplete_sink.rs` remains untouched at 399 LOC; reuse `incomplete_sink::surface_type_detection::consume_object`.

Documentation: O18 spec/plan, `docs/architecture/option-parity-v4.md`, and `docs/roadmap.md`.

## Task 1 — Write every behavioral RED before the successor exists

1. **Capability RED:** update all `capabilities::validate` test call sites with the new global operand. Add unit and public-archive cases for global spiral `true`, including bottom-shell count/thickness covering every reached layer. Prove counterfactually that every record-local spiral flag would be false, yet early result is `UnsupportedProjectFeature("spiral_mode")` and O17/O18 invocation counts are zero. Add a mixed-invalid case and place the new spiral check after all existing capability keys so current key-major precedence is unchanged.
2. **Direct source RED:** require `RegionSurfaceKind::InternalSolid = 5`, false bridge classification, and in-place retag access. On ordered distinct geometry/metadata, pin inactive behavior; each individual pass; combined sequential behavior; `99.99995%` inside, exact-difference `1e-4`, and `99.9998%` outside; and direct `spiral_mode = true` behavior where passes 1/3 skip and pass 2 runs.
3. **Identity RED:** capture the fill vector pointer/capacity, every surface expolygon contour/hole/point pointer, complete order, and all metadata; require selected kind tags to be the only changes.
4. **Alignment RED:** require a separate first phase to validate all object counts, per-object record counts, `Some`/`None` slot presence, source identities, and input/O17 slot identities before any mutation. Synthetic mismatches must panic before retagging, with a borrowed pre-mutation snapshot unchanged; a valid `None` slot must remain `None`.
5. **Typed provenance RED:** real KSR-derived archives independently and jointly activate top-zero, bottom-zero, density-100, `99.99995%`, and `99.9998%`. A normal-part/model-settings region override leaves the global process option unchanged but changes the reached record. A synthetic aligned two-object state combines separately parsed archives whose embedded region options differ, proving orchestration does not use global/first-object/first-record shortcuts.
6. **Ownership RED:** capture boxed predecessor, perimeter, thin-fill, typed-slice, fill-surface, fill-boundary, and no-overlap allocations before O18; require all identities and non-kind values unchanged afterward.
7. **Lifecycle RED:** in `fill_surfaces/lifecycle.rs`, require O18 once on KSR and zero for global spiral, counterbore, O17 interface/active-extra-bridge, and an instrumented O17 geometry failure. Public success remains `ProjectSlicingIncomplete` before vertical shells. In the named O17 `cleanup.rs`, require the constrained-stack (64 KiB on Unix, 256 KiB on Windows) / depth-10,000 drop-probe witness to pass through the O18 consumer.
8. **KSR RED:** add an independent-twice O18 characterization test with literal O17 checksum/totals predecessor guards, but leave the new O18 literal marked for parent capture. Record focused failures before creating `PreparedPostFillSurfacePreparation`.

## Task 2 — Implement the minimum GREEN

1. Pass `resolved.views.full.process.print.spiral_mode.0` into `capabilities::validate`; reject it after existing capability checks with the existing stable key. Never use `PerimeterInputRecord::spiral_mode` for this project capability.
2. Add only `InternalSolid = 5` to `RegionSurfaceKind`, update `is_bridge()` to false, and add a focused mutable retag method. Do not change the separate `SurfaceType` enum.
3. Preserve O17's four-kind contract exactly: use four fixed `(kind, GeometryStep)` pairs in clipping; O17 KSR/options test helpers treat `InternalSolid` as an impossible O17 invariant, so the frozen O17 checksum and 24-value totals remain byte-for-byte unchanged.
4. Implement an in-place helper with three literal loops in source order. It accepts the exact global spiral operand for direct source semantics and reads `RegionOptions.top_shell_layers.0`, `bottom_shell_layers.0`, and `sparse_infill_density.0`. It introduces no allocation, geometry, sorting, or validation.
5. Implement an allocation-free two-phase orchestrator using only borrowing and iterator traversal—no temporary collections or defensive copies. Phase one validates every aligned object/record/slot/identity without writing. Phase two destructures O17 into its exact boxed predecessor and object vector, walks the aligned Classic-prelude input records, resolves each record only through `input_object.region_options(input)`, reads the global operand only from retained `resolved.views.full.process.print.spiral_mode`, and retags only `record.fill_surfaces`.
6. Return `PreparedPostFillSurfacePreparation` directly, not `Result`; O18 is infallible after O17. The successor owns the boxed predecessor and moved O17 object records directly, not a stale nested O17 wrapper.
7. Wire `slice_project_sync` through O18 without `?`, consume object records through the existing child sink, dispose the traversal predecessor iteratively, and remain incomplete.
8. Run every Task 1 test GREEN, then focused O17 regressions to prove its literal checksum/totals remain unchanged.

## Task 3 — Freeze characterization and literal transitions

1. Diff corresponding O17 and O18 fill-surface kind sequences—no extra transition instrumentation—to freeze nonzero literal transition counts for each active 3MF case. Preserve typed slices, coordinates, metadata, and all non-fill fields.
2. Freeze O18 checksum/totals only after a clean run. KSR non-spiral 5/3/15% makes O18 structurally equal to O17, but named state type and invocation assertions independently prove execution. O18 totals include a separate `InternalSolid` bucket even though the KSR value is zero.
3. Remove diagnostics/parent-capture markers and rerun focused O18 plus O17-O10 regressions.

## Task 4 — Documentation, full verification, review, and ship

1. Update architecture, roadmap, spec, and plan with boundary, three-pass order, global spiral capability repair, static-false pass-1 target, typed record provenance, identity rules, exact checksums/totals/counts, and next boundary `discover_vertical_shells` at `PrintObject.cpp:595`.
2. Run focused O18, O17-O10 regressions, `cargo nextest run --workspace`, `cargo check --workspace --all-targets`, strict workspace all-target/all-feature Clippy, both WASM checks, rustfmt, and `git diff --check`.
3. Audit every Rust source/test file below 400 LOC. Diff-scope zero-addition audits must enumerate `unsafe`, `include!`, `include_bytes!`, binary payloads, broad lint allowances, source-text/hash/line pinning tests, reference-G-code reads, Orca runtime/FFI, and fixture name/hash/layer-count/geometry-identity branches. Require `git diff -- Cargo.toml Cargo.lock` empty and prove no `.pi-subagents/` or generated evidence is staged.
4. Record no public API, persisted format, dependency, migration, or compatibility layer. Rollback restores the O17 terminal and removes only O18 state/wiring/tests/docs while preserving the global spiral capability repair, which closes a known bypass of O17's deferred spiral envelope, and all other reviewed O1-O17 behavior.
5. Run an independent read-only six-dimensional implementation review and separate OpenCode review against the same final diff/evidence. Main thread applies findings, reruns affected/full gates, and returns the updated diff to both until literal approval.
6. Synchronize final status only after both approvals. Create small Conventional Commits for slicing/docs, exclude `.pi-subagents/` and generated logs, push `main`, and verify clean `HEAD == origin/main`.

## Stop condition

Stop O18 only when every RED preceded the successor, the global spiral gap is closed before O17, every populated record runs the exact three source passes from typed aligned options, only fill kinds change with full identity preservation, KSR and active provenance mutations pass, O17 checksum/totals remain frozen, public slicing reaches O18 once and remains incomplete before vertical shells, all gates pass, both final reviewers approve, documentation is synchronized, and commits are pushed.
