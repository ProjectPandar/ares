# Task 22O.82 implementation plan

1. Add RED retained inventory/index-addressability tests.
2. Add owned source/outer/inner contour context around O77 lines.
3. Keep existing lines-only seam as a temporary delegating shell.
4. Run focused regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

The RED test required the missing retained slice context. GREEN passes 2/2 O82
tests. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No path measurement/emission, chaining, entity output, lifecycle, fallback, or
G-code.
