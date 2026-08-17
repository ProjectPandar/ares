# Plan: Task 22O.151 full-pivot aligned-seam precision

1. Add failing focused KSR assertions for the first clipped outer-wall extrusion and its terminal wipe retraction.
2. Clip loop endpoints in scaled integer coordinates, preserving OrcaSlicer's truncating cast before unscaling for emission.
3. Port seam placement to scaled f32-to-integer target and segment projection semantics.
4. Replace the modified Gram-Schmidt spline solver with full-pivot Householder QR matching the cited upstream fitting boundary.
5. Run the focused seam, motion, and KSR contracts; generate the CLI slice and identify the next exact divergence.
6. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
