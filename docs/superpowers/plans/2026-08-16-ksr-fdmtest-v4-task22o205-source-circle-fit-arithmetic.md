# Plan: KSR FDM Test V4 task205 source circle-fit arithmetic

1. Add a failing high-coordinate three-point circle test whose source-ordered and rearranged formulas differ by microns.
2. Replace determinant/numerator rearrangement with the literal `a`, `b`, `c`, and `-b/(2a)`, `-c/(2a)` source expressions.
3. Remove `[DEBUG-e205]` instrumentation and rerun fixture normalized comparison past timing/object metadata.
4. Run focused arc/motion tests, line-count checks, formatting, and workspace Clippy.
5. Record the source-cited arithmetic fix in `docs/roadmap.md`, commit, and push independently.
