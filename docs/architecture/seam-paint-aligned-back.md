# AlignedBack seam paint

## Native source and Rust boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode/SeamPlacer.cpp`:

- `:850–860,957–978` resolves blocker overlap before marking central
  candidates. In both `SeamComparator::is_first_better` and
  `is_first_not_much_worse` (`:1176–1268`), AlignedBack and Aligned prefer a
  central enforcer first, then Enforced > Neutral > Blocked. Equal enforced
  points are acceptable to the alignment comparison. Rear compares paint
  types but does not give the central flag comparator priority.
- `:196–199,2150–2167` computes visibility for AlignedBack as for Aligned,
  adding `clamp((normal · (0, -1, 0) + 1.2) * 0.5, 0, 1)` to each
  visibility sample. This penalizes front-facing surfaces without making
  maximum Y a primary geometric key. Rear skips visibility and uses larger
  Y in the strict comparator and a Y tolerance during alignment
  (`:1201–1203,1258–1260`). Paint takes precedence over those preferences.
- `:1339–1350,2176–2204` picks a point on each perimeter and aligns across
  layers for AlignedBack, Aligned and Rear. The next-layer central-enforcer
  shortcut (`:1802–1818`) applies to all three, independently of comparator
  priority. `align_seam_points` (`:1883–2081`) builds neighboring-layer
  strings, weights sharp angles and adds 3 to enforced points, fits a cubic
  B-spline, and interpolates toward it with corner snapping. It retains the
  paint-aware loop projection (`:1498–1627`) and the extra corner-dependent
  pull for enforced points. Only Rear applies the 0.3 string-length multiplier
  before determining the segment count (`:2037–2039`).

The boundary is `project_slice/seam_placement`: reuse the existing AlignedBack
visibility bias and the shared candidate, paint comparison, alignment and
constrained fit paths. Admission in `seam_placement/paint.rs` should not change
unpainted placement. Any discrepancy in the unchanged unpainted control is
an independent gap and must be reported before enabling painted AlignedBack.

## Public behavior

Reuse the X1C, K1 and MK4 seam-paint projects, changing only `seam_position`
to `aligned_back`. Each printer exercises plain, front enforcer, rear blocker,
both annotations and side paint. Use strict complete G-code comparisons
(only the generator line is normalized) between the public API, CLI and both
Native producers. Compare Native wall approaches to the plain control per
printer; distinguish active paint from annotations that leave a seam unchanged.

Continue rejecting Random, spiral, Precise Seam, extended palette values and
full-green normalization. No 3MF format, geometry, or normalization change is
part of painted AlignedBack.
