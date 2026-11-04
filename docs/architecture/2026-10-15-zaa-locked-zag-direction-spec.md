# ZAA ordinary LockedZag regional direction SPEC

Docs only. No Native/build/test run or RED claimed for this slice.

Builds on the anchoring slice through0eadb58ff75e597b2c876f93cfbee8f99f89fdd3,
which remains unchanged.
Upstream inspected: OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Owning libslic3r boundary

Fill/Fill.cpp::Layer::make_fills:1235-1242 creates each SurfaceFill's filler,
sets its layer ID and assigns the printing region's runtime conjunction:
`rcfg.zaa_enabled && rcfg.zaa_dont_alternate_fill_direction`.
Fill/FillBase.cpp::Fill::new_from_type:75 selects FillLockedZag for
ipLockedZag. Fill/FillRectilinear.hpp:210-227 derives FillLockedZag from
FillRectilinear without overriding _layer_angle. It only overrides the
extrusion entry and consistent-pattern/lock-region behavior.

Fill/Fill.cpp:1305-1309 supplies locked_zag, skin/skeleton depths and the
lock-region parameters. FillRectilinear.cpp::FillLockedZag::
fill_surface_extrusion:3936 onward calls fill_surface_locked_zag:3866-3934.
The skeleton and skin passes call the same object's fill_surface at
3907/3927; clearing cross_params.locked_zag at3919 changes the skin's
additional rotation, not the object's regional dont_alternate flag.
FillRectilinear::fill_surface_by_lines:2762-2765 calls _infill_direction
before adding the independent locked_zag pi/2 rotation.

FillBase.cpp::Fill::_infill_direction:275-323, specifically311-313,
skips the layer-angle term when dont_alternate_fill_direction is true.
FillBase.hpp:211 supplies the normal odd-layer pi/2 alternation. Bridge
angle precedence, fixed_angle, thickness_layers and the skeleton's extra
pi/2 remain separate behavior. This is a missing runtime flag port, not
a new skin/skeleton fill algorithm or a new Ares pipeline.

## Rust gap and exact destination

Ordinary fill_entities/generate.rs resolves the canonical printing object
and all_regions[fill.region_id] for its existing rectilinear branch, as
ported in58f963c54. The separate LockedZag arm instead passes layer.id to
fill_entities/locked_zag.rs::append. That function initializes its base
MonotonicFillParams::dont_alternate_fill_direction to false. Its existing
skin/skeleton closure copies that base before assigning density,
horiz_move and locked_zag. The shared fill/rectilinear/surface.rs::
infill_direction:185-195 already implements the upstream conditional.

Production boundary: only fill_entities/generate.rs's LockedZag call
and fill_entities/locked_zag.rs's parameter context/base initialization.
Use the actual fill region in resolved_object.all_regions; reuse existing
SurfaceCall in place of the layer-id argument, preserving the five-argument
append boundary and existing ObjectFillGeometry/capacity strategy. Carry
only the regional conjunction to the existing base; preserve its layer ID.
No new helper framework, allocation, lint suppression or first-region
selection. Existing adapters remain temporary compatibility shells around
the named Layer/FillLockedZag source boundaries.

## Included and deferred

Included: ordinary sparse LockedZag skin/skeleton generation through its
existing shared rectilinear direction kernel, for the missing regional
flag only. First public coverage is a single classic object/instance with
positive sparse density and alternating, non-bridge infill layers.

Deferred: Line and other dedicated generators; ironing's separate regional
assignment; support, raft and tower; new modifier, multi-object or Arachne
witnesses; any LockedZag geometry/density/flow/width/phase defect. Do not
change lock-region construction, clipping, residual gaps, sorting, motion,
ZAA contour sampling, cooling or bridge candidate logic. Anchoring's flag
is already handled by0eadb58ff and is not reopened here.

## Bounded Native acceptance and activity before public RED

Reuse the admitted input setup from
option_coverage/fill_grid.rs::
locked_zag_shifts_preserve_complete_output:30-39 and its compare
helper:80 onward: existing wide slab height2.4mm, LockedZag40%, relative E,
infill angle45, infill_shift_step0.731272, one top/bottom shell, wall_loops0,
prime tower disabled. Preserve model/member provenance and existing normalized
vendor profile options. No fixture search or geometry amendment is included.

Prepare three input-only cases differing solely in the two ZAA switches
and shared valid ZAA parameters (min_z0.05, minimize_perimeter_height35):
1. ZAA enabled, dont_alternate false: complete-output parity control.
2. ZAA enabled, dont_alternate true: proposed activity/RED case.
3. ZAA disabled, dont_alternate true: conjunction control, included in RED.

Use official and seeded modes with a fresh
Native cache/artifact root for each invocation; no full/workspace. Force
two fresh Native references per case/mode (six per mode), with frozen ZIP
metadata. Require successful complete Native exports and exact effective
LockedZag/ZAA settings, repeat-stable normalized output and Sparse infill
XY/E streams. Check both controls against current Ares API+CLI before RED.

Require positive Sparse infill XY extrusion on odd/even layers and a
Native enabled-flag0/flag1 geometry difference on an odd non-bridge layer.
Report that stream's layer, role and SHA plus corresponding Ares flag1
stream; do not infer activity merely from a footer or estimated time.
If Native rejects/crashes, controls diverge, repeats vary or no geometry
witness exists, STOP and report; no widening or fabricated RED.

## Public RED and GREEN gates

Following accepted Native evidence, commit public E2E RED separately.
Use input-only frozen SHA-pinned3MFs, OrcaRunner::from_env's existing
Some/else-return convention, public API+CLI, typed option assertions,
repeat artifacts and unchanged strict whole-output comparator. Do not
commit Native output. Assert active Sparse infill XY/E equality before
whole-output parity, with both controls in the same RED commit. Expected
true case FAIL, both controls PASS in both modes. If the divergence is
outside this boundary, STOP and request localization rather than changing
comparisons or repairing adjacent code.

GREEN then carries the one regional conjunction through the existing
context; tests and comparisons remain unchanged. Require all three cases
PASS both modes, raw-unchanged false/disabled controls, existing LockedZag
shift/skin/skeleton-width controls, ZAA/fill-direction/anchoring controls,
fmt, strict clippy and wasm32 on the exact clean hash. Files stay at most
400LOC.
