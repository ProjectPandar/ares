# Plan: KSR FDM Test V4 task206 monotonic three-opt path pass

1. Freeze the missing pre-measurement `monotonic_3_opt` call through source comparison and the focused monotonic chain suite.
2. Port the source consecutive-link dependency guard, three-edge before/after cost comparison, strict improvement rule, and in-place middle swap.
3. Invoke the pass on every ant path before path length and best-path selection.
4. Re-run the fixture normalized structural comparison and record whether the first divergent arc/path changes.
5. Run line-count checks, formatting, and workspace Clippy; record the source slice in `docs/roadmap.md`, commit, and push independently.
