# Task 22O.29 — Source-taking RegionExpansion propagation

## Status and source boundary

Approved implementation specification. Independent and default-model OpenCode
reviewers both returned literal `VERDICT: APPROVE`. Exact predecessor is released O28 at
`be334375be871eb12ca98c98d889b65a92d13a37`; exact-SHA Tier-1 run
`31156094839` is green. Rewrite target is OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone ports only:

- `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.cpp:463-466`, the
  source-taking `propagate_waves(src, boundary, params)` overload;
- `RegionExpansion.cpp:468-477`, its scalar overload; and
- declarations at `RegionExpansion.hpp:74-83`.

O27 already owns the supplied-seed propagation kernel at
`RegionExpansion.cpp:440-461`; O28 already owns `wave_seeds`. O29 is their
crate-private composition, not an Ares-owned pipeline or project stage.

## Implemented status and bounded evidence

O29 is implemented locally against pinned
`RegionExpansion.cpp:463-466,468-477` and `RegionExpansion.hpp:74-83`. The Rust
destinations are the crate-private
`geometry::region_expansion::propagate_waves_from_sources` and
`propagate_waves_from_sources_with_steps` wrappers, reexported only through the
crate-private geometry facade. The parameter wrapper passes literal
`sorted=true` to unchanged O28 `wave_seeds` and then hands its complete ordered
seeds directly to unchanged O27 `propagate_waves`. The scalar wrapper calls
`RegionExpansionParameters::build` exactly once and delegates exactly once,
retaining the same explicit `CoordinateScale` for both operations.

The final five-test composition shard freezes the complete compact vector, the
complete sorted and unsorted two-source/two-boundary vectors, one complete
16-point Normal-scale scalar polygon, and one complete 128-point LargeBed
scalar polygon. It also proves valid discovery before a propagation error,
direct error forwarding, empty-input precondition order, and scalar delegation.
The final focused composition run passes 5/5 tests and the complete
RegionExpansion regression passes 58/58 tests. Ten runtime mutations were
killed and restored one at a time, and one differently typed signature mutation
was rejected at compile time. The source audit truthfully records structural,
not behavioral, evidence for the scalar wrapper: exactly one builder call is
followed by exactly one parameter-wrapper call, with no direct `wave_seeds` or
O27 call. The frozen six-argument scalar API exceeds the workspace's configured
five-argument Clippy threshold, so that function alone carries a reasoned
`#[expect(clippy::too_many_arguments)]`; no lint `allow` was added. Final
physical LOC are 172 for `propagate.rs`, 55 for `region_expansion.rs`, 150 for
`geometry.rs`, 5 for the RegionExpansion test root, and 264 for
`composition.rs`; every Rust file remains below 400 LOC and the new shard
remains below 300 LOC.

The restored final local state passes composition 5/5, RegionExpansion 58/58,
O26 lifecycle 3/3, and workspace 5,999/5,999 with 2 skipped. Native all-target
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized WASM
builds, wasm-bindgen export and JavaScript syntax audits, two 11/11 Playwright
runs, static audits, and disposable rollback are green. Final documented-state
independent six-dimensional and default-model OpenCode reviews return literal
`VERDICT: APPROVE`. O29 was released as implementation commit `55c2c23` and
documentation commit `118f6a7`; exact-pushed-SHA Tier-1 run `31168584784`
passed all format, WASM/browser, Linux, Windows, and macOS jobs at
`118f6a72b33926efe41ced1c931f9a51b26b2945`.

The real compiling RED artifact at `/tmp/task22o29-red-focused-all.txt`
predates the final test refactor. It records an earlier eight-test iteration:
seven assertions failed against the empty signature stubs and
`scalar_scale_outputs_differ` passed while both wrapper stubs returned empty
(the passing comparison used the explicit pipelines). The tests were later
consolidated and strengthened into the final five-test shard, including the
valid-discovery-before-propagation-error witness. There is no fresh
chronological RED for that exact final test list, and none is synthesized.
Mutation kills and their restored GREEN runs are post-hoc recurrence evidence,
not original RED evidence.

