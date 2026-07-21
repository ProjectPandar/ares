# Task 22N Implementation Plan: Perimeter Inputs and Flow Dispatch

## Objective and gate

Implement the approved Task 22N specification as the smallest complete fixed
Orca boundary after Task 22M: single-region per-layer perimeter-generator input
records, four exact Flow values, spiral/model-rotation state, and exhaustive
Classic/Arachne dispatch. Stop before either perimeter process body. Public
`slice_project` continues to return `ProjectSlicingIncomplete`.

No production or tracked-test change begins until the specification and this
plan are frozen into one exact frame and both independent document reviewers
approve that frame. Any document-byte change invalidates both approvals.

## Working rules

Every behavior change follows RED, minimal GREEN, focused verification, and
read-only package review before the next package. Expected values must exist
independently before production behavior. Tests exercise public archive loading
or the crate-private stage boundary, not private helper shape.

Manual edits use `apply_patch`. Parallel workers may edit only disjoint leaf
paths explicitly assigned by the main thread and report changed paths,
commands, results, physical LOC, and risks. Shared registration roots remain
main-thread owned. Existing user files, ignored oracle evidence, generated
output, and untracked `main.obj` are preserved and excluded from staging.

All Rust source and test files remain below 400 physical LOC. Tests use real
`mod` files. No source-splitting macro, generated textual source, new unsafe,
broad lint allowance, dependency, fixture branch, reference G-code read, or
legacy rectangular perimeter fallback is permitted.

## Planned path manifest

Production and tracked-test implementation is limited to:

- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project/transform.rs`;
- `crates/ares-core/src/project/tests/transform.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/compensation.rs`;
- delete `crates/ares-core/src/project_slice/compensation/flow.rs` after the
  released external-flow behavior moves without byte drift;
- `crates/ares-core/src/project_slice/compensation/preflight.rs`;
- `crates/ares-core/src/project_slice/perimeters.rs`;
- `crates/ares-core/src/project_slice/perimeters/types.rs`;
- `crates/ares-core/src/project_slice/perimeters/flow.rs`;
- `crates/ares-core/src/project_slice/perimeters/preflight.rs`;
- `crates/ares-core/src/project_slice/perimeters/context.rs`;
- `crates/ares-core/src/project_slice/checkpoints.rs`;
- `crates/ares-core/src/project_slice/task22n_oracle.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/compensation.rs`;
- `crates/ares-core/src/project_slice/tests/compensation/flow.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/types.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/flow.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/preflight.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/context.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/archive.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/flow_pairs.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/flow_pairs/oracle.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/flow_pairs/widths.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/flow_pairs/selectors.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/flow_pairs/bridges.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/context_pairs.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/fixture/context_pairs/options.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/oracle.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/task22n_synthetic.bin`;
- `crates/ares-core/Cargo.toml`;
- `crates/ares-wasm/Cargo.toml`;
- `crates/ares-wasm/src/lib.rs`;
- delete `crates/ares-wasm/tests/browser/task22m-vectors.mjs`;
- `crates/ares-wasm/tests/browser/task22n-vectors.mjs`;
- `crates/ares-wasm/tests/browser/server.mjs`;
- `crates/ares-wasm/tests/browser/project-slice-page.mjs`;
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`;
- `.github/workflows/tier1.yml`;
- `docs/architecture/option-parity-v4.md`; and
- `docs/roadmap.md`.

The fixed spec and plan are also part of the final content frame. Ignored
`.superpowers/sdd/task22n-oracle/` evidence is never staged. If implementation
needs a path outside this manifest or any file would reach 400 LOC, write a
small amendment, freeze a new spec/plan frame, and reacquire both approvals
before editing that path.

## Package 0: fixed evidence, independent oracle, and document approval

Create the ignored Task 22N oracle from fixed Orca objects only. Record the
fixed Ares/Orca commit, tree and relevant blob identities; prove the local Orca
checkout drift; inventory the KSR typed Options through an independent ZIP/JSON
probe; and prove painted MMU, fuzzy, interlocking, and multi-region gates
inactive.

