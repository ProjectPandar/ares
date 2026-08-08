# Task 22O.34 — Compose source wave expansion and ExPolygon merge

## Status and source boundary

Released as implementation/documentation commits `f499058`/`25460c2`.
Exact-SHA Tier-1 run `31259140846` passed format, WASM/browser twice, Linux,
Windows, and macOS at
`25460c2abfc5bf94104f41b05df5af2dfac419ee`. Exact predecessor O33 was
released as implementation/documentation commits `b9e65fd`/`0f6f801`.
Pinned Orca remains v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only `Algorithm/RegionExpansion.hpp:113` and
`Algorithm/RegionExpansion.cpp:589-594`: `expand_merge_expolygons`. This is the
thin composition boundary that propagates source waves with an existing
`RegionExpansionParameters` value, then consumes the original sources and the
complete propagated records through O33's merge helper.

Deferred: `LayerRegion`/`PrintObject` external-surface orchestration and every
Option, lifecycle, checkpoint, cancellation, persistence, CLI/WASM/browser
export, fill, toolpath, seam, motion, serialization, G-code, post-processing,
and normalized KSR parity boundary.

## Frozen crate-private API

Add only:

```rust
pub(crate) fn expand_merge_expolygons(
    src: Vec<ExPolygon>,
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError>;
```

The explicit `CoordinateScale` is Ares' replacement for Orca's mutable global
scale. It must be forwarded unchanged both to O29 source discovery/propagation
and O33's AABB fallback. No scalar overload, builder call, generic adapter,
public export, or alternate error type is allowed.

## Frozen semantics

The implementation is exactly one ordered composition:

```rust
let expanded = propagate_waves_from_sources(&src, boundary, params, scale)?;
merge_expansions_into_expolygons(src, expanded, scale)
```

Requirements:

1. Borrow the complete `src` slice for exactly one O29 parameter-entry call.
2. Forward `boundary`, `params`, and the same `scale` unchanged.
3. Complete propagation through `?` before moving `src` into O33.
4. Call O33 exactly once with the original source allocation, the complete
   ordered propagation output, and the same scale.
5. Return O33's output/error directly.
6. Do not build parameters, discover seeds directly, call O27/O28/O30/O31/O32,
   rescale, sort/regroup, union, clone, retry, fallback, map errors, emit partial
   output, or add an empty-input shortcut.
7. Preserve error order: discovery and propagation errors escape before O33 can
   inspect source contours or source IDs; O33 safety-offset errors then escape
   directly.
8. Preserve trusted internal panics. This composition adds no validation for
   malformed geometry or identifiers.

Because `propagate_waves_from_sources` sorts discovery by boundary/source and
O33 re-sorts by source using the fixed-MSVC index permutation, O34 must not add
another ordering step. Parameter construction assertions belong to callers and
are not repeated here.

## Tests and TDD

Add one ordinary
`crates/ares-core/src/geometry/tests/region_expansion/expand_merge.rs` module
registered from the existing RegionExpansion test root. Keep it at most 300
physical lines.

The archived compiling chronological RED against the temporary
`Ok(Vec::new())` stub ran five tests and reported 0/5, but only four failures
were attributable to the O34 stub. The deleted fifth coordinate-error witness
failed first in its direct O29 setup assertion and never called O34. Preserve
`/tmp/task22o34-red-focused.txt` unchanged as historical evidence; do not call
all five failures meaningful.

The replacement successful non-empty handoff witness was added after the
literal body and has no chronological RED. Its later failures/passes are
post-hoc recurrence/GREEN evidence only, never reconstructed TDD chronology.
The four genuine stub failures still establish compiling RED for empty/
ownership, natural/manual parity, discovery precedence, and propagation
precedence before body authorization.

Tests use behavior-named Rust literals, never source text/hash/line pinning or
serialized oracle payloads, and cover:

- empty sources and no-expansion sources, including preserved source topology,
  order, and point-buffer ownership;
- a complete non-empty natural expansion/merge vector;
- equality with the explicit O29-then-O33 pipeline, including output point and
  hole order;
- the same complete vector and explicit-pipeline parity under Normal and
  LargeBed scales. These checks freeze scale-sensitive behavior where the
  geometry exposes it; unchanged forwarding to both calls is fixed separately
  by the literal two-call body and structural/diff audit;
- discovery error precedence over an O33 trusted empty-contour panic;
- propagation error precedence over later merge work;
- a successful non-empty O29 handoff into O33, cross-checked against the
  explicit pipeline;
- function-pointer shape and crate-private facade visibility.

O29's discovery/propagation Clipper checks reject near-range geometry before it
can become an O33 safety-offset input, so O34 does not require an artificial
"successful O29 then O33 coordinate error" witness. Direct O33 result/error
forwarding is fixed by the literal tail call and structural audit; an O33-error
swallowing mutation that is unreachable through valid O29 output must be
reported as a truthful survivor, not manufactured through a production seam.

