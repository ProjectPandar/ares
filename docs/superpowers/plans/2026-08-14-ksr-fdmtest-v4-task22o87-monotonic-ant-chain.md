# Task 22O.87 implementation plan

1. Add RED precedence/determinism path tests.
2. Port standard default MT19937-64 in a separate shard.
3. Port greedy initialization and ant rounds over O86.
4. Run O80-O86 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved missing chain/RNG modules. GREEN passes 3/3 O87 and both O86
regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No polyline/entity output, lifecycle, fallback, or G-code.