The oracle must cover all Flow constructor and context cases named by the
specification. For KSR it fail-closed validates the approved 3,008,346-byte
Task 22M predecessor, preserves those bytes, and derives only the N payload
from a separately frozen ZIP/JSON/XML Option/transform probe plus the exact
additive layer recurrence. It emits a deterministic composite `ARES22N\0`
aggregate and readable trace and runs twice from a clean VS2022 C++20
`/O2 /fp:precise` build. Add two wrong algorithms: logical-to-physical nozzle
pre-mapping and direct width scaling for bridge ratio. Freeze their distinct
outputs. Register no tracked expected constant yet.

Compute one sorted-path, length, SHA-256 content frame for the specification
and plan. Dispatch two read-only reviewers in parallel:

1. fixed-source boundary, formulas, branch inventory, oracle independence,
   included behavior and explicit deferrals;
2. current Ares types/APIs, path manifest, TDD feasibility, WASM/browser,
   transactionality and LOC.

Both must APPROVE the same frame. Any correction requires a new frame and two
fresh approvals.

## Package 1: shared Flow record and released external-Flow preservation

Allowed leaf paths:

- `project_slice.rs`;
- `project_slice/perimeters.rs`;
- `project_slice/perimeters/types.rs`;
- `project_slice/perimeters/flow.rs`;
- `project_slice/compensation.rs`;
- `project_slice/compensation/flow.rs` deletion;
- `project_slice/compensation/preflight.rs`;
- `project_slice/tests.rs`;
- `project_slice/tests/perimeters.rs`;
- existing and new Flow test leaves listed in the manifest.

Write compile-RED tests for the absent shared project-slice Flow record. Copy
independent exact expected bits from the approved oracle for width, height,
spacing, nozzle, bridge, volume per mm, and equality. Cover ordinary auto,
absolute, percent and fallback widths, first/later layers, selector fallback,
invalid spacing/volume, and both coordinate scales without scaling the Flow
record itself. Equality must ignore spacing and volume exactly as fixed
`Flow.hpp`, while tests compare those derived fields explicitly.

GREEN moves the released Task 22M external-perimeter resolver onto the shared
record and pure constructor without changing any Task 22M checkpoint byte or
error precedence. Make `PostCompensationPrintObject`'s crate-private borrowed
parts accessor compile in every build so later global preflight can inspect all
objects before moving any. Remove the old leaf only after all Task 22M focused
and complete checkpoint tests pass exactly. Do not add the other three roles
yet.

Focused gates:

```text
cargo nextest run -p ares-core task22m_flow
cargo nextest run -p ares-core task22m
cargo nextest run -p ares-core task22n_flow_record
```

Package review must verify exact predecessor identity, f32 conversion points,
no filament map, no geometry, and no compatibility-flow adapter.

## Package 2: four role Flows and transactional preflight

Allowed leaf paths:

- `project_slice/perimeters.rs`;
- `project_slice/perimeters/flow.rs`;
- `project_slice/perimeters/preflight.rs`;
- `project_slice/perimeters/types.rs`;
- `project_slice/tests/perimeters.rs`;
- `project_slice/tests/perimeters/fixture.rs`;
- corresponding new Flow/preflight test leaves.

Add one RED behavior at a time for internal-perimeter and solid-infill role
selection, then thick and nonthick overhang Flow. Cover every
preparation-reachable `with_cross_section` branch and EPSILON boundary,
configured/auto bridge width, positive ratios, nozzle selector fallback,
differing role selectors, first-layer override, and exact error keys. The
shrink-width case must prove that `with_width` recomputes spacing; the
grow-height and decrease-round cases prove that they retain prior spacing. The
noncanonical increase branch requiring an earlier `with_spacing` mutation is
deferred. Zero, negative, and nonfinite `bridge_flow` are preflight REDs for
`invalid Orca option bridge_flow`, not successful records. Add all-config
transaction REDs in which a later invalid layer/object leaves every predecessor
object untouched.

