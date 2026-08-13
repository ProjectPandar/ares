# Task 22O.80 implementation plan

1. Add RED single-run and flip-parity tests.
2. Add consumed state to the owned region-generation working copy.
3. Implement vertical run boundaries and exclusive overlap extension.
4. Add separated/multiple-run/repeatability tests.
5. Run focused regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing region module. GREEN passes 2/2 O80 tests.
Strict core Clippy, rustfmt, diff, and LOC gates pass.

No neighbors/chaining/polyline/entity output, lifecycle, fallback, or G-code.
