# Plan: Task 220.139 reverse extrusion-path wipe

1. Use the existing KSR layer-zero timelapse assertion to reproduce the forward-path wipe jump.
2. Traverse the retained emitted path from its final endpoint toward its start when constructing wipe moves.
3. Verify the exact KSR wipe endpoint and timelapse block, then run focused G-code tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