O29 adds no lifecycle wiring, public API/export, KSR checkpoint, G-code byte,
option, persisted state, or ARD change; ARD-0024 remains unchanged. Public
slicing still consumes O26 and returns `ProjectSlicingIncomplete`. The full
local native/WASM/browser/static and disposable-rollback gates are green as
recorded above. Final documented-state independent six-dimensional and
default-model OpenCode rereviews return literal `VERDICT: APPROVE`; exact-SHA
Tier-1 run `31168584784` passed at
`118f6a72b33926efe41ced1c931f9a51b26b2945`.

Mechanical rollback removes only the two O29 wrappers, their crate-private
reexports and signature assertions, the composition shard and module
registration, and O29 documentation. It retains O27 propagation, all O28
ClipperZ/wave-seed/AABB behavior, and the exact O26 lifecycle.

## Scope

Included:

1. one source-taking parameter entry that discovers sorted seeds and invokes
   unchanged O27 propagation;
2. one source-taking scalar entry that builds parameters once and delegates to
   the parameter entry;
3. exact crate-private Rust signatures, ordered composition tests, error and
   precondition ordering tests, mutation evidence, and documentation.

Deferred: `propagate_waves_ex` beginning at `RegionExpansion.cpp:480`,
`expand_expolygons`, all expansion union/merge helpers,
`clipper_round_offset_error`, `LayerRegion::process_external_surfaces`,
`PrintObject::process_external_surfaces`, project lifecycle/checkpoints,
Options, CLI/WASM/browser exports, fill/toolpath/seam/motion/G-code and
post-processing behavior, and normalized KSR G-code parity.

Public slicing must continue to consume O26 and return
`ProjectSlicingIncomplete`.

## Crate-private API

Rust cannot overload the existing O27 `propagate_waves(&[WaveSeed], ...)`.
Freeze these descriptive names instead; names ending in `_ex` are prohibited
because upstream reserves that suffix for `RegionExpansionEx` output.

```rust
pub(crate) fn propagate_waves_from_sources(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError>;

pub(crate) fn propagate_waves_from_sources_with_steps(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    expansion: f32,
    expansion_step: f32,
    max_nr_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError>;
```

`CoordinateScale` explicitly replaces Orca's mutable global scale. The same
scalar-entry scale must be passed to both `RegionExpansionParameters::build`
and source discovery. Scalar expansion values are already scaled and must not
be rescaled. A parameter entry may be called with parameters built for a
different scale; this is a trusted internal-caller mismatch and receives no
validation or fallback in O29.

Both functions remain `pub(crate)` through `geometry::region_expansion` and the
crate-private `geometry` facade. They must not be exported through `lib.rs`,
CLI, WASM, browser JavaScript, or another crate.

## Exact composition semantics

The parameter entry is straight-line source composition:

```rust
let seeds = wave_seeds(
    src,
    boundary,
    params.tiny_expansion,
    true,
    scale,
)?;
propagate_waves(&seeds, boundary, params)
```

Requirements:

- pass literal `true`; reuse O28's fixed comparator and complete seed/path
  order without sorting, regrouping, canonicalizing, filtering, or rebuilding;
- complete seed discovery before starting O27 propagation;
- pass `params.tiny_expansion` without reassociation or conversion;
- pass the original `boundary` and `params` references unchanged;
- return the first existing `ClipperError` directly with `?`; no `SliceError`,
  wrapper error, validation, retry, fallback, or partial output;
- add no early empty-input shortcut. O28's positive tiny-expansion assertion
  must run before its own empty-side shortcut, then O27 receives empty seeds.

The scalar entry is exactly:

