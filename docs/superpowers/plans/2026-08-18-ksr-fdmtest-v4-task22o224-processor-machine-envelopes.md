# Plan: KSR FDM Test V4 task224 processor machine envelopes

1. Add failing processor-seam tests for M203-capped elapsed time, M201-capped acceleration, and shared collinear junction speed.
2. Split motion state and planner code from `processor.rs` into a normal Rust module before extending it.
3. Parse emitted M201/M203 commands and clamp block speed/acceleration per active axis; use retract acceleration for every extrusion-only move.
4. Correct the junction-entry index so each block receives the limit computed between it and its predecessor.
5. Run focused processor tests and the complete KSR slice, record the changed time estimate, then run formatting and file-size checks before committing and pushing independently.
