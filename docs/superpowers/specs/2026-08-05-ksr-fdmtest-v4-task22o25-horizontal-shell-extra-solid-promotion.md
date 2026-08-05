# Task 22O.25 — Horizontal-shell extra-solid promotion Spec

## Status

Implemented from Ares baseline
`551a8c10420783cd56f185fe8e22ea3d3eab015c` against pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1` after this spec and its plan each
received literal `VERDICT: APPROVE` from the required independent reviewer and
a separate default-model OpenCode reviewer. Task 22O.24 exact-SHA Tier-1 run
`31004838262` passed its complete predecessor matrix, and both post-fix
implementation/evidence review paths approve O25. O25 remains unshipped until
the work is committed and pushed and that exact SHA passes the same matrix.

## Upstream source boundary

This milestone rewrites only the first coherent operation of
`PrintObject::discover_horizontal_shells`:

- the caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:618`;
- function and source-order region/layer/config iteration at
  `PrintObject.cpp:3955-3964`;
- `extra_solid_infills` matching and in-place `stInternal` to
  `stInternalSolid` promotion at `PrintObject.cpp:3966-3972`;
- the option declaration/default at `PrintConfig.hpp:1112` and
  `PrintConfig.cpp:2987-2992`;
- `check_layer_id_pattern` at `Utils.hpp:730` and `utils.cpp:1749-1830`;
- the source surface vocabulary and metadata at `Surface.hpp:7-42`.

The exact stop is after line 3972, before the
`ensure_vertical_shell_thickness == evstAll` gate at lines 3974-3976. The gate,
all neighbor propagation, and all horizontal-shell geometry remain deferred.
This stop is coherent because the included block is an independently
observable option-controlled state transition and requires no interpretation
of the later control-flow gate.

The Rust destination is a crate-private
`prepare_infill::horizontal_shell_promotion` successor after
`PreparedPostVerticalShellAssignment`. It mutates the retained
`PreparedSurfaceTypeRecord::fill_surfaces` only after whole-project parsing and
alignment preflight, retains the exact O24 predecessor/sidecar graph, and adds
no durable geometry or branch-state sidecar. O19-O25 remain temporary
source-compatibility state rather than an Ares-owned slicing pipeline.

## Exact behavior

For each aligned printing region and layer in source order:

1. select that record's typed resolved region options;
2. test the resolved raw `extra_solid_infills` string for emptiness exactly as
   pinned line 3966 does;
3. only for a nonempty raw string, parse the typed schedule and invoke its
   matcher against the zero-based planned layer-array index, whose matching
   semantics convert it to a one-based layer number;
4. when the schedule matches, scan `fill_surfaces` in existing order and retag
   every surface whose kind is exactly `Internal` to `InternalSolid`;
5. leave every other kind, `slices`, geometry, path ordering, allocation,
   metadata, perimeter result, and inherited sidecar unchanged.

The operation has no sparse-density condition. It must not reuse the existing
legacy `sparse_or_extra_solid` role helper because pinned lines 3966-3971 do not
consult `sparse_infill_density`. It uses planned layer-array order, not stored
layer ID. Promotion is in-place, stable, allocation-free, and idempotent.

The real KSR fixture resolves `extra_solid_infills` to the empty schedule, so
all 460 records are exact no-ops for O25. That baseline proves typed wiring,
state preservation, ownership, cleanup, and repeatability, but not active
promotion. Active behavior must be proven with normal typed archive mutations
and direct synthetic records, never fixture identity branches.

## Typed option boundary

Ares already owns the typed schedule representation in
`options/infill/extra_solid.rs`. O25 must reuse its `ExtraSolidInfills` parser
and `matches_layer` implementation rather than duplicate the grammar or build a
legacy `SliceOptions` JSON object.

Expose only the narrowest crate-private raw-string parser needed by resolved
`RegionOptions`, and make the existing JSON entry delegate to it. Read
`RegionOptions.extra_solid_infills.0` during whole-project staging. An exactly
empty raw string stages a no-op without invoking the parser or matcher; a
nonempty raw string is parsed before any project mutation. Preserve the existing
stable Ares external error exactly:

`SliceError::InvalidInput("invalid extra_solid_infills pattern")`.

Pinned Orca's `std::stoi` helper stores positive components in signed `int`.
The shared Rust parser must therefore accept only strictly parsed decimal values
in `1..=i32::MAX`, then convert them to `usize`. This source-sized numeric domain
makes `start + count` non-overflowing on 32-bit WASM as well as 64-bit native;
checked addition remains required at the matching boundary so malformed or
future out-of-domain state cannot panic. Oversized components and ranges return
the same stable invalid-pattern error on every Tier-1 target, before mutation.
Raw-string and JSON-delegation tests must cover `i32::MAX`, `i32::MAX + 1`, and
an explicit range near the bound on native and WASM.

