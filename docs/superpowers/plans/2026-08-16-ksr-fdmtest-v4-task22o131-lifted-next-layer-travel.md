# Plan: Task 220.131 lifted next-layer travel

1. Add a red KSR assertion that the second-layer first-object travel has `F60000`, no Z word, and is followed by the explicit `G1 Z.4` lowering move.
2. Make lifted and unlifted XY travel share the source XY/feedrate command; retain lift state until the existing lowering and deretraction sequence.
3. Regenerate KSR output, verify every post-first-layer label travel omits a recomputed hop Z, then run focused tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the isolated slice.
