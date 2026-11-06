# Avoid-crossing inner offset source contract

## Upstream boundary

This is a source-cited rewrite slice of
`src/libslic3r/GCode/AvoidCrossingPerimeters.cpp`, function `inner_offset`,
covering the source boundary at lines `1036–1060` and its immediate result
selection through line `1106`.

The destination is the existing avoid-crossing boundary preparation path. This
slice ports the upstream contract only; it does not introduce a new routing
pipeline.

## Included behavior

1. Derive the scaled epsilon used by the inner-offset width candidates from
   the active coordinate scale, preserving upstream rounding and units.
2. Offset the local distance-grid bounding box by that scaled epsilon before
   creating the grid.
3. Apply the upstream double-precision padded bounding-box area predicate,
   comparing its area with `sqr(scale_(0.1f))`. When the predicate selects the
   small-expolygon path, preserve the original resampled polygon in the result
   collection.
4. Use the upstream hole-collapse probe defaults: miter join and miter limit
   `3`, with the existing variable-offset miter limit remaining separate.
5. Preserve upstream resampling order, width-candidate order, contour-distance
   preparation, empty-result handling, largest-result selection, and the
   existing union handoff.

## Deferred behavior

Routing, waypoint selection, fallback normalization, closest-line selection,
final cumulative-distance carriers, shared grid backends, external-boundary
assembly, zero-area handling, and topology behavior outside the listed
`inner_offset` operations are deferred. No tolerance tuning, numeric variant,
formatter change, comparator weakening, or output compensation is included.

## Non-negotiable decisions

- Epsilon is derived from coordinate scale; a fixed coordinate constant is not
  equivalent.
- Owner-local epsilon padding is distinct from final routing-grid radius
  padding.
- The hole-collapse probe defaults are distinct from the variable-offset
  backend's miter configuration.
- The source area predicate and polygon-preservation behavior are retained
  without substituting width/height tests.
- Routing decisions and final cumulative distances remain unchanged.
- Whole-file ordered generator output remains the qualification criterion.

## RED witness

The existing
`closest_lines_runtime::single_volume_closest_lines_control_matches_native`
whole-file test is the RED boundary. Qualification compares the original Orca generator output with both
public API and CLI output using the exact input, resources, executable
identities, headers, metadata, and ordered line comparison. This specification
records no waypoint causality claim and no GREEN result.
