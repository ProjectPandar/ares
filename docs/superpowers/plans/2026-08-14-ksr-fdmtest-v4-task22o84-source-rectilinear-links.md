# Task 22O.84 implementation plan

1. Add RED wraparound/source-distance topology test.
2. Migrate O78-O81 tests to retained slices and remove lines-only linking.
3. Replace approximate candidate and Euclidean quality logic with source
   directed selection and O83 contour measurements.
4. Add invalid symmetry tests and run O77-O83 regressions.
5. Run strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved O78 still accepted bare lines. GREEN passes 2/2 O84 and all
15 O77-O83 regressions. Strict core Clippy, rustfmt, diff, approximation-removal,
and LOC gates pass.

No region costing/chaining, entity output, lifecycle, fallback, or G-code.
