# Task 22O.72 architecture decision record

## Status

Accepted and implemented.

## Upstream boundary

Port the admitted branch of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:673-680` — call
  `bridge_over_infill()` before `combine_infill()` and cross the combination
  boundary before the next lifecycle stage;
- `src/libslic3r/PrintObject.cpp:4163-4174` — visit printing regions and leave
  a region unchanged when combination is disabled or its sparse density is
  exactly zero; and
- `src/libslic3r/PrintObject.cpp:4176-4287` — the active combination body whose
  behavior defines the capability gate but remains deferred.
- `src/libslic3r/PrintObject.cpp:3701-3706` — normalize sparse density against
  the source `0.00011f` literal before anchoring and combination.

The public exact-zero RED also exposed the directly preceding anchor-generation
case. `PrintObject.cpp:2509-2555,2737-2753` retains bridge candidates and asks
the lower layer for sparse anchor lines, while `Fill/Fill.cpp:855-902,1394-1508`
omits zero-density Internal groups and therefore returns no sparse lines. O71
now projects that result as an empty anchor vector and continues its existing
boundary-angle path; it does not clear candidates or skip the bridge transaction.

`PrintConfig.cpp:3973-3980,4092-4104` defines the Boolean activator and the
active body's maximum-height option, including their `false` and `100%`
defaults. `Print.hpp:529-532` owns the two ordered operations.

The optional second internal-bridge pass at `PrintObject.cpp:3393-3546` is not
the next admitted operation: its `InternalBridgeOnly` and `ApplyToAll` modes are
already rejected by O71, while O17 rejects `ExternalBridgeOnly` and
`ApplyToAll`. Therefore every currently admitted object has that pass disabled,
and O72 must not manufacture a no-op lifecycle module for it.

## Rust destination seam

Add the successor module
`project_slice/prepare_infill/combine_infill.rs`:

```rust
pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeOverInfill,
) -> Result<PreparedPostInfillCombination, SliceError>;

pub(in crate::project_slice) fn dispose(
    prepared: PreparedPostInfillCombination,
);
```

`PreparedPostInfillCombination` owns the complete O71 predecessor. Public
slicing calls `prepare_infill::combine_infill::prepare` exactly once after O71,
then passes its successor to `consume_post_infill_combination`. The sink disposes
that successor and continues to return `ProjectSlicingIncomplete`.

## Decision

O72 is the exact identity/capability boundary at the source call, not an active
combination implementation. For every retained region, `prepare` evaluates the
source condition without epsilon or sign substitution:

```text
infill_combination == false || sparse_infill_density == 0.0  => identity
infill_combination == true  && sparse_infill_density != 0.0  => unsupported
```

All regions must be proven inactive before a successor is returned. An active
region returns `UnsupportedProjectFeature("infill_combination")`, consumes and
disposes the O71 predecessor, and exposes no partial successor. The identity
path neither mutates nor copies prepared surfaces; it only transfers ownership
into the new state.

This boundary is required because the current typed public domain admits
enabled, nonzero-density combination. Skipping the call and advancing directly
to downstream fill grouping would silently accept a project whose pinned Orca
surface graph is changed by `combine_infill()`.

The old `crates/ares-core/src/infills/combination.rs` path-level scaffold and
its `InfillOptions` projection are not reused, wrapped, or treated as a fallback.
They do not implement the pinned `SurfaceCollection` intersection, clearance,
and rewrite owner at `PrintObject.cpp:4163-4287`. A future exact active-body
slice may deepen this same module without changing the lifecycle seam.

## Included and deferred

Included: the exact disabled-or-zero identity condition; an explicit gate for
every enabled nonzero-density region; ownership, disposal, and public lifecycle
activation after O71; deterministic inspection of the already aligned object
and region state; the source-equivalent zero-density empty-anchor projection;
and the unchanged real-KSR prepared-surface checkpoint.

Deferred: active-body surface-kind selection, nozzle and maximum-height
resolution, layer scheduling, multi-layer intersection, area filtering,
clearance, and surface rewrites at `PrintObject.cpp:4176-4287`; the unreachable
second internal-bridge pass at `3393-3546`; debug exports, status, cancellation,
and TBB adapters; fill grouping and narrow-solid splitting; extrusion, motion,
G-code, CLI, and complete golden parity.

The gate is an upstream-rewrite staging constraint, not an Ares-owned
combination design. O72 adds no filesystem, terminal, UI, OpenGL, unsafe, or
native-thread dependency.

## Verification contract

The real KSR fixture has `infill_combination = 0`,
`infill_combination_max_layer_height = 100%`, and
`sparse_infill_density = 15%`. O72 must therefore return a successor whose full
ordered topology and metadata equal O71 exactly: 47 `InternalBridge` surfaces,
15,689 points on the same 17 bridge-bearing planned layers, and SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.

Focused tests must also prove enabled plus exact-zero density is an accepted
identity, enabled plus nonzero density returns
`UnsupportedProjectFeature("infill_combination")`, disabled combination does
not resolve active-body operands, global/object/part overrides drive the
materialized region gate, input ownership is transferred or disposed exactly
once, no partial successor escapes, public slicing invokes O72 once, and
repeated runs are identical. Compiling mutations of both Boolean operands, the
exact-zero comparison, and the error branch must be killed and restored.

Exit also requires focused and dependency Nextest, full workspace Nextest,
strict Clippy/rustfmt, Tier-1 portability builds, diff/LOC/static/no-staged
checks, a clean pinned Orca worktree, and independent six-axis review followed
by unconditional re-review after any repair.

## Next upstream owner

After this gate, the first KSR-active downstream helper is pinned
`Fill/Fill.cpp:216-346,829-1067`: `SurfaceFillParams`, `SurfaceFill`, and the
base parameter projection, ordering, aggregation, and priority clipping in
`group_fills`, owned by the caller at `Fill/Fill.cpp:1213-1224`. That is O73;
neither the inactive second bridge pass nor the gated active combination body
should be represented by another placeholder stage.

KSR enables narrow internal-solid detection, so `Fill/Fill.cpp:349-827` and its
application at `1152-1186` must follow the base grouping helper before grouped
fills may activate a production lifecycle successor.

## Final evidence

The compiling RED first failed with unresolved imports for the absent O72
module. A later public enabled-plus-zero RED failed with O71's premature
`UnsupportedProjectFeature("sparse_infill_density")` rejection;
the repaired path retains a nonempty real O43 candidate map and commits a
nonempty InternalBridge result before reaching O72. Final verification passed
focused 14/14, prepare-infill dependency 255/255, and workspace 6,486/6,486
with two configured skips.

Six compiling mutations were killed and byte-exactly restored: dropping either
O72 gate operand, flipping the exact-zero comparison, changing the error key,
flipping O71's exact-zero anchor-line predicate, and replacing the promoted
source `0.00011f` normalization threshold with a decimal f64 literal. Strict
workspace Clippy, rustfmt, WASM core and adapter checks, both Windows targets,
both macOS targets, diff/static/no-staged scans, and the clean pinned Orca
worktree pass. Every changed or added Rust file remains below 400 LOC; the
maximum is 373 lines and the new O72 production shard is 70 lines.
