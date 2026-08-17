# Plan: Task 22O.143 seam-gap clip ordering

1. Tighten the existing KSR seam-gap behavior test with the exact first inner-wall terminal extrusion; confirm the current simplify-before-clip order fails it.
2. Move end clipping ahead of option-gated arc fitting and simplification in `motion::path::emit`, matching OrcaSlicer loop export order.
3. Run the Task 22O.141 simplification assertion, the Task 22O.143 clipping assertion, strict Clippy, rustfmt, then commit and push this slice.
