# Task 22O.72 implementation plan

1. Independently audit pinned `PrintObject.cpp:673-680,4163-4287`, prove the
   optional pass at `3393-3546` is unreachable in the admitted domain, and
   approve the identity/gate ADR and exact successor seam.
2. Add real-KSR REDs proving O72 runs after O71 and that its successor preserves
   the full O71 ordered topology/metadata digest without copying the graph.
3. Add focused REDs for disabled/nonzero identity, enabled/exact-zero identity,
   and enabled/nonzero `UnsupportedProjectFeature("infill_combination")` through
   global, object, and part effective-option materialization.
   Drive exact zero through the public API with the real O43 candidate map; if
   O71 rejects before O72, port the source zero-density empty-anchor result
   without clearing candidates or bypassing the bridge transaction.
   Freeze `PrintObject.cpp:3701-3706` by proving raw decimal `0.00011%`
   normalizes to zero under the source f32 literal and the next representable
   promoted value remains nonzero.
4. Implement
   `prepare_infill::combine_infill::{prepare, dispose, PreparedPostInfillCombination}`
   over the consumed O71 predecessor. Inspect already typed region options,
   prove every region inactive before returning, and dispose the input on the
   first active region without mutation or partial output.
5. Move the public sink to `consume_post_infill_combination`; verify one
   invocation, one disposal, error precedence, ownership cleanup, and
   repeatability while keeping `ProjectSlicingIncomplete` as the only admitted
   KSR public result.
6. Prove the module has no dependency on the legacy `infills::combination`
   scaffold or `InfillOptions`; kill and byte-exactly restore compiling
   mutations of both gate operands, exact-zero equality, and error behavior.
7. Run focused/dependency/workspace Nextest, strict Clippy/rustfmt, six Tier-1
   builds, diff/LOC/static/no-staged and clean-Orca gates, then independent
   six-axis review; repair only in the main thread and repeat review to
   unconditional approval.

The active `PrintObject::combine_infill` body at `4176-4287`, optional second
internal-bridge behavior, fill grouping, extrusion, motion, G-code, CLI, and
complete golden parity remain deferred. O73 next ports pinned
`Fill/Fill.cpp:216-346,829-1067,1213-1224`; KSR-active narrow-solid behavior at
`349-827,1152-1186` must follow before grouped-fill lifecycle activation.

Final execution passed focused 14/14, prepare-infill 255/255, and workspace
6,486/6,486 with two configured skips. Six compiling mutations, including the
promoted source f32 density threshold, were killed and byte-exactly restored;
strict Clippy/rustfmt, six Tier-1 target checks, LOC/static/diff gates, and a
clean pinned Orca worktree also passed.
