# Task 22O.48 — thick solid-infill bridge Flow

## Status

Implemented, gate-verified, and independently approved. Seven focused tests
pass and the O47 KSR composition retains its 115-path / 5,641-point ordered
digest. The repair/re-review loop found no remaining findings.

## Goal and upstream boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`LayerRegion::bridging_flow(frSolidInfill, true)` behavior from
`LayerRegion.cpp:31-61`.

Direct dependencies are `PrintRegion.cpp:7-22` for role selector ownership,
`Config.hpp:624-628,1284-1286` for vector element-zero fallback and f64
float-or-percent evaluation, `Flow.hpp:14,49-115` for the extra-spacing
constant and bridge Flow fields, `Flow.cpp:213-229` for spacing and circular
volume, and
`PrintObject.cpp:2795,3154-3157,3214` for the reached bridge-over-infill
callers.

The Rust seam is:

```rust
pub(in crate::project_slice) fn resolve_thick_solid_infill_bridge_flow(
    region: &RegionOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError>;
```

It uses the existing project-slicing Flow model and typed effective 3MF
options. It does not create a prepared successor.

## Required behavior

1. Select the nozzle through `internal_solid_filament_id - 1`; selector zero,
   negative underflow, or an out-of-range index falls back to element zero.
2. Convert the selected nozzle to f32 before later arithmetic and reject a
   missing, nonfinite, or nonpositive selected nozzle through the existing
   stable nozzle option error.
3. Evaluate `bridge_line_width` as f64: absolute values remain f64 and percent
   values multiply the selected f32 nozzle promoted to f64. A positive width is
   cast to f32; a nonpositive width uses the selected nozzle f32.
4. Read `bridge_flow` as f64. When positive, compute `sqrt` in f64, cast that
   result to f32, then multiply the f32 thread diameter. Preserve the source's
   no-multiply branch for nonpositive values; raw project validation remains
   the public invalid-option boundary.
5. Return one circular bridge Flow with `width == height == diameter`,
   `spacing == f32(f64(diameter) + 0.05)`, the selected nozzle, and
   `bridge == true`.
6. Compute volume as
   `f64(f32(f64(diameter * diameter) * 0.25 * PI))`, matching the existing Flow
   helper and source f32 storage.
7. Return the existing bridge-flow error if the derived volume is nonfinite or
   nonpositive. Do not catch, substitute a nonbridging Flow, or clamp.
8. Borrow options/nozzles and leave every input bit unchanged.

## Included and deferred behavior

Included: only `frSolidInfill, thick_bridge=true`, plus private factoring of the
same thick branch already used by Ares overhang Flow.

Deferred: non-thick bridge Flow, all other role dispatch beyond existing Ares
callers, layer-dependent nonbridge Flow, internal-bridge flow multipliers,
clustering, O47 production composition, anchored polygon construction, surface
commit, extrusion, motion, G-code, and CLI activation.

## Acceptance

Use TDD in a separate `project_slice/tests/perimeters/thick_bridge_flow.rs`
module. Focused tests must freeze exact field bits for default KSR options,
absolute/percent/zero width, selected and fallback nozzle indices, ratio cast
order, spacing, circular volume, invalid nozzle/derived flow, repeatability,
and complete input nonmutation.

The real-KSR test must resolve the Flow from embedded options, assert exact
field bits, and feed its height into O47's caller-side first `0.9_f32` factor;
the existing 18-layer flat geometry digest must remain unchanged.

All changed Rust files remain below 400 LOC and use no source-splitting include
macros. Final gates are focused/dependency/workspace Nextest, rustfmt,
warning-denying workspace Clippy, wasm32, diff/LOC/static audits, and the same
independent six-axis repair/re-review loop used by O47.
