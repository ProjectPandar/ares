# Plan: Task 22O.105 project layer-change prologue

1. Add source-cited preamble framing before the first resolved layer; verify the
   generated output reaches the first layer without placeholder errors.
2. Evaluate `layer_change_gcode` with typed layer index and Z context; verify
   focused template tests and the project golden's first executable block.
3. Run formatting, clippy, focused nextest, LOC and diff checks; commit and push
   the increment before the next independent review attempt.
4. Record timing, motion-state, and convex-hull gaps as deferred follow-up slices.
