# Task 22O.71 architecture decision record

## Status

Accepted and implemented source-cited implementation boundary.

## Upstream boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:2725-2761` — generate the lower-layer sparse
  infill lines used as bridge anchors;
- `src/libslic3r/PrintObject.cpp:3114-3310` — expand candidate layers in
  cluster order and retain the completed candidate history; and
- `src/libslic3r/PrintObject.cpp:3315-3389` — rebuild and commit the first
  internal bridge layer.

This closes the already ported O43-O70 operations at their source owner,
`PrintObject::bridge_over_infill`, whose pipeline call is at
`PrintObject.cpp:673-680`.

## Rust destination seam

Add the deep successor module
`project_slice/prepare_infill/bridge_over_infill/transaction.rs`:

```rust
pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeCandidates,
) -> Result<PreparedPostBridgeOverInfill, SliceError>;
```

Its private ordinary submodules own candidate expansion and surface rewrite.
The existing O43 `bridge_over_infill::prepare` remains the temporary raw
candidate predecessor; callers after O71 observe only the mutated prepared
surface graph, not the candidate map.

## Decision

The transaction first generates O46 anchoring lines for every candidate's
lower layer while the O43 map is still raw, then runs O54 clustering. Within
each cluster it processes layers in ascending source order and carries the
O64-committed history into O56. Within a layer it preserves O55 candidate
order and carries the reduced O63 expansion area and accepted candidates into
the next candidate. Present-but-empty map keys remain present for the O65
current/upper scheduling rule.

Only after every candidate layer is complete does the transaction traverse
physical layers. O65 uses the completed current candidates and the upper
candidate's normal `solid_infill_flow`. Each region reads one unchanged
pre-commit surface collection for O66-O69, assembles results strictly as
O67 then O68 then O69, and performs one O70 commit.

The prepared input is consumed. Any option, flow, or geometry error disposes
the entire predecessor, so no partially rewritten successor escapes. Stable
`CandidateSource` indices replace Orca's pointer identity until the O70 commit;
the candidate map is then discarded.

The transaction executes deterministically in ascending object, cluster,
layer, candidate, and region order. Orca's TBB scheduling is not an observable
result and is deferred.

## Included and deferred

Included: the active single-region, non-Lightning CrossHatch path; exact O46
through O70 operation order; ordinary and thick-flow provenance; completed
candidate history; first internal bridge surface mutation; prepared lifecycle
activation; and explicit unsupported rejection instead of fallback when an
unported sparse anchoring pattern is active.

Deferred: Lightning's temporary surface/generator path at `2593-2723`, adaptive
and support-cubic octrees at `2728-2735`, generic multi-region and other sparse
infill generation, TBB/logging/timeout/debug adapters, the second and third
internal bridge layers at `3393+`, `combine_infill`, extrusion, motion, G-code,
CLI, and complete golden parity.

The existing single-region/CrossHatch limitations are upstream-rewrite staging
constraints, not Ares-owned pipeline alternatives or fallback behavior. The
module remains platform-neutral and introduces no filesystem, terminal, UI,
OpenGL, unsafe, or native-thread dependency.

## Verification contract

The behavioral RED is the real KSR fixture at planned layer 15: O43 has a raw
candidate there but the pre-O71 graph has no `InternalBridge`; successful O71
must create one with source metadata and final geometry while consuming the
candidate predecessor. Follow-up tests freeze candidate-key scheduling,
ordering, ownership/disposal, unsupported-pattern behavior, repeatability, and
the full committed KSR surface snapshot. Exit also requires workspace Nextest,
strict Clippy/rustfmt, Tier-1 portability builds, LOC/static checks, and an
independent six-axis review/repair loop.

## Verification evidence

The implemented transaction passes 16/16 focused O71 tests, 240/240
bridge-over-infill dependency tests, and 6,473/6,473 workspace tests with two
configured skips. Strict workspace Clippy with warnings denied, rustfmt,
`git diff --check`, WASM, x86_64/aarch64 Windows, and x86_64/aarch64 macOS
checks pass on the final tree.

The complete ordered KSR topology/metadata snapshot contains 47
`InternalBridge` surfaces and 15,689 points on planned layers
`[15, 30, 31, 41, 45, 60, 65, 70, 75, 82, 85, 90, 105, 116, 125, 136, 255]`,
with SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.
The independent pinned-Orca G-code witness identifies the same 17 bridge
layers across 30 `Internal Bridge` feature runs. Ares still returns
`ProjectSlicingIncomplete` after disposing the O71 successor, so this is a
prepared-surface checkpoint and not a complete G-code golden claim.

Every changed or added Rust file is below 400 LOC (maximum 372; maximum new
O71 shard 362); the O71 slice contains no source-splitting include macro,
TODO/FIXME marker, stale dead-code allowance, platform I/O, or staged file.
The pinned OrcaSlicer worktree is clean at
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Independent source, logic,
standards, spec, test, ownership, and lifecycle re-reviews all approved the
repaired final tree without a remaining item.