Post-hoc mutations are recorded separately from the chronological RED. Kill at
least dependency omission, argument substitution, source replacement, scale
substitution where behaviorally observable, error swallowing, and early-empty
mutations. A behaviorally equivalent hard-coded scale must be reported as a
truthful survivor; no production injection seam or instrumentation may be added
solely to manufacture observability. Compiler-rejected signature mutations and
other truthful equivalent survivors remain separate.

## Files, limits, and prohibitions

Allowed Rust edits only, with this exact destination boundary:

- place the sole production body immediately after O33 in
  `crates/ares-core/src/geometry/region_expansion/merge.rs`;
- add only the crate-private reexport and an `ExpandMergeFn` function-shape
  assertion beside the existing O27-O33 assertions in
  `crates/ares-core/src/geometry/region_expansion.rs`;
- add only the existing-facade-pattern crate-private reexport and matching
  function-shape assertion in `crates/ares-core/src/geometry.rs`;
- register one ordinary `mod expand_merge;` in
  `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- add the tests and their local `ExpandMergeFn` alias only in new
  `crates/ares-core/src/geometry/tests/region_expansion/expand_merge.rs`.

Do not place or duplicate the body, reexports, assertions, or tests elsewhere.

Allowed docs: this spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O33 spec/plan release-state
corrections. No ARD change.

Every Rust file remains below 400 physical lines; the new test shard is at most
300. No manifest/lock/dependency change, lint allowance, broad expectation,
`unsafe`, FFI, filesystem/native thread, platform branch, `include!`,
`include_bytes!`, source concatenation, fixture identity/name/hash/layer-count/
geometry branch, reference-G-code access, binary oracle, public hook, legacy
fallback, or source text/hash/line pinning test.

## Verification, review, release, rollback

The exact literal body and five-test shard are present in the five-file Rust
allowlist. The historical stub run reported 0/5 with the four genuine failures
and one setup defect described above. After replacing the defective witness,
focused debug/release pass 5/5 and complete RegionExpansion passes 92/92. Six
runtime mutations are killed, one signature mutation is compiler-rejected, and
two behaviorally equivalent scale substitutions plus the valid-O29-unreachable
O33-error swallowing mutation are recorded as truthful survivors. Exact
post-mutation restoration, rustfmt, diff check, LOC, visibility, staging, and
forbidden-pattern audits pass. The default-model OpenCode initial implementation
review approved, but independent review required the sole body to follow O33
physically and required non-vacuous multiple-source/multiple-hole ordering and
ownership evidence. Both repairs are present and verified. The repaired exact
candidate passes focused debug/release 5/5, O29 5/5, O33 13/13,
RegionExpansion 92/92, PolyTree 6/6, offset 58/58, O26 lifecycle 3/3, and
workspace 6,033/6,033 with 2 skipped. All-target check, warning-denying Clippy,
rustfmt, four WASM checks, two optimized builds, export and JavaScript audits
pass. Both local Playwright attempts reach all 11 launches but stop before test
code because Chromium cannot load `libglib-2.0.so.0`; pushed Tier-1 installed
browser dependencies and passed both runs. Disposable exact-O33 rollback proves
candidate/primary byte identity and passes RegionExpansion 87/87, PolyTree 6/6,
offset 58/58, and lifecycle 3/3. The repaired exact candidate's independent
six-dimensional and default-model OpenCode rereviews both return literal
`VERDICT: APPROVE`. Implementation/documentation commits `f499058`/`25460c2`
were pushed; exact-SHA Tier-1 run `31259140846` passed all five jobs, including
both browser runs, at `25460c2abfc5bf94104f41b05df5af2dfac419ee`. O34 is
released.

Require focused debug/release, complete RegionExpansion, O29/O33 focused,
PolyTree/offset, O26 lifecycle, workspace Nextest, all-target check,
warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export/JavaScript audit, two Playwright runs, exact allowlist/LOC/visibility and
forbidden audits, and disposable exact-O33 rollback.

Fresh independent six-dimensional and default-model OpenCode reviewers must
both return literal `VERDICT: APPROVE`. Any requested repair requires affected
and complete exact-candidate verification, refreshed evidence, and both reviews
again. O34 is released only after commit/push and a Tier-1 run whose `headSha`
equals the pushed documentation SHA and whose five jobs, including both browser
runs, pass.

Public slicing must still consume O26 and return `ProjectSlicingIncomplete`;
the KSR golden test remains unchanged and incomplete. The next source boundary
must be separately reconnoitered and reviewed from pinned external-surface
callers; O34 itself does not activate lifecycle behavior.
