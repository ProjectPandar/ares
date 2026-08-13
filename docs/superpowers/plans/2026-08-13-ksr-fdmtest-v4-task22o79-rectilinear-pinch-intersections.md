# Task 22O.79 implementation plan

1. Add RED no-op and synthetic disconnected-inner-pair tests.
2. Add phony identity to intersection state and implement source scan/insertion.
3. Implement current/previous/next link reindexing with focused witnesses.
4. Run focused regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing pinch module. GREEN passes 2/2 O79 tests and all
5 O77/O78 regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No monotonic traversal, filler/entity output, lifecycle, fallback, or G-code.
