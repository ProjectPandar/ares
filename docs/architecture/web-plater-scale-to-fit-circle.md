# "Scale to build volume" on a circular bed

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. This is the follow-up that `web-plater-scale-to-fit.md` deferred: `Selection::scale_to_fit_print_volume`'s Circle branch, `fit_circle` (`Selection.cpp:1534-1569`). Today the session refuses a Circle bed and the web disables the item there. This slice replaces that refusal.

## Upstream boundary
- **`fit_circle`** (`Selection.cpp:1534-1569`). For every selected `GLVolume`:
  - It transforms the volume's convex hull mesh (`GLVolume::convex_hull`, `3DScene.hpp:361`) by its world matrix.
  - It keeps the largest Z extent as `max_z`.
  - It appends the hull's 2D hull points, `TriangleMesh::convex_hull` (`TriangleMesh.cpp:436-445`): the vertices through `Point::new_scale` (`Point.hpp:214`, truncating), then `Geometry::convex_hull`.

  It then fits a circle and moves the selection:
  1. The points' circle is `smallest_enclosing_circle_welzl(points)` (`Circle.hpp:151-154`, with `SCALED_EPSILON`, `libslic3r.h:109`). Its radius is widened by 0.01 mm.
  2. `s = min(print_radius / circle_radius, printable_height / max_z)`.
  3. `offset = s * ((circle centre, 0.5 * max_z) - selection centre)`.
  4. The print centre is the bed circle's centre at `0.5 * printable_height`, shifted by `plate_shift`. `plate_shift` is the current plate's centre minus `BuildVolume::bed_center()` (`BuildVolume.hpp:87`).
  5. It calls `fit(s, print_centre - (selection centre + offset))`.

  It returns without a change for a zero bed radius, no points, or a zero circle radius or `max_z`.
- **The bed circle** is `BuildVolume::circle()` (`BuildVolume.hpp:91`): the RANSAC circle of `printable_area`, scaled (`BuildVolume.cpp:46-75`). Ares already ports it as `BedShape::Circle` (`print_volume/circle.rs`), at the plate origin. `fit_circle` reads the bed's own volume (`Plater.cpp:11095`), which sits at the first plate; `plate_shift` moves it.
- **Welzl** (`Circle.hpp:112-148`) is the deterministic, unshuffled Welzl loop over `CircleSq` (`:40-63`).
  - Each new circle is the pair or triple circle, inflated by epsilon.
  - It uses the strict `contains`.
  - The triple's centre comes from `circle_center` (`:18-36`), with its collinear branch and the `epsilon` passed in.
  - `Circle(CircleSq)` takes the square root (`:67-93`).

## Upstream UB not ported
- `GLVolume::convex_hull` is set only for model parts (`3DScene.cpp:921-927`). The selection holds every volume of the instance, so `*v.convex_hull()` dereferences null for a modifier, negative volume or support blocker. Ares refuses such an instance on a Circle bed with `InvalidInput("scale to build volume on a circular bed needs every volume to be a model part")`.
- Every RED fixture holds model parts only, except the one refusal case.

## Ares destination
- **The Circle arm of `PlaterSession::scale_to_fit`** (`plater/scale_to_fit.rs`) calls `fit_circle`, built on the existing `fit`. The welzl port goes into a new `geometry/enclosing_circle.rs`.
- **`print_volume/circle.rs`'s `circle_center`** takes the `epsilon` argument, as upstream's does: RANSAC passes `EPSILON`, welzl passes `SCALED_EPSILON`. The bed circle is read at the first plate's origin.
- **Removed:** `scale_to_fit_ported`, wasm `scaleToFitPorted`, and the web's disabling of the item, all of which existed only for the refusal.

## Included and deferred
- **Included:** `fit_circle` and welzl.
- **Deferred** (as before): multi-object selections and undo.

## Oracle and RED
In `plater_scale_to_fit.rs`, case 6 is retargeted from the refusal to the fit.

The circular bed is the Ender-3 with a 36-point `printable_area` of integer points exactly on the radius-65 circle around (110, 110): the Pythagorean points of 65². This makes the RANSAC circle exactly (110, 110) with radius 65, and the largest gap undercuts by 0.65 mm, under the 3 mm limit. Expectations are recomputed in-test.
1. **Height-limited.** A 10 x 10 x 200 mm cube gets `s = 250 / 200` and centres at (110, 110) on the bed. It also slices like Native.
2. **Radius-limited.** A 100 x 100 x 10 mm cube (Size X and Y, uniform off) gets `s = 65 / (50·√2 + 1e-4 + 0.01)`, centred and on the bed. It also slices like Native.
3. **Plate 2.** Case 1 on an added plate 2 centres at plate 2's origin + (110, 110): the `plate_shift`. It also slices plate 2 like Native.
4. **Refusal.** A cube with a modifier volume on the circular bed is refused (`InvalidInput`), and its project is byte-unchanged.

Web E2E `plateScaleToFitCircle.spec.ts`: case 1's project through the object menu, then plate 1 compared with Native.
