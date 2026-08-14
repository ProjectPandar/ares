# Task 22O.91 implementation plan

1. Add RED stage ordering/ownership and public lifecycle tests.
2. Add transactional all-object/all-layer materialization and disposal.
3. Advance `slice_project_sync` through O91 before the explicit incomplete sink.
4. Run O76/O90 and lifecycle regressions, strict Clippy/rustfmt/LOC, update docs,
   commit/push.

## Completed evidence

Compile RED proved the stage missing. Full KSR RED then exposed three upstream
prerequisite topology rules; after their source-cited fixes, GREEN passes 3/3
O91 plus O79/O80/O90 regressions. Strict core Clippy, rustfmt, diff, and LOC
gates pass.

No thin fills, motion, fallback, or G-code.
