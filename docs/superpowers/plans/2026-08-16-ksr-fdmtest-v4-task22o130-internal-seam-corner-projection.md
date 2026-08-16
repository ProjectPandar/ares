# Plan: Task 220.130 internal seam corner projection

1. Add a focused internal-corner loop assertion whose expected split point distinguishes OrcaSlicer's candidate-relative overshoot from fitted-position-relative placement.
2. Port OrcaSlicer's fitted-displacement guard and candidate-relative concave overshoot into `place_loop` without changing outer-loop placement.
3. Verify the focused source invariant and the retained bounded KSR seam contract; run rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the isolated slice.
