# Plan: KSR FDM Test V4 task198 Arachne inner-contour separation

1. Add a focused failing test with zero-width closed even, zero-width odd, and positive-width inset groups.
2. Add the even-odd polygon-union wrapper and port source inset classification and contour extraction.
3. Return printable toolpaths and inner contours from the wall pipeline; update focused callers.
4. Run wall/Clipper tests, line-count checks, formatting, and workspace Clippy.
5. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
