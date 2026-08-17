# Plan: KSR FDM Test V4 task199 Arachne variable-width simplification

1. Add a focused failing collinear open-line test in an extrusion-line simplification child module.
2. Port accumulated shoelace-area simplification, width-area error limits, and long-successor intersection adjustment.
3. Apply source default scaled thresholds after short-line removal and before inner-contour separation.
4. Run extrusion-line/wall tests, line-count checks, formatting, and workspace Clippy.
5. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