GREEN ports only fixed `PrintRegion.cpp`, `LayerRegion.cpp`, and `Flow.cpp`
semantics needed by the four records. Preflight resolves every required record
for all objects/layers before it consumes Task 22M state. Use exhaustive enums
and immutable validated records; do not add error handling inside trusted pure
helpers.

Focused gates:

```text
cargo nextest run -p ares-core task22n_flow
cargo nextest run -p ares-core task22n_preflight
cargo nextest run -p ares-core task22m
```

## Package 3: single-region layer context and dispatch

Allowed leaf paths:

- `project_slice/perimeters.rs`;
- `project_slice/perimeters/context.rs`;
- `project_slice/perimeters/types.rs`;
- `project/transform.rs`;
- `project/tests/transform.rs`;
- `project_slice/tests/perimeters.rs`;
- `project_slice/tests/perimeters/types.rs`;
- `project_slice/tests/perimeters/context.rs`.

Start with REDs for zero layers, empty surfaces, one and three layers, exact
current/lower/upper/upper-same-region resolution, one compatible region,
identity preservation, complete multi-surface current/upper collections, and
multiple object occurrences. Add independent REDs for spiral thresholds,
Arachne/nonspiral, Arachne/spiral-to-Classic, Classic/spiral,
disabled/enabled model alignment and transform rotation. The transform RED
freezes a direct crate-private `(m00, m10)` accessor including signed zero;
rotation must not be reconstructed through transformed-point subtraction.

GREEN consumes preflight plus owned Task 22M objects to create one record slot
per planned layer. It stores indices into the owned predecessor and exposes
read-only resolvers; it never clones the geometry tree. Empty surfaces have no
record. The wrapper preserves plan, sidecars, compensated surfaces, raw
`lslices`, ordering and coordinate scale exactly. It does not call a perimeter
process function.

Focused gates:

```text
cargo nextest run -p ares-core task22n_context
cargo nextest run -p ares-core task22n_dispatch
cargo nextest run -p ares-core task22m
```

## Package 4: checkpoint, real 3MF matrix, and public lifecycle

Allowed leaf paths:

- `project_slice/checkpoints.rs`;
- `project_slice/task22n_oracle.rs`;
- `project_slice/perimeters.rs`;
- `project_slice/perimeters/context.rs`, only to remove the temporary Package 3
  dead-code expectation once production wiring consumes the function;
- `project_slice.rs`;
- `project_slice/tests/perimeters.rs`;
- `project_slice/tests/perimeters/fixture.rs`;
- `project_slice/tests/perimeters/fixture/archive.rs`;
- `project_slice/tests/perimeters/fixture/flow_pairs.rs`;
- `project_slice/tests/perimeters/fixture/flow_pairs/oracle.rs`;
- `project_slice/tests/perimeters/fixture/flow_pairs/widths.rs`;
- `project_slice/tests/perimeters/fixture/flow_pairs/selectors.rs`;
- `project_slice/tests/perimeters/fixture/flow_pairs/bridges.rs`;
- `project_slice/tests/perimeters/fixture/context_pairs.rs`;
- `project_slice/tests/perimeters/fixture/context_pairs/options.rs`;
- `project_slice/tests/perimeters/oracle.rs`;
- `project_slice/tests/perimeters/task22n_synthetic.bin`.

### Package 4 test-module layout amendment (2026-07-21)

The first complete Option-matrix drafts reached 760 physical lines in
`flow_pairs.rs` and 674 physical lines in `context_pairs.rs` after rustfmt.
They may not be compressed with long lines, formatting exclusions, or source
inclusion macros to evade the strict under-400-LOC rule. The five child paths
listed above are therefore approved only as real Rust test modules:

- `flow_pairs.rs` owns the shared archive/loader/M/N comparison harness;
- `flow_pairs/oracle.rs` owns independent fixed Flow bit literals;
- `flow_pairs/widths.rs`, `selectors.rs`, and `bridges.rs` own the three
  disjoint Option-pair tables;