Pinned Orca's `std::stoi` accepts some numeric prefixes and ignores some
malformed comma tokens while the established Ares typed boundary rejects them.
O25 otherwise does not broaden into a parser-compatibility rewrite: the spec
records this existing discrepancy honestly, preserves the already tested stable
Ares error, and does not claim malformed-token parity. Ordinary valid empty,
repeating, explicit-list, and `base#count` patterns remain source-compatible.

## Transactionality, alignment, and ownership

Before parsing a schedule or mutating a surface, validate the complete inherited
O24 alignment envelope: selected coordinate scale against typed printable area;
outer object and O18-O24 sidecar lengths; record, plan, input, prelude, and
`lslices` lengths; `Some`/`None` slots; source object/transform identity;
planned index and layer IDs; current layer/region; region ID; and the existing
single compatible-region constraint.

While borrowing O24, parse the resolved schedule for every populated record and
stage only whether that record matches. No record is retagged until all schedule
parsing and alignment checks for the complete project succeed. A later-record
parse failure therefore exposes no successor and no partial promotion, and the
exact O24 state is disposed iteratively.

After successful staging, move the exact O24 graph and commit in stable
object/record/surface order with `RegionSurface::retag`. Empty/nonmatching
records are not touched at all, preserving their vector pointer, capacity, all
inner allocations, and metadata. Matching records preserve the same allocation
properties because only enum discriminants change.

The successor retains the exact boxed predecessor, objects, caches,
projections, trims, regularizations, and filters. Its disposal reconstructs the
exact O24 state and delegates to O24 cleanup. Public slicing invokes O25 once
after O24, disposes it iteratively, and continues returning
`ProjectSlicingIncomplete`.

## Explicitly deferred

- trace logging at `PrintObject.cpp:3957`, per-layer cancellation at line 3961,
  caller cancellation at line 619, and any new public cancellation API;
- `ensure_vertical_shell_thickness == evstAll` at
  `PrintObject.cpp:3974-3976`;
- `print_z`, `bottom_z`, Top/Bottom/BottomBridge source gathering, layer-count
  and thickness windows at lines 3978-4023;
- all safety-offset intersection, density/ensure-mode control flow, opening,
  width filtering, expansion, union, collection rebuilding, and metadata
  templating at lines 4024-4145;
- debug SVG output and the function close at lines 4152-4161;
- `process_external_surfaces`, fill generation, seams, ordering, motion,
  G-code, and post-processing;
- malformed-pattern parser parity outside the established typed Ares boundary;
- public API, persisted formats, new dependencies, fallback, Orca runtime/FFI,
  reference-G-code reads/replay, or fixture name/hash/layer/geometry branches.

## Tests and acceptance

### Direct option and promotion witnesses

1. Preserve existing parser tests and add raw resolved-string coverage for
   empty, whitespace/quotes, `N`, `N#K`, comma lists, explicit ranges,
   `i32::MAX`, `i32::MAX + 1`, near-boundary range arithmetic on 32-bit WASM and
   64-bit native, and the exact invalid-pattern error. JSON parsing must delegate
   to the same raw parser and produce identical results/errors.
2. Freeze one-based schedule matching at array-index boundaries and distinguish
   planned array index from deliberately nonconsecutive stored layer IDs.
3. A matching record interleaving Top, Internal, Bottom, InternalVoid,
   InternalSolid, and BottomBridge retags every and only Internal value while
   preserving order, geometry, contour/hole/point ordering, metadata, vector
   capacity/pointer, and all inner allocations.
4. Empty and nonmatching schedules are allocation-exact no-ops. Reapplying O25
   is idempotent.
5. Matching promotion remains active at 0%, 15%, and 100% sparse density,
   proving that no legacy density gate was introduced.
6. Promotion affects `fill_surfaces`, never `slices`, and uses the record's
   resolved region options rather than a global/first-record schedule.

### Integration, rollback, cleanup, and lifecycle

7. Every inherited O24 alignment mismatch fails before parsing/commit. A
   malformed schedule in a genuine later record or later typed option context
   proves whole-project rollback and zero early commits.
8. Preserve exact predecessor, object, sidecar, record, geometry, and unrelated
   field ownership. Only matching Internal kind tags may change.
9. Prove iterative cleanup for both independent 10,000-node predecessor
   families on direct success, parse failure, and public-incomplete disposal,
   using the shared Unix/non-Windows 64 KiB and Windows 256 KiB stack baseline.
