# Plan: KSR FDM Test V4 task231 centripetal acceleration

1. Add a failing focused processor test that distinguishes print acceleration from travel acceleration on a parsed travel block.
2. Carry both acceleration values through the motion block without changing its move-type trapezoid acceleration.
3. Apply print acceleration only to the shallow XY turn centripetal speed ceiling, matching OrcaSlicer.
4. Regenerate the complete KSR fixture and record timing, M73 count, output counts, and the next normalized divergence.
5. Run focused processor and complete-slice tests, formatting, and focused Clippy; commit and push independently.
