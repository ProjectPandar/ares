# Plan: KSR FDM Test V4 task196 Arachne wall-line stitching

1. Add a focused failing four-edge even rectangle test in a child stitch module.
2. Port endpoint-grid lookup, shortest candidate selection, reversal/connect rules, two-direction extension, closure thresholds, and closed-line marking.
3. Apply stitching after raw generation; run Arachne wall-toolpath/extrusion-line tests, line-count checks, formatting, and workspace Clippy.
4. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
