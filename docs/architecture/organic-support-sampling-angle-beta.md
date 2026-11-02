# Organic support sampling angle: beta rewrite slice

## Decision and source boundary

Port only the roof-angle expression in OrcaSlicer beta revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Support/TreeSupport3D.cpp:749-756::generate_support_infill_lines`.
The 2.4.2 conditional-precedence expression selected `-pi/4` or `+pi/4`
from the truth value of `interface_angle + parity`. Beta instead adds
`interface_angle` to the parenthesized parity choice:

```cpp
support_params.interface_angle +
    ((layer_idx & 1) ? float(-M_PI_4) : float(+M_PI_4))
```

The Rust destination is
`support_material/tree/organic/infill_lines.rs::generate`. Replace its old
precedence expression and comment with the literal beta expression using
an f32 sum. Preserve the base angle, spacing, density, pattern dispatch,
polygon processing, and all downstream sampling. This module is a temporary
compatibility shell around the upstream function, not an independent support
pipeline. Do not normalize angles modulo pi or retain a legacy fallback.

## Qualification and public RED

Use the unchanged organic overhang fixture
`tests/parity/skirt-support-occupied-input/support-overhang.3mf`, SHA-256
`58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d`.
Initial overhang inputs are ordered-exact: two 8-vertex polygons at input
index 50, each with signed double area 253120000000000. Preparation and
observed roof-descent polygons also match. The index maps through the top
gap to support layer 48 and through the roof stack to seed layer 45.

At sampling layer 46, polygon input, interface angle, spacing, density and
`dont_adjust` match, but Native configures angle `2.356194496154785` while
Rust configures `-0.7853981852531433`. Native raw sampling-line outputs were
optimized out; this is not proof that the angle alone explains all tip-count
or support-path differences.

The baseline first positive-E XY support command differs:

- Native: `G1 X102.572 Y101.777 E.03026`.
- Rust: `G1 X99.953 Y101.169 E.03026`.

Admit a focused public API support-command RED and a separate unchanged
complete Native/API/CLI comparator test. Require two runtime Native captures.
The complete comparator still fails earlier at the M73 prefix; do not mask
progress, feeds, metadata, or footer, or call a focused improvement full parity.

## Included, deferred, and verification

Included: the f32 angle sum and its public output witnesses, as separate
ARD, RED, and port commits. Compare an identical frozen test inventory before
and after, requiring zero PASS-to-FAIL transitions; retain inherited failures.
Run rustfmt and strict all-target Clippy without suppressions.

Report the first support-fill difference and tip counts after the port. If
83 versus 82 tips persist, localize `ensure_maximum_distance_polyline` only
under a subsequent scope. Distance sampling, roof behavior, graph generation,
organic drawing, skirt/seam, flow, geometry kernels, writer/progress, presets,
and footer repairs are outside this slice. Remove temporary observations,
rebuild, and verify complete-byte restoration before reporting results.

## Bounded result

The literal angle port restores the first support command and all 26913
ordered positive-E XY commands tagged `Support` in this fixture. Seed counts
are 83 in both implementations; draw-entry counts are 1140 in both. The first
two installed support layers have ordered-identical path coordinates and flow,
with 284 and 319 points respectively. No distance-sampler repair is included.
The frozen 41-test inventory improves from 5 to 6 PASS with zero PASS-to-FAIL
transitions. Complete output still fails at the separate M73 prefix; these
support observations are not a complete-output parity exception.
