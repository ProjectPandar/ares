# Layer-region aggregate islands rewrite slice

Architecture: [ARD0087](../../architecture/ard-0087-layer-region-islands.md).

## Source boundary

Port `OrcaSlicer/src/libslic3r/Layer.cpp:34-67::Layer::make_slices`
using `ClipperUtils.hpp:364::union_safety_offset_ex` into existing
`ares-core/src/project_slice/slice_ordering.rs`. Rename the single-region
scaffold to make_slices; retain its existing observable single-region ordering
as a temporary compatibility shell, not a new pipeline or fallback.

## Included behavior

All regions contribute contours and holes to the safety-offset union.
Retain each original region's surface metadata, options and identity.
Keep planned empty layers and source-owned first-point island ordering.
Propagate geometry failure without expanding visibility or adding guards.

## Deferred behavior and acceptance

This slice does not remove the existing project multi-region limitation.
Required subsequent owners are PrintObjectSlice.cpp region compensation,
Layer.cpp::make_perimeters compatibility groups, Fill.cpp::make_fills
region ownership, SupportMaterial.cpp all-region traversal and Tree support.
Normal support cannot merge before Tree output acceptance.

Supplemental rectangle/hole/metadata tests verify this geometry prerequisite,
not full modifier parity. Clean-HEAD workspace and unchanged KSR complete
output must pass, with fmt/Clippy and Tier-1 compile/browser regression.
Use untouched official Orca for a finite public complete-output control;
exclude only the generator line, never relax motion or byte ordering.
Freeze the independent patch for six-axis review before code+docs commit/push.