```rust
let params = RegionExpansionParameters::build(
    expansion,
    expansion_step,
    max_nr_steps,
    scale,
);
propagate_waves_from_sources(src, boundary, &params, scale)
```

It must build once before any empty-input shortcut or geometry operation. The
existing O27 `f32`/`f64` expression order and positive assertions remain
unchanged. Do not duplicate the builder, manually construct parameters, swap
arguments, or inline a second discovery/propagation pipeline.

O27 direct propagation and O28 seed discovery are behaviorally unchanged.
No new engine, dependency, state record, option, cancellation hook, or
lifecycle seam is permitted.

## Files and LOC

Allowed production edits:

- `crates/ares-core/src/geometry/region_expansion/propagate.rs` — two wrappers;
- `crates/ares-core/src/geometry/region_expansion.rs` — crate-private reexports
  and function-pointer arity/type-shape assertions;
- `crates/ares-core/src/geometry.rs` — crate-private facade reexports and
  function-pointer arity/type-shape assertions.

Allowed tests:

- add ordinary shard
  `crates/ares-core/src/geometry/tests/region_expansion/composition.rs`;
- register it from the existing ordinary module root.

Allowed documentation: this spec, its reviewed plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O28 ship-state corrections.
ARD-0024 does not change: O29 adds no kernel or architecture.

Every Rust file remains below 400 physical lines; the new test shard is at most
300 lines. Do not use `include!`, `include_bytes!`, source concatenation,
`unsafe`, FFI, filesystem access, native threads, platform branches, fixture
identity, or reference-G-code access.

## TDD and focused evidence

Write compiling RED tests before production bodies. Stubs may expose the
approved signatures and return empty output so tests compile and fail; missing
symbols or compilation errors are not RED behavior evidence. Archive commands,
exit codes, and bounded failure excerpts under `/tmp/task22o29-*`; do not
commit generated oracle output.

The focused shard must cover:

1. **Exact ordered parameter composition.** Reuse the compact O28→O27 square
   witness and assert the full ordered `(src_id, boundary_id, points)` vector,
   not counts, area, or bounds. Also compare against the explicit
   `wave_seeds(..., true, scale)` then O27 handoff.
2. **Mandatory sorted discovery.** Use the existing two-source/two-boundary
   witness whose sorted and unsorted seed orders differ. Assert complete
   wrapper output and IDs; prove it equals the `sorted=true` pipeline and
   differs from `sorted=false`.
3. **Empty behavior and preconditions.** Valid parameters with either empty
   side return `Ok([])`. Invalid `tiny_expansion` still panics even when both
   sides are empty. Each invalid scalar precondition still panics on empty
   inputs because parameter construction occurs first.
4. **Operation and error order.** A seed-discovery range error escapes directly.
   A valid discovery followed by an out-of-range propagation step returns the
   propagation `ClipperError`. Invalid scalar construction precedes invalid
   geometry. O28 remains the authoritative boundary-before-source witness
   because both range sites share the same error variant.
5. **Scalar delegation.** For Normal and LargeBed scales, compare complete
   scalar output against one explicit build followed by the parameter entry.
   Use a geometry whose scale-derived parameter values produce observably
   different complete outputs, and freeze both ordered vectors after review.
6. **Signatures and inactivity.** Function-pointer assertions freeze arity,
   each distinct argument type position, and return type only; they do not
   prove visibility or distinguish the two adjacent `f32` scalar arguments.
   Module/reexport/export audits prove crate-only visibility, while ordered
   scalar behavioral witnesses and the expansion/step swap mutation prove the
   semantics of same-typed positions. Existing O26 lifecycle and public KSR
   incomplete tests remain unchanged and green; static search must find no O29
   symbol under project slicing, `lib.rs`, CLI, or WASM.

Existing O27 parameter, direct propagation, and O28 wave-seed/AABB tests remain
unchanged and are part of the focused regression.

## Mutation gate

