# Plan: Task 220.138 cooling-buffer redundant feedrates

1. Add a red KSR assertion that the first variable-speed inner wall retains one precise restored feedrate rather than repeating it for every segment.
2. Add a per-layer cooling rewrite that tracks integer feedrate state, drops redundant standalone G0/G1 feedrate commands, and removes redundant inline `F` words without altering motion.
3. Regenerate KSR output and compare feedrate frequencies and the first dynamic block, then run focused G-code tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
