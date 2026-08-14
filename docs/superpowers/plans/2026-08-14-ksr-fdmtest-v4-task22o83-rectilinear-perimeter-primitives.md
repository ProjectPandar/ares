# Task 22O.83 implementation plan

1. Split existing rectilinear tests into a directory module.
2. Add RED direct arc and slice-indexed perimeter tests.
3. Port source directed distance, length, and emit primitives.
4. Run O77-O82 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

RED caught incorrect same-segment reverse append. GREEN passes 2/2 O83 and all
7 O77-O79 regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No corrected link graph, region costing/chaining, entity output, lifecycle,
fallback, or G-code.