- `context_pairs.rs` retains the complete real-KSR inventory; and
- `context_pairs/options.rs` owns the six context Option pairs and their
  shared comparison harness.

This layout amendment changes no behavior, production path, Option case,
oracle value, gate, or deferral. Every module must be independently readable,
must stay below 400 physical lines after rustfmt, and may not use an include
macro for source splitting. No new path may be written until two independent
read-only reviewers approve the unchanged specification plus this amended plan
as one exact content frame.

The 23,071-byte aggregate with SHA-256
`6cba4f1ada6716cb0e3a6a60609f2e8385ed69ea50fd975d04e1c89f601296bd`
is the pre-repair historical oracle and is explicitly superseded by the repair
amendments below. The latest repair amendment owns the current exact identity.
Loading the tracked binary with `include_bytes!` is fixture access, not source
splitting; no Rust source may be split through an include macro.

First add parser KATs and absent-stage REDs for exact magic, all record fields,
resolved neighbor geometry, wrong magic, truncation, malformed values,
trailing bytes and exact EOF. Register the complete approved synthetic
aggregate before GREEN. The encoder writes N magic, an exact-length complete M
wire, and the independent N payload without changing predecessor bytes.
Structural tests, rather than the M wire, prove preservation of scale, typed
config and non-kind Surface metadata.

Build semantic in-memory 3MF Option pairs before production orchestration.
Each pair varies exactly one consumed Option and freezes the public loader's
typed value, semantic-entry identity, predecessor expectation and independent N
change. Flow Option pairs disable elephant-foot compensation and therefore
require exact M equality. Selector pairs use effective one/two, with a
raw-zero/raw-one normalization-invariance pair whose base extruder is fixed at
one and a raw-zero/base-two versus explicit-two pair that proves scoped
base-extruder fallback. Alignment pairs keep one transform fixed and toggle
alignment. A pair necessarily consumed by an earlier stage records the
expected M delta instead of requiring equality. Include the anti-map pair, all
four roles, bridge branches, spiral, alignment, transform and generator
dispatch. Then register the committed KSR M input and complete oracle N frame,
including 460 layers, one region, complete
multi-surface collections, exact adjacency, representative first/later Flow
bits, record count, repeatability and EOF.

GREEN wires `prepare_post_perimeter_inputs` immediately after Task 22M in
public orchestration and destructures it only at the existing incomplete sink.
No public method exposes the checkpoint. Public native KSR slicing must still
return `ProjectSlicingIncomplete` after reaching N.

Focused gates:

```text
cargo nextest run -p ares-core task22n
cargo nextest run -p ares-core task22m
cargo nextest run -p ares-core task22
```

## Package 5: feature transition and real browser

Allowed paths are `crates/ares-core/Cargo.toml`,
`crates/ares-core/src/lib.rs`, `crates/ares-core/src/project_slice.rs`,
`crates/ares-core/src/project_slice/checkpoints.rs`,
`crates/ares-wasm/Cargo.toml`, `crates/ares-wasm/src/lib.rs`, the Tier-1
workflow, deletion of the M vector module, addition of the N vector module, and
the existing browser server/page/spec files in the manifest.

Replace `task22m-browser-oracle` with `task22n-browser-oracle`; retain no alias.
Default exports contain no Task 22 hook. The feature build exposes exactly
`task22nBrowserInputOracle` and `task22nBrowserOracle`. Native M helpers remain
available only under tests.

Browser RED/GREEN covers parser KATs before fixture fetch, exact default and
feature exports, complete KSR M/N frames, representative Flow/context fields,
every Option-only archive family, anti-map invariance, public incomplete
lifecycle, exact EOF and repeated identity. Run default and feature wasm32
checks, isolated optimized builds, pinned wasm-bindgen, generated-export audit,
Node syntax checks, locked install, and two fresh Playwright Chromium passes.

### Package 5 lint-expectation amendment (2026-07-21)

