# Plan: Task 22O.153 scaled object-origin arithmetic

1. Add a failing KSR assertion for the retained fitted arc endpoint at `X104.96`.
2. Quantize the 3MF-derived model center through the active `CoordinateScale` before constructing `EmitState`.
3. Continue adding already-scaled local points to that unscaled integer origin in the motion emitter.
4. Run focused KSR and G-code motion contracts, generate the CLI slice, and locate the next exact divergence.
5. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
