# Task 22O.90 implementation plan

1. Add RED graph-native Monotonic/MonotonicLine entity tests.
2. Rename the pattern-specific layer pass to the source layer-entity seam.
3. Add monotonic parameter derivation, geometry mapping, and role/flow entities.
4. Run O76/O89 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved missing Monotonic dispatch. GREEN passes 2/2 O90, all three
O76, and both O89 regressions. Strict core Clippy, rustfmt, diff, and LOC gates
pass.

No other fillers/thin fills, lifecycle, fallback, motion, or G-code.
