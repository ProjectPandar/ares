# Plan: Task 22o.122 single final-layer timelapse

1. Add a failing KSR assertion that counts the configured timelapse template once per printed layer.
2. Remove the obsolete export-finalization rendering now that the per-layer emitter covers the final layer.
3. Run the focused KSR test, rustfmt, and clippy; commit and push.
