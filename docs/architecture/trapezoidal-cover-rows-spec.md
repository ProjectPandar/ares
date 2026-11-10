# Stars and Triangles multiline rows built over the surface cover

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/Fill/`.

- `FillStars::fill_surface` (`FillRectilinear.cpp:3586-3601`) and `FillTriangles::fill_surface`
  call `FillRectilinear::fill_surface_trapezoidal` (`:3100-3485`) when `multiline > 1`, with
  pattern type 2 for Stars (tri-hexagon) and 1 for Triangles.
- Common frame, `:3128-3147`:
  - the surface is rotated by `-base_angle` about `rotate_vector.second`;
  - `layer_mod` is `infill_layer_id % 3`, and `angle` is `layer_mod * 2π/3`;
  - for types 1–3 the contour is translated by `-rotate_vector.second` and, when `layer_mod`
    is set, rotated by `-angle`;
  - its extents, grown by `period`, form `cover`.
- Stars rows, case 2 (`:3317-3377`):
  - the tiles run from `floor(cover.min.x / period) - 1` up to, not including,
    `ceil(cover.max.x / period) + 1`; tile `t` starts at `t * period - period / 2`;
  - the rows run from `floor((cover.min.y - row_y_offset) / hex_height) - 1` through
    `ceil((cover.max.y - row_y_offset) / hex_height) + 1`, with
    `row_y_offset = tri_height - (2 * tri_height) / 3`;
  - row `r` sits at `y = r * hex_height + row_y_offset`. It is shifted by `period / 2`, plus
    another `period / 2` when `r % 2 != 0`; C++ `%` keeps the sign, so negative odd rows
    shift too;
  - each row adds the normal template, then its mirror about `hex_height`.
- Triangles rows, case 1 (`:3237-3316`):
  - x runs from `(floor(cover.min.x / period) - 1) * period` to
    `(ceil(cover.max.x / period) + 1) * period`;
  - the rows run from `floor(cover.min.y / h) - 1` through `ceil(cover.max.y / h) + 1`,
    with row `r` at `y = r * h`;
  - each row adds its base line, then the normal template, or the mirrored one when
    `r % 2 != 0`.
- Back to the surface frame, `:3417-3426`: for types 1–3, each row is rotated by `angle`
  about the origin when `layer_mod` is set, then translated by `rotate_vector.second`.
- The shared tail from `:3428` (centerline crop, multiline offset, cap start, contraction,
  clipping, connection) is not changed.

## Rust destination

- `crates/ares-core/src/fill/stars_trapezoidal.rs` and `stars_trapezoidal/rows.rs`.
- `crates/ares-core/src/fill/triangles_trapezoidal.rs` and
  `triangles_trapezoidal/rows.rs`.
- Today both `rows.rs` files size the rows from the object bounds, symmetric about the
  origin. They use `periods_x` and `periods_y` from half the bounds size, rounded up, with
  `periods_y` made even. That range predates the cover.
- Each `fill_surface` now builds `cover` and passes it to `rows::generate`, in the same way
  `fill/cubic_trapezoidal.rs` already does for case 3:
  - the cover comes from the rotated contour, shifted by `-params.reference` and rotated by
    `-angle`;
  - the rows are rotated by `angle` and shifted by `+params.reference`.
- `rows::generate` is rewritten as the case 2 or case 1 loop over that cover.
- The `bounds` field of the Stars and Triangles `SurfaceGeometry` is no longer read, so it
  is removed:
  - in `project_slice/fill_entities/stars.rs` and `triangles.rs`, the `LayerGeometry.bounds`
    field and its `unreachable!` guard go;
  - in `prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs` and `arms_tail.rs`,
    only the use of the bounds is dropped. The shared object-bounds plumbing stays, because
    Grid still reads it.
- `trapezoidal_tail.rs`, the cubic port, the grid port and the count-1 paths are not
  changed.

## Included

- Cases 2 and 1 above, and the step back to the surface frame (`:3417-3426`) for those two
  types.

## Deferred

- Grid, case 0 (`:3150-3236`). Its Ares rows already span the whole extended object box
  (aligned from `bb.min`), which contains every surface, so no row ends inside a surface.
  250b keeps only the rows that meet the cover; the extra Ares rows lie outside the surface
  and the tail clips them away. The remaining 250b difference is a different mechanism: the
  grid is phased through `grid_center` (`align_to_grid` with the box center) and transposed
  about it (`:3167-3175`, `:3229-3234`). No grid test fails; all `grid_mode0_*` cases pass,
  including `count10`.
- Rotating the surface about `rotate_vector.second` (`:3130-3132`). The Stars, Triangles and
  cubic ports rotate about the origin and handle the reference only as the translation
  above. That is unchanged here.

## Premises

1. `tri_hexagon_multiline_s10` (Ender-3, tri-hexagon, `fill_multiline = 10`, 30%, 45°)
   fails in both modes.
   - On its first sparse layer, 235 of Native's 252 sparse segments are also in Ares.
   - Native's other lines run straight to the contracted edge at x = 133.02. In Ares they
     stop around x 130–132, y 121–123, and turn back with rounded multiline outline caps
     inside the part.
   - With `multiline = 10` the period is about 48 mm, so the old symmetric range ends a row
     inside the surface. The tail then caps it there.
2. `triangles_mode1_t3` (triangles, `fill_multiline = 3`, 30%, 45°) fails in both modes,
   with a different sparse byte count from Native.
3. Before writing this spec, I tried the cover-based rows above as two temporary probes,
   since reverted. In both modes:
   - with the Stars probe, `tri_hexagon_multiline_s10` went from FAIL to PASS, and the
     other 10 tri-hexagon tests (S1, S2, S3, S2A, H1, H2, ironing, support ironing, two
     multiline crops) stayed PASS;
   - with the Triangles probe, `triangles_mode1_t3` went from FAIL to PASS, and the other
     18 triangles tests (mode1 G2, R1, S2, T1, T2, T2A0, T2C2, T2D20, T10, TH1, TH2,
     count-1/2 bridge streams, named crop, ironing, support ironing, the triangle-count CLI
     case, resource defaults) stayed PASS.

## Observable behavior

For Stars and Triangles with `fill_multiline > 1`, the sparse infill rows reach past the
surface on every side by at least one period. Every multiline cap then lies outside the
surface and is clipped away. The sparse infill G-code equals Native's.

## Failure modes

- Lattice phase: the tile and row origins must stay at the origin of the rotated, shifted
  frame (Stars: `t * period - period / 2`, rows at `r * hex_height + row_y_offset`;
  Triangles: `x_min` aligned to `period`, rows at `r * h`). The passing S1–S3 and T1, T2,
  T10 cases show this, since their phase already matches.
- Row parity with negative rows: `r % 2 != 0` holds for negative odd `r`, in Rust as in C++.
- The cover in the wrong frame: it must be built after the shift by `-reference` and the
  rotation by `-angle`, as upstream does. Building it before those steps leaves the rotated
  layers (layer_mod 1 and 2) short.
- An off-by-one at the ends: the tile range is half-open, the row range is closed, and both
  are padded by one past floor and ceil.
- The anchoring callers: the bridge-over-infill sparse anchoring uses the same Stars and
  Triangles functions. Their output changes only where the old range was short.

## Test plan

Both witnesses already exist as whole-output parity tests, are named by behavior, and fail
at the base in both modes:

| Test | Before the port | After the port |
|---|---|---|
| `option_coverage::tri_hexagon_multiline::tri_hexagon_multiline_s10` | F | P |
| `runner::triangles_multiline_mode1::triangles_mode1_t3` | F | P |

So this slice adds no new test commit; the two existing tests are its failing tests. If a
separate test is wanted, I would add it before the port.

The port is expected to turn exactly these two into PASS in both modes, with no PASS
turning into FAIL in the full run.
