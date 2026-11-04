# Spec: PrintObject-centered fill reference

## Upstream boundary

OrcaSlicer2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`Print.hpp:347 PrintObject::bounding_box()` returns a symmetric box
`[-m_size/2,+m_size/2]`; `Fill/Fill.cpp:1224 Layer::make_fills` passes that box
to fillers; `FillBase.cpp:286-290 Fill::_infill_direction` uses its center.
The center is exactly `(0,0)` in the centered print-object coordinate frame,
not the center of compensated/quantized slicing contours.

Rust destination: `project_slice/fill_entities/generate.rs::generate_layer`,
where the reference passed to grid/cubic and other existing pattern ports is
currently computed using `fill_entities::object_center(object_slices)`.
Replace that temporary slice-bounds approximation at this production fill
boundary with the exact upstream centered reference. This is a source-semantic
repair of the existing compatibility shell, not an Ares pipeline feature.

## Included / deferred

Included: reference selection for existing `generate_layer` fill consumers;
no changes to scanline arithmetic, contour connection, sorting, travel, cooling,
M73, comparator or emitted output normalization. Zero is derived from upstream
symmetric box construction, not a printer/nozzle special case.

Deferred: extended consistent-pattern bounding-box size approximation,
bridge-over-infill sparse anchoring (retained `object_center`
caller), tree/support, viewer data, template RNG, and unrelated vendor causes.
The remaining `object_center` helper is still used by bridge anchoring; do not
remove it or silently broaden this repair into bridge behavior.

## Reproduction and failure chain

Strict existing public byte replay on main7ac55555 is 0/1, first difference
M73 line80. Official single-CPU re-slice matches the frozen reference after generator
normalization only. External GDB captures on z0.84 show identical rotated
expanded contour vertices for all3 cubic families, but native reference0,0
versus Rust1,0. Raw scanline endpoints already differ by one scaled unit,
then anchor connections and travel lengths change; M73/cooling are downstream.
No production debug probes are needed.

## Slice scope

The complete68-case pre-edit replay is0/68 PASS; the candidate is52/68 PASS.
This slice covers those52 restored cases, retaining the full68 report and all16
failures. Fourteen Snapmaker0.2 Internal Bridge one-line extrusion residuals
belong to the bridge-over-infill slice; two Qidi0.2 late-layer grid residuals
are a separate follow-up slice. Exact case paths and first differences are in
the plan. No 68/68 or 8/8 official claim is permitted: initial official
representatives are6/8, controls823/823.
This scope does not alter any comparator, fixture, or failing assertion.

## Acceptance

Keep public byte-input/output seam and existing strict ordered comparator.
Before editing replay all60 Snapmaker and8 Qidi residuals on main. After editing replay the same
unchanged68 inputs/references, classifying any remaining differences honestly.
The mandatory tracer must be fully green, not merely movement-equivalent.
Replay existing green stock controls using the same immutable reference data.
Fresh official2.4.2 single-CPU output must match for representative nozzle/pattern
cases. Workspace Nextest, Clippy, fmt and core+adapter WASM must pass. Do not waive genuine residual failures or weaken tests.
