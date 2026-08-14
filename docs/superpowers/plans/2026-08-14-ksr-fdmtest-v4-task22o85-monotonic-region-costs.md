# Task 22O.85 implementation plan

1. Add RED symmetric/scaled orientation-cost tests.
2. Expose O80 vertical-run helpers within rectilinear only.
3. Port source dual-orientation traversal and common-minimum normalization.
4. Run O77-O84 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing cost module. GREEN passes 2/2 O85 and both O84
regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No path matrix/ant chaining, entity output, lifecycle, fallback, or G-code.
