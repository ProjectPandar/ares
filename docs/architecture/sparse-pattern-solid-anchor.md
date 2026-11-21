# Cross Zag and Locked Zag bridge anchor phase

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), under
`src/libslic3r/`:

- `Fill/Fill.cpp:1561–1684`, `Layer::generate_sparse_infill_polylines_for_anchoring`,
  creates the configured sparse filler, but calls `fill_surface` with fresh
  `FillParams`. Neither `horiz_move` nor `locked_zag` is set on the anchor call
  (`Fill/FillBase.hpp:128,132`: defaults zero and false). Cross Zag and Locked
  Zag retain consistent object-wide bounds (`Fill/FillRectilinear.hpp:210–228`)
  and use the inherited `FillRectilinear::fill_surface` on single-line sparse
  fill (`Fill/FillRectilinear.cpp:3490–3501`). `fill_surface_by_lines` applies
  the supplied phase to the scan origin at `:2783–2825`.
- `Layer::make_fills` (`Fill/Fill.cpp:1449–1474`) separately supplies the
  alternating `infill_shift_step` phase and the Locked Zag extrusion parameters.
  `FillLockedZag::fill_surface_locked_zag` and `fill_surface_extrusion`
  (`Fill/FillRectilinear.cpp:3968–4044`) handle the printed skin and skeleton;
  that is not the anchor path.
- `PrintObject::bridge_over_infill` (`PrintObject.cpp:3364–3384,3839,3925–3926,3961–4028`)
  builds and clips anchors from the layer below each bridge candidate, then uses
  those lines to reconstruct the bridge. The surface transaction
  (`:4036–4096`) clips the next layer's bridge outline ring to the current
  layer's perimeter band and merges it into `stInternalSolid`, subtracting any
  same-layer bridge cut. The Z1.6 candidate thus changes the solid at Z1.4
  before ordinary infill extrusion is generated.

## Rust boundary

In `crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/`
`sparse_anchoring/linear.rs`, use zero phase in the `MonotonicFillParams` for
`append`. The existing dispatch already represents the upstream sparse anchor
classes and their consistent bounds. Keep the configured filler, spacing,
rotation, anchor lengths and existing unlocked state. Do not change the
bridge transaction, sticky phase projection, or ordinary extrusion path.
This boundary builds on the region/body octree handoff at `6692d80ac`, which
also ports part of `generate_sparse_infill_polylines_for_anchoring`.

## Verification

Slice a synthetic stepped MK4 prism with the stock sparse pattern and with
separate single-key Cross Zag and Locked Zag selections. The pattern variants
retain a nonzero shift step; the stock is a control. Compare entire G-code
from the public slice API and the CLI to official and seeded Native with only
the generator line normalized. Compare existing infill, bridge, anchor,
solid, Zag, and Adaptive/Support Cubic parity cases before and after the change.

Hilbert Curve and Octagram Spiral share the downstream bridge transaction but
enter a separate PlanePath anchor branch with additional turning-pattern
handling. This does not address their endpoints, multiline dispatch, or
other filler behavior.
