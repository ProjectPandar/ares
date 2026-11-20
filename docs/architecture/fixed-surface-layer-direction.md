# Fixed top and bottom fill directions

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Fill/Fill.cpp:1036–1064` (`group_fills`), selects the angle before
flow and fill grouping. `Surface.hpp:113–114` defines top as `stTop` and bottom
as `stBottom` or `stBottomBridge`.

- Outside the sparse-infill branch, a matching top or bottom setting `>= 0`
  (including zero) takes precedence over `solid_infill_direction` and
  `solid_infill_rotate_template`. Upstream converts the setting from degrees
  to radians and sets `params.fixed_angle = true` without evaluating the
  template. Model alignment, if enabled, still adds its offset afterward.
- When the matching setting is `< 0` (default `-1`), or the surface is neither
  top nor bottom, upstream calls `calculate_infill_rotation_angle` with the
  ordinary solid angle and template and sets `fixed_angle` to
  `!solid_infill_rotate_template.empty()`. Ares already has this branch and the
  separate sparse angle/template branch, but does not yet select the fixed
  surface angle.

`Fill.cpp:343,380` includes the flag in fill grouping. Both extrusion fill
(`:1384–1385`) and the eligible anchoring fill path (`:1561–1690`, assignment at
`:1617–1618`) forward the chosen angle and flag to `Fill`. The shared direction
calculation (`FillBase.cpp:447–466`) still honors an explicit bridge angle
first; otherwise the flag suppresses alternating rotation without changing the
reference point or the final quarter-turn. This does not admit top/bottom
surfaces to sparse anchoring or change anchor lengths.

## Rust destination

Move the already parsed settings from the source-only declarations in
`crates/ares-core/src/options/process_options/region_source.rs` to the shared
`options/region_fields.rs` so resolved regions carry them, as in upstream
`PrintConfig.hpp:1480–1481`. Consume them in
`project_slice/group_fills/params/projection/rotation.rs`. Pass the surface kind
from `projection.rs`; the emitted extrusion role alone cannot distinguish
external bottom bridges from internal bridges. Keep the shared angle conversion,
model alignment, and the existing grouping, fill and anchoring consumers. The
existing projection is the compatibility shell for `group_fills`; no schema or
new pipeline is needed.

## Verification

Use a synthetic prism with stock parent controls and separate 23-degree top
and bottom settings on MK4, plus the same bottom setting on K1. Compare Native
Orca to the public slice API and CLI in official and seeded modes, over the
entire G-code file with only the generator line normalized. Also compare the
existing infill, fill, surface, ironing, bridge and solid parity subsets before
and after the change.

## Deferred

The separate ironing calculation at `Fill.cpp:1799–1806` is not part of
`group_fills` and the fixed-direction witnesses do not enable ironing. Sparse
Cross Zag and Locked Zag anchoring endpoints, PlanePath endpoints, and remaining
Adaptive Cubic and Support Cubic differences are also outside this slice.
