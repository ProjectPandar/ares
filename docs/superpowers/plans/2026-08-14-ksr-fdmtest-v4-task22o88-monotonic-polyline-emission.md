# Task 22O.88 implementation plan

1. Add RED exact rectangular zigzag and empty tests.
2. Port path transition, vertical-run, and O83 perimeter emission.
3. Port finish, near-zero removal, and O79 phony split merge.
4. Run O79-O87 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing emitter. GREEN passes 2/2 O88 and all three O87
regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No full filler/rotation, extrusion entities, lifecycle, fallback, or G-code.
