# Plan: Task 22O.106 first-layer print bounds

1. Replace emitted-path bounds with transformed model-part bounds; verify the
   placeholder values change with project geometry.
2. Run focused parser/project checks, clippy, formatting, LOC and ignored golden
   comparison; record the remaining convex-hull delta.
3. Commit and push the source bridge before implementing the deferred Orca hull.
