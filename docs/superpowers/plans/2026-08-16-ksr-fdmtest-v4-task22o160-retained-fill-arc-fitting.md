# Plan: Task 22O.160 retained fill arc-fitting payload

1. Add a failing KSR contract for the first bottom-surface contour arc currently emitted as a straight chord.
2. Trace the source contour vertices through monotonic emission and the fill-path simplification stage.
3. Extend the fill extrusion path seam with retained fitting ranges and preserve their indices and direction during reversal.
4. Store the simplifier result on fill paths and pass it to retained-arc G-code emission.
5. Run focused KSR motion and shortest-path contracts, rustfmt, Clippy, and file-size checks.
6. Record the roadmap milestone, commit, and push independently.
