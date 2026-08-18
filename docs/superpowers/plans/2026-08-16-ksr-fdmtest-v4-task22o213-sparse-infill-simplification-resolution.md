# Plan: KSR FDM Test V4 task213 sparse infill simplification resolution

1. Add a failing focused role-to-tolerance test.
2. Port the source 0.04 mm sparse constant and select it only for `InternalInfill` fill paths.
3. Keep configured resolution for every other fill, perimeter, and gap path.
4. Run arc/path and sparse fill tests; slice the fixture and compare sparse moves/counts.
5. Run line-count checks, formatting, and workspace Clippy; update `docs/roadmap.md`, commit, and push independently.
