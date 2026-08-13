# Task 22O.72 — infill-combination identity gate

Port the admitted branch of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:673-680,3701-3706,4163-4287`. The source calls
`combine_infill()` immediately after `bridge_over_infill()` and leaves a region
unchanged exactly when `infill_combination` is false or
`sparse_infill_density == 0.0`. `PrintConfig.cpp:3973-3980,4092-4104` defines
the activator and active-body height option with `false` and `100%` defaults.
The retained density is first normalized against Orca's `0.00011f` literal;
Rust must use its promoted double value, not decimal f64 `0.00011`.

Destination seam:

```rust
pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeOverInfill,
) -> Result<PreparedPostInfillCombination, SliceError>;

pub(in crate::project_slice) fn dispose(
    prepared: PreparedPostInfillCombination,
);
```

Add this seam at `prepare_infill::combine_infill::{prepare, dispose}`. It must
consume O71, inspect every retained region's already typed effective options,
and return an ownership-only successor only after all regions satisfy the
source identity condition. If any region has combination enabled and nonzero
sparse density, dispose the predecessor and return
`UnsupportedProjectFeature("infill_combination")` before mutation or output.
Do not replace exact zero with an epsilon or positive-only test.

Public slicing must call O72 exactly once after O71 and pass the result to
`consume_post_infill_combination`; that sink disposes the successor and returns
`ProjectSlicingIncomplete`. The identity path must preserve the complete O71
surface graph without copying it.

Do not reuse `crates/ares-core/src/infills/combination.rs`, its path-level
algorithm, or the legacy `InfillOptions` projection. They are not an exact port
of the pinned `SurfaceCollection` owner. Do not add a lifecycle wrapper for the
optional second internal-bridge pass at `PrintObject.cpp:3393-3546`: O17 and
O71 already reject every mode that activates it.

Included: exact disabled and density-zero identity behavior; enabled,
nonzero-density capability rejection; successor ownership/disposal; public
lifecycle activation; source-equivalent empty sparse-anchor lines at zero
density without clearing O43 candidates; and unchanged KSR topology/metadata.

Deferred: the active combination body at `PrintObject.cpp:4176-4287`, including
surface selection, nozzle/max-height resolution, scheduling, intersections,
area filtering, clearance, and rewrites; the optional second internal-bridge
pass; debug,
status, cancellation, and parallel adapters; fill grouping, extrusion, motion,
G-code, CLI, and complete golden parity.

The real KSR behavioral RED must prove O72 is invoked after O71 and preserves
the exact O71 checkpoint: 47 `InternalBridge` surfaces, 15,689 ordered points,
17 bridge-bearing planned layers, and SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.
Focused tests must cover disabled/nonzero identity, enabled/zero identity,
enabled/nonzero rejection through global/object/part effective options, exact
error naming and precedence over the incomplete sink, one-time
ownership/disposal, no partial successor, and repeatability.

The exact-zero case must cross the public API with the real O43 candidate
inventory. O71 must still commit a nonempty InternalBridge result; tests must
not manufacture this branch by clearing candidates.

Final gates: focused/dependency/workspace Nextest, compiling mutations of the
gate, strict Clippy/rustfmt, WASM and x86_64/aarch64 Windows/macOS checks,
diff/LOC/static/no-staged checks, clean pinned Orca, and independent read-only
six-axis review followed by main-thread repair and unconditional re-review.

O73 is the next source owner: pinned
`Fill/Fill.cpp:216-346,829-1067,1213-1224` for `SurfaceFillParams`,
`SurfaceFill`, and base `group_fills`. Because the KSR fixture enables narrow
internal-solid detection, `Fill/Fill.cpp:349-827,1152-1186` must follow before
that grouped state becomes lifecycle-active.