10. Public slicing reaches O25 exactly once after O24. Every earlier capability
    or O17/O19/O20/O21/O22/O23/O24 failure invokes O25 zero times and preserves
    error precedence.
11. Typed model-part precedence and a real archive mutation to a valid matching
    schedule produce nonzero promotion through normal config resolution. ZIP
    repack/timestamp/order and non-slicing rename remain invariant.
12. Two independent real KSR captures reassert O24 evidence and resolve the
    empty schedule through the aligned record path. Freeze 460 aligned raw
    schedule visits, zero nonempty guards, zero parser/matcher invocations, zero
    matches/promotions/changed records, unchanged kind/geometry/record digests,
    and exact O25 invocation/cleanup. Tests never read reference G-code.

### Mutation and repository gates

13. Required compiling behavioral mutations are killed by intended witnesses:
    omit one-based conversion; use stored layer ID; reverse match condition;
    promote only the first Internal; promote InternalVoid/InternalSolid or
    external kinds; rebuild surfaces with default metadata; consult one global
    schedule; add the legacy sparse-density gate; mutate `slices`; bypass O25
    public wiring; skip inherited scale/alignment validation; or commit before
    a later parse failure. Restore production byte-exactly before final GREEN.
14. Focused O25 tests, explicit O21-O25 regressions, workspace Nextest, native
    all-target check, strict all-target/all-feature Clippy, four WASM checks,
    optimized default/feature browser-WASM export audit, both Playwright runs,
    rustfmt, diff, dependency, staging, rollback, LOC, and forbidden-pattern
    audits pass.
15. Every Rust file remains below 400 LOC and each new O25 shard is at most 300
    LOC. New Rust contains no `unsafe`, `include!`, `include_bytes!`, broad lint
    allowance, binary oracle payload, source-text/hash/line pinning test,
    reference-G-code access, fixture identity branch, Orca command/FFI, or
    fallback. Tests use ordinary `mod` and real files.
16. The final committed and pushed O25 SHA must pass the repository's complete
    Tier-1 matrix on Windows, macOS, Ubuntu, and optimized browser-WASM,
    including export audit and both Playwright runs. O25 cannot ship while that
    exact-SHA run is pending or failing; local gates and the predecessor O24 run
    do not substitute for it.
17. Independent and default-model OpenCode spec and plan reviews both return
    literal `VERDICT: APPROVE` before implementation. After implementation, an
    independent six-dimensional reviewer and separate OpenCode reviewer judge
    requirements completeness, logic correctness, boundary cases, code quality,
    test coverage, and actual execution. The parent sole writer fixes every
    blocker and repeats both reviews until approval.

## Implementation evidence

The implemented successor preserves the exact O24 graph and exposes no public
API. Two independent real-KSR captures freeze:

- checksum `58727684244877231975278290246623082466`;
- record-sequence digest `160750122870413723145549886803558415603`;
- event-sequence digest `95826544899519698779358289371798515623`;
- unchanged surface digest
  `-107673730348313625723619859456104452971`;
- 460 records, all unchanged, kind totals `[113, 6, 48, 1281, 575, 0]`,
  geometry totals `[2023, 270, 73848]`, event totals `[460, 0, 0, 0, 0]`, zero
  commits, and exactly one prepare/disposal.

A normal typed archive mutation promotes all 1,281 Internal surfaces in 460
matching records, ending at Internal/InternalSolid `0/1856` while retaining
outer, record, surface, contour, hole, point, and sidecar allocation identity.
Forty-two focused O25/shared-option tests, 191 explicit O21-O25 regressions,
and 5,856 workspace tests with 2 skipped pass. Fourteen compiling mutations
cover the specified parser, behavior, transaction, ownership, cleanup,
lifecycle, metamorphic, KSR, and browser-WASM boundaries. Native, strict
Clippy, four WASM checks, optimized export audit, two 10-test Playwright runs,
formatting, and static audits are green. Both implementation/evidence reviews
approve; the exact pushed-SHA Tier-1 result remains required before shipping.

## Documentation and rollback

After final evidence is frozen, update `docs/architecture/option-parity-v4.md`,
`docs/roadmap.md`, this spec, and the plan with exact KSR captures, test totals,
review evidence, and the next source boundary at `PrintObject.cpp:3974`.

Mechanical rollback restores O24 as the public terminal consumer and removes
only the O25 module/state/wiring/tests/docs plus the narrow crate-private
raw-string schedule parser seam. It retains all O24 geometry, vocabulary,
sidecars, options, dependencies, persisted formats, and public API unchanged.
