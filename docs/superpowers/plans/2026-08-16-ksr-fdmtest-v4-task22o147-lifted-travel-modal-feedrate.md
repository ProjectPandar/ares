# Plan: Task 22O.147 lifted-travel modal feedrate

1. Tighten the focused KSR assertion to the reference XYZ move without a redundant feedrate and observe failure.
2. Distinguish the travel that consumes a newly emitted retract lift from an already-lifted lifecycle travel.
3. Omit the feedrate only for the former, then run the focused inter-path and next-layer contracts.
4. Run rustfmt and Clippy, commit, and push the slice independently.