Each listed runtime mutation must fail a named focused test, be restored, and
rerun GREEN. The listed signature-shape mutation must be rejected at compile
time by a function-pointer or call-site assertion, then be restored and rerun
GREEN; it is not a compiling behavioral mutation. Record a
`/tmp/task22o29-mutation-manifest.txt` with command, failure excerpt,
classification, and restored result. Minimum observable mutations:

1. `sorted=true` to `false`;
2. `params.tiny_expansion` to another parameter field;
3. skip discovery or pass empty seeds;
4. reverse/sort output after propagation;
5. hardcode scalar-builder scale;
6. swap scalar expansion and step or alter max-step forwarding;
7. move scalar build after an empty shortcut;
8. suppress/map a discovery or propagation error;
9. change arity or a differently typed argument position.

Faithfully inlining the parameter-entry body into the scalar entry is
behaviorally equivalent and therefore is not a mutation claim. The required
single-delegation structure is instead a non-generated source/diff review item:
the final scalar body must contain exactly one builder call followed by one
call to `propagate_waves_from_sources`, with no direct `wave_seeds` or O27 call.
Do not add instrumentation or pretend this structural audit is a killed
behavioral mutation.

Do not claim that an O29 test distinguishes boundary-before-source range errors
when both produce `CoordinateOutOfRange`; retain O28's tested evidence.
Parameter-entry scale forwarding into O28's AABB may be output-insensitive for
valid contained seeds. The explicit argument, function signature, unchanged
O28 dual-scale AABB tests, and scalar dual-scale witness are the accepted
combined evidence; do not add instrumentation solely to manufacture a witness.

## Validation and acceptance

Required local gates:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown
```

Also require optimized default/feature WASM builds, unchanged wasm-bindgen
exports, JavaScript syntax audit, two Playwright runs, LOC/dependency/forbidden
pattern/lifecycle/staging audits, `git diff --check`, a disposable mechanical
rollback rehearsal, and final independent six-dimensional plus default-model
OpenCode reviews returning literal `VERDICT: APPROVE`.

After reviewed docs and verification, commit and push only the O29 allowlist.
Shipping requires `HEAD == origin/main` and a successful Tier-1 run whose
`headSha` equals the exact pushed commit.

Acceptance requires:

1. both exact crate-private signatures and no public/lifecycle change;
2. literal sorted O28 discovery followed by unchanged O27 propagation;
3. scalar build-once then parameter-entry delegation with one explicit scale;
4. complete ordered path/ID witnesses, including sorted/unsorted distinction;
5. exact precondition and direct `ClipperError` order;
6. compiling RED chronology, every observable named mutation killed/restored,
   and the behaviorally equivalent scalar-delegation structure verified by
   source/diff review without a false mutation claim;
7. focused/full/native/WASM/browser/static/review/Tier-1 gates green;
8. documentation truthfully states that O29 changes no KSR checkpoint or
   G-code byte and lists all deferred adjacent behavior.

## Rollback

Remove only the two O29 wrappers, their crate-private reexports and signature
assertions, the O29 composition test shard/registration, and O29 documentation.
Retain all O27 direct propagation/types/end types/tests, all O28
ClipperZ/wave-seed/AABB code/tests/ARD amendment, and the exact O26 lifecycle.
No persisted state, migration, option, checkpoint, adapter, manifest, or
fixture rollback is required.

## Residual risks

- Parameter-taking callers can provide a scale inconsistent with the scale used
  to build parameters; trusted lifecycle wiring must later pass the retained
  project `CoordinateScale` consistently.
- Valid wave-seed output often cannot expose AABB epsilon differences, so O29
  does not overclaim an isolated forwarding mutation witness.
- The observed and frozen scalar ordered vectors and manual-pipeline equality
  are composition evidence, not a new independent Orca oracle.
- Full KSR seed counts and G-code remain unavailable until source-cited
  external-surface, fill, toolpath, motion, serialization, and processing
  milestones are implemented.
