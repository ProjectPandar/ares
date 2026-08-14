# Task 22O.81 implementation plan

1. Add RED linear-chain and one-to-many neighbor tests.
2. Implement boundary-to-region maps and source overlap scattering.
3. Sort/deduplicate and repair symmetric links.
4. Run focused regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing neighbor module. GREEN passes 2/2 O81 tests and
all 1,179 task22o core regressions. Strict core Clippy, rustfmt, diff, and LOC
gates pass.

No lengths/chaining/polyline/entity output, lifecycle, fallback, or G-code.
