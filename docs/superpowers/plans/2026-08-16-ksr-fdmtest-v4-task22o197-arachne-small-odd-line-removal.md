# Plan: KSR FDM Test V4 task197 Arachne small odd-line removal

1. Add a focused failing test containing short and threshold-surviving odd open lines.
2. Port source walk-length comparison, minimum-width threshold selection, and swap removal in a child postprocess module.
3. Apply removal after stitching; run wall-toolpath/extrusion-line tests, line-count checks, formatting, and workspace Clippy.
4. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