The N browser feature intentionally consumes `PerimeterInputRecord` and
`PostPerimeterInputPrintObject::as_parts` outside `cfg(test)`. Therefore the two
existing default-build `dead_code` expectations in
`crates/ares-core/src/project_slice/perimeters/types.rs` would become
unfulfilled expectations under `task22n-browser-oracle` and fail the required
all-features `-D warnings` gate. Package 5 may change only those two
`cfg_attr(not(test), expect(dead_code, ...))` conditions to apply when neither
tests nor `task22n-browser-oracle` are enabled. This path is already in the
fixed overall manifest; the amendment changes no runtime behavior, type,
visibility, oracle byte, Option case, or deferral. Re-run the Package 4 native
matrix after the edit, and include the amended file in the Package 5 review.

## Package 6: docs, complete matrix, six-axis review, and release

Only after Packages 1-5 receive read-only approval, update architecture and
roadmap with the exact implemented boundary, Flow rules, context ownership,
oracle and archive evidence, native/browser identities, continuing incomplete
status, corrected perimeter gap policy, explicit deferrals, and next fixed
Classic boundary. Do not edit or re-label frozen historical Task 22M docs.

Run at least:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo nextest run -p ares-core task22n
cargo nextest run -p ares-core task22m
cargo nextest run -p ares-core task22
cargo nextest run -p ares-core
cargo nextest run --workspace
```

Also run the complete default/feature WASM and two-pass real Chromium matrix.
Audit every changed Rust file below 400 LOC; real test modules; no source
splitting macro, new unsafe, broad lint allowance, pinning test, Git/Orca
runtime inspection, fixture/reference branch, outside-path change, stale M
browser feature/export/vector, generated staging, or changed fixture hash; and
run `git diff --check`.

Freeze one exact sorted content frame. Dispatch one dedicated read-only
reviewer with six required sections:

1. requirement completeness;
2. fixed-source logical correctness;
3. boundary and edge cases;
4. code quality and module structure;
5. test coverage and oracle independence;
6. actual native, WASM and browser execution.

The reviewer returns P0-P3 findings, concrete repairs, and APPROVE/REJECT
without editing. The main thread fixes every finding, reruns focused and full
gates, freezes a new frame, and sends it back to the same reviewer. Repeat
until all six lists are empty and the verdict is APPROVE. Then obtain fresh
independent fixed-source, quality, anti-hardcoding/default-model and docs
approvals on the unchanged frame.

Stage exactly the approved manifest, excluding ignored/generated/user files.
Create one Conventional Commit, push normally without force, verify local,
tracking and direct remote SHAs agree, and monitor the exact-SHA Tier-1 run
through format, Ubuntu, Windows, macOS and WASM/browser. Any failure reopens
repair, complete verification and review before another push. Begin Task 22O,
the complete KSR-reached Classic generator, only after Tier-1 is green.

## Six-axis review repair amendment (2026-07-21)

The first final-review frame was rejected with two P1, two P2, and one P3
finding. This amendment supersedes the earlier deferral at lines 179-180 of
the canonical increase branch and authorizes one additional existing test
path:

- `crates/ares-core/src/project_slice/tests/compensation/fixture.rs`.

No production or test implementation may be edited for this repair until two
independent read-only reviewers approve the exact amended spec/plan frame.

### Repair 1: RED the complete fixed Flow behavior

Extend the ignored fixed-commit C++ `/fp:precise` probe first. Freeze the exact
canonical increase-else result for height `0x4113a9f3`, nozzle `0x4253561c`,
width `0x440415d2`, spacing `0x44039711`, area `0x4597ce34`, and
`bridge_flow=1.0000001`: returned width `0x440415d1`, spacing `0x44039710`,
volume f32 `0x4597ce33`, and stored f64 volume `0x40b2f9c660000000`.
Also freeze fixed `mm3_per_mm()` failure for thick and nonthick
`bridge_flow=f64::MIN_POSITIVE`. Regenerate the tracked synthetic oracle and
browser expectations only if their covered records change.

Add RED coverage without a new module:

- `tests/perimeters/flow.rs`: pure exact-bit canonical increase-else;
- `tests/perimeters/fixture/flow_pairs/bridges.rs`: a separate native real-3MF
  canonical increase-else reducer, reusing the already authorized
  `fixture/archive.rs` builder if needed. The compared archives must differ by
  exactly one semantic `bridge_flow` replacement from `1` to `1.0000001`, keep
  M bytes identical, produce two populated N slots, expose the exact repaired
  overhang Flow bits, and make public `slice_project` return
  `ProjectSlicingIncomplete` on both sides without panic. This is an edge
  reducer, not a twentieth Flow Option family;
- `tests/compensation/flow.rs`: spacing-valid, volume-underflow Task 22M Flow
  remains `Ok` with width/height `0x0da24260` and spacing `0x0d7ee054`;
- `tests/compensation/fixture.rs`: real 3MF/public M regression with a closed
  `1e-30`-high prism;
- `tests/perimeters/preflight.rs`: the same ordinary Flow fails only at N with
  the existing invalid-volume error, and tiny-positive thick/nonthick
  overhangs fail transactionally as `invalid Orca option bridge_flow`; and
- existing browser page/spec/vector paths: a generated real-archive reducer
  proves the exact increase-else bits, unchanged M, two populated N slots,
  public incomplete result without trap, plus tiny-positive error behavior.

The browser increase reducer replaces all synthetic model top `z="0.4"`
values with `z="18.5"`, sets both layer heights to `9.2289915`, both nozzles to
`52.83409`, all five reached ordinary widths to `1000%`, bridge width to zero,
thick mode false, and changes only `bridge_flow` from `1` to `1.0000001` between
the compared archives.

Run focused REDs and record the expected panic/current predecessor regression
before production changes. Tests may be added to existing test functions when
that keeps semantic ownership clear; exact test counts in architecture and
roadmap must match the final nextest inventory.

### Repair 2: implement the source boundary, not a fallback

In `perimeters/flow.rs`, replace the increase assertion with fixed
`Flow.cpp:181-183`: when `area_new / height` is not greater than spacing, pass
`rounded_width(area / height, height)` through the existing canonical
width/spacing constructor. Preserve f32 operation order and the fixed use of
old `area`, not `area_new`.

Remove eager volume rejection from the shared ordinary constructor so Task
22M again accepts finite positive spacing even when cached f32 volume is zero.
At `resolve_perimeter_flows`, validate final role volumes before returning an N
record. Preserve `invalid external perimeter flow volume` for the ordinary
underflow reducer. Validate the final thick/nonthick overhang result at the N
boundary and attribute zero/nonfinite derived volume to
`invalid Orca option bridge_flow`. Do not add checks inside trusted private
geometry helpers or change KSR-normal bits.

### Repair 3: remove review-proven structure defects

Delete all four `#[rustfmt::skip]` attributes newly added to
`tests/perimeters/flow.rs`. Rewrite only its expected-data representation so
normal rustfmt leaves the file below 400 LOC; do not add a path, include macro,
generated Rust source, or broad lint allowance. Rename Task 22N role masks and
oracle constants in Rust and browser tables to explicit internal-perimeter,
external-perimeter, overhang, solid-infill, nozzle, and percent meanings. Do
not introduce a new abstraction for the rename.

