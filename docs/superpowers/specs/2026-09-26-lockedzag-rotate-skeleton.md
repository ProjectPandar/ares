# LockedZag rotated skeleton pass

## Upstream and Rust boundaries

`FillRectilinear.cpp:3866-3934` splits a LockedZag surface into a skeleton
(`offset_ex(-skin_infill_depth)` ∩ density bucket, expanded by
`infill_lock_depth`, clipped to the surface) and a skin ring
(`diff_ex(surface, zig_expas)` ∩ density bucket). Skeleton fill uses
`zig_params` which keeps `params.locked_zag = true`; skin fill uses
`cross_params` with `locked_zag = false` (`:3919`).

`FillRectilinear.cpp:2763-2765`: `params.locked_zag` rotates the infill
direction an additional π/2 inside `fill_surface_by_lines`. Combined with
`_infill_direction` (+π/2) the skeleton grid is orthogonal to the skin grid.

`Fill.cpp:1305-1318`: `params.locked_zag` is set only for `ipLockedZag` in
the fresh `FillParams`, alongside `infill_lock_depth`/`skin_infill_depth` and
the `lock_param` sidecars.

Rust destinations: `fill/rectilinear/surface.rs` (`MonotonicFillParams`
gains `locked_zag`), `fill/locked_zag.rs` (passes the flag per pass),
`fill_entities/locked_zag.rs` (dispatch wiring).

Included: +90° rotation of the skeleton pass. Traversal parity required two
further ports in `fill/rectilinear/traverse/` (same landed workstream):

- `horizontal_available` rejects a target whose outward-adjacent
  intersection is INNER (`FillRectilinear.cpp:521-527`), so a perimeter
  segment is only followed when it lands at the END of a vertical run; the
  right-link consumption flag is `it_this.consumed_perimeter_right`
  (`:532`), the left side reads `it_other`'s flag.
- `vertical_outside` same-line continuation (`:1525-1563`): previous-slot
  first (`:253-268`), `emit_perimeter_segment_on_vertical_line` with
  `forward = going_up ? has_left_vertical_up() : has_right_vertical_down()`
  (contour direction, not travel direction — `:1540-1541`), TooLong links
  end the polyline and start a new one at the target, bypassed outer
  intersections are consumed, and `consumed_perimeter_right` is marked on
  the current and adjacent intersections (`:1557-1558`).

Deferred: `symmetric_infill_y_axis` mirroring (already wired at the
fill_entities level) and unrelated patterns.

## Evidence

`sparse_infill_pattern=lockedzag` official sweep before this fix emitted
skin-like zigzag for both passes; Orca's skeleton lines are diagonal offsets
of each other while Ares produced the same direction twice. After
`locked_zag` gating, the skeleton pass direction differs by 90° and matches
the oracle's diagonal serpentine shape on the cube10 fixture. After the
contour-direction fix the sweep case is byte-identical to official Orca
2.4.2 — `option/sparse_infill_pattern/lockedzag` PASS
(26/26 values).
