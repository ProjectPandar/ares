# adaptive_infill: prove the lookup guard unreachable, remove only that guard

## Decision requested

This slice removes an obsolete **internal-invariant** error branch, not an
option-admission restriction. The sole literal `adaptive_infill` guard is
`fill_entities/adaptive.rs:139`, and cannot be reached by a valid object
that has passed the preceding traversal stage. Native preflight proves the
adaptive configuration itself is reachable; it does **not** prove the
missing-resolved-object branch is reachable.

Base: `origin/main 6a6587f8f`. Production/tests unchanged at SPEC submission.

## Upstream ownership and Rust destination

- `OrcaSlicer/src/libslic3r/Fill/FillAdaptive.hpp:25-48`:
  opaque `Octree`, `OctreePtr`, `adaptive_fill_line_spacing`, transforms,
  and `build_octree(mesh, overhang_triangles, spacing, support_only)`.
- `FillAdaptive.cpp:276-357`, `adaptive_fill_line_spacing`: identifies
  nonempty adaptive/support regions across layers, averages density and
  extrusion width, and returns separate adaptive/support spacings.
- `FillAdaptive.cpp:1483-1513`, `build_octree`: constructs object-data
  octree from rotated mesh and internal overhang triangles.
- `PrintObject.cpp:972-1008`, `PrintObject::prepare_adaptive_infill_data`:
  the owning PrintObject directly supplies its model object, centered
  transform, layers, region-derived spacing and internal bridges; returns
  separate adaptive/support octrees, with the latter `support_only=true`.
- `PrintObject.cpp:2735`: prepares and retains these octrees during the
  bridge-over-infill stage; `:707-715` passes both trees to layer fills.

Rust destination remains
`crates/ares-core/src/project_slice/fill_entities/adaptive.rs`:
`octree_for` and `build_object_octree` connect the already ported
`crate::fill::adaptive::{octree,filler}` to the retained object traversal.
No new Ares slicing pipeline or crate is designed.

## Proof of unreachability in the current Rust ownership chain

1. `perimeters/types.rs:162-165`, the input object's `identity()`, comes
   from the retained plan's `(source_object_index, transform_index)`.
2. `perimeters/classic/traversal.rs:15-35`, `finish`, visits every predecessor
   object and looks up the **same** source identity in
   `prepared.resolved.objects`. It already requires membership with
   `expect("O5 object retains its resolved source")`.
3. `traversal.rs:38-59` moves that very `resolved` config and the same
   predecessor objects into `PreparedPostClassicTraversal`.
   `traversal/types.rs:9` owns this config; downstream adaptive consumers
   receive only immutable references, not mutation access to the list.
4. `fill_entities/adaptive.rs:123-139`, `build_object_octree`, follows the
   retained predecessor identity again and searches the retained resolved
   list. No intervening filtering or membership removal occurs.
5. `adaptive.rs:104-112`, `octree_for`, already invokes
   `build_object_octree(...).expect("resolved group")`. Thus the branch is
   not even an observable public `UnsupportedProjectFeature` rejection:
   an invariant violation would be converted to a panic immediately.

The earlier stage requires the same membership before adaptive append or
anchoring runs. A public input cannot remove it between these immutable
stages. Invalid inputs are handled at their existing input boundaries;
this slice adds no validation, fallback or speculative error handling.

## Included behavior and minimal edit

- Replace the missing lookup's `ok_or_else(UnsupportedProjectFeature(
  "adaptive_infill"))` with the owning-source invariant `expect`.
- `build_object_octree` becomes infallible (`octree::Octree`); remove only
  the now-unneeded `Ok(...)` wrapper and caller `.expect("resolved group")`.
- Keep mesh transforms, world center, line-spacing/cache keys, append and
  anchoring behavior byte-for-byte equivalent. Other errors/guards stay.
- Add a concise source/ownership citation to the changed boundary.
- Keep files <=400 LOC (current adaptive.rs is below the cap).

## Explicitly deferred scaffold gaps — not part of this invariant deletion

The current object adapter remains a temporary compatibility shell around
upstream PrintObject adaptive data preparation; this slice does not claim
complete `prepare_adaptive_infill_data` parity:

- It builds/cache-keys trees per fill line spacing rather than upstream's
  region-averaged spacing; defer `FillAdaptive.cpp:276-357` averaging.
- It always supplies `false` for support-only mode; defer separate support
  trees and upstream `PrintObject.cpp:1006-1007` dispatch.
- It supplies empty internal-overhang triangle inputs; defer upstream
  `PrintObject.cpp:986-1003` bridge triangulation and propagation.

These require independent source-cited specs and Native-backed RED cases.
Do not mix them with deletion of the missing-object invariant error branch,
remove other sparse pattern guards, or change region selection.

## Native-only preflight evidence (CPU18)

Root `${TMP_ROOT}/ares-adaptive-infill-preflight-VSCkZ0/` contains reproducible
input 3MFs, producer logs, exit-code files and output G-code. Inputs derive
from the retained overhang parity fixture, changing only sparse pattern,
20% density, and blank sparse/solid rotation templates. Export includes
real sparse infill, not just schema admission:

| producer | pattern | exit | Sparse infill blocks |
|---|---|---|---|
| official | adaptivecubic | 0 | 63 |
| official | supportcubic | 0 | 63 |
| seeded | adaptivecubic | 0 | 63 |
| seeded | supportcubic | 0 | 63 |

Commands: outer `taskset -c 18`, `--slice 0 --outputdir <fresh-dir>
<pattern.3mf>`; official `/tmp/orca-wrapper-inherit-cpu.sh` with explicit
`ORCA_APPDIR`; seeded `scripts/orca-seeded-parity.sh` with canonical replay
binary, seed0 and replayCPU18. START/END receipts recorded in inbox. No
Ares execution or parity success is claimed by this preflight. Native
outputs remain outside Git.

## Verification / RED boundary ruling requested

An E2E RED that triggers this missing lookup through legal public input
would contradict the ownership proof above. Do not manufacture a malformed
internal state, weaken assertions, or pretend Native pattern admission is
RED for an unreachable branch. Request the coordinator's invariant-removal
exception (W2 precedent): source proof plus unchanged public E2E behavior
before/after, rather than a false failing public guard case.

After APPROVED: baseline and candidate public slicing cases with printable
adaptive cubic objects (including nonprintable earlier source groups and
multiple instances to exercise retained source indices), official+seeded
strict Native comparison where established parity holds. No support/scaffold
behavior port is included. If baseline reveals a deferred scaffold mismatch,
preserve/report it for scope ruling; do not expand production or normalize
it away. Keep every existing assertion unless separately approved.

Then strict clippy/fmt/WASM, authorized serialized workspace and separately
requested exclusive canonical full. Record source proofs, exact artifact
paths, and residual risks. No worker main push command: coordinator alone
merges after approval.