### Repair 4: verification and same-reviewer loop

After GREEN, rerun the complete Package 6 native, all-feature, WASM, optimized
binding/export, Node, and two-pass Chromium matrix. Reconfirm the fixed KSR
fixture hash, exact M/N identities, oracle repeatability, default no-hook
exports, 19 Flow and six context Option-pair inventories, all changed Rust
files below 400 LOC after rustfmt, and exact planned-path membership. The new
compensation fixture path is authorized only for the predecessor regression.

Freeze a new ordinal-sorted UTF-8/LF content frame and return it to the same
dedicated six-axis reviewer. Fix every remaining P0-P3 finding and repeat until
APPROVE. Only then obtain the fresh fixed-source, quality,
anti-hardcoding/default-model, and docs approvals required by the original
release section.

## Second six-axis review repair amendment (2026-07-21)

The first repaired frame was rejected with two P1 findings: a fixed-release
decrease-rounding input can panic at Rust's unconditional intermediate-width
assertion, and the plan still presented the pre-repair synthetic identity as
current. This amendment explicitly supersedes that identity and authorizes the
following additional paths:

- `crates/ares-core/src/project_slice/tests/perimeters/flow_edges.rs`;
- `crates/ares-wasm/tests/browser/task22n-edge-vectors.mjs`.

