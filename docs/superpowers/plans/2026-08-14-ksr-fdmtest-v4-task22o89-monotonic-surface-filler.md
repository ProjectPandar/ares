# Task 22O.89 implementation plan

1. Add RED exact public filler/rotation tests.
2. Split O82 contour retention from vertical-line population.
3. Port direction, offsets, spacing adjustment, line alignment, graph pipeline,
   and inverse rotation.
4. Run O77-O88 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing surface module. GREEN passes 2/2 O89 and all five
O77/O88 boundary regressions. Strict core Clippy, rustfmt, diff, and LOC gates
pass.

No grouped entities, lifecycle, fallback, motion, or G-code.
