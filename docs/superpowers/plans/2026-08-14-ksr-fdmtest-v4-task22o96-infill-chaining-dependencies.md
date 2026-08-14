# Task 22O.96 implementation plan

1. Add RED shortest-path entity tests and pattern no-sort assertions.
2. Add source-owned `no_sort` to fill collections.
3. Generalize classic shortest path with explicit-cursor constrained reversals
   and source fallback ordering.
4. Add fill/gap endpoint, reverse, generic reorder, and `chained_path_from`
   operations without runtime activation.
5. Run focused and relevant regressions, strict core Clippy, rustfmt, diff, macro,
   and LOC gates; update evidence.

## Completed evidence

Four focused entity tests and all ten shortest-path regressions pass. KSR
freezes 782 no-sort and 876 sortable generated collections with valid endpoints.
Strict core Clippy, rustfmt, diff, and LOC gates pass; the largest changed Rust
shard is 383 LOC.

Deferred: O95 activation and its real cursor, multi-region role filtering,
motion, and G-code. No fixture branch or compatibility fallback.