No production, test, oracle, or browser implementation for this repair may be
edited until two independent read-only reviewers approve the exact amended
spec/plan frame.

### Repair 5: RED fixed release decrease-rounding

First add an oracle self-check for the `/DNDEBUG` case with nozzle `100`,
initial and inner-wall widths `500%`, layer and first-layer height `2e-7`,
`bridge_line_width=0`, nonthick mode, and
`bridge_flow=f64::MIN_POSITIVE`. It must require `with_cross_section` to return
the zero Flow and require the subsequent `mm3_per_mm()` call to throw the fixed
negative-flow error; record RED while the hand-written intermediate-width
throw still masks that behavior. Then delete only that hand-written throw, run
the oracle self-check GREEN, and freeze the f32 predecessor fields: width
`0x43fa0000`, height `0x3456bf95`, area `0x38d1b718`, `area_new=0`, intermediate
width `0xb8000000`, and fixed release rejection when the final zero Flow volume
is consumed.

After the oracle is GREEN, add focused Rust and browser REDs before production
changes:

- `tests/perimeters/flow_edges.rs` calls the in-memory Flow resolution boundary
  directly and requires `invalid Orca option bridge_flow` without panic;
- `tests/perimeters/fixture/flow_pairs/bridges.rs` builds the real 3MF reducer,
  proves its settings are the stated single case, preserves M bytes, returns
  the exact error transactionally from public `slice_project`, and does not
  panic;
- `task22n-edge-vectors.mjs`, the existing browser page, and the existing
  Playwright spec generate the same archive from Option values and require the
  exact error without a WASM trap.

The new modules exist only because both existing vector-owning files are at
399 LOC. They must stay below 400 LOC, use normal Rust module / JavaScript import
boundaries, and may not use source-splitting include macros.

### Repair 6: match release behavior and refreeze evidence

Delete only the unconditional decrease-branch `assert!(width > 0.0)` in
`perimeters/flow.rs`. Preserve fixed f32 operation order and all other branch
behavior. Do not clamp the negative intermediate width, add a private defensive
check, or introduce a fallback; fixed release continues to the zero-diameter
Flow, and the existing Task 22N overhang volume boundary owns the
`invalid Orca option bridge_flow` error.

The release-rounding case is an independent expected-error self-check. Do not
add it to the synthetic wire's successful object list and do not invent an
error-record schema. After GREEN, regenerate the complete 25-object independent
success aggregate twice and require it to remain exactly 23,747 bytes / SHA-256
`82ccfa1db8bcfea1c4689147561be8c7058c6fdefe0df9b7b8ad127e99487fd1`.
Prove the two existing mutants remain distinct and reconfirm exact KSR M/N
identity. Reconfirm this success-aggregate identity and object count in the
Rust parser KAT, oracle README, architecture, and roadmap; update only test
counts that actually change. The 23,071-byte pre-repair aggregate remains
historical evidence, not a current gate.

Rerun affected tests, full Task 22M/22N/22, dynamic-value audit, full core and
workspace nextest, fmt, diff-check, workspace all-feature clippy, four WASM
checks, optimized default/feature export audits, Node syntax, and two fresh
Chromium passes. Freeze a new exact content frame, obtain two fresh independent
read-only spec/plan approvals for the final identity, and return the unchanged
frame to the same six-axis reviewer. Repeat on any P0-P3 finding.
