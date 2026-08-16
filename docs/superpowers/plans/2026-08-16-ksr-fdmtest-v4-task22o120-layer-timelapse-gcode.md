# Plan: Task 22o.120 per-layer timelapse G-code

1. Add failing KSR assertions for the timelapse count and first/final rendered motion lines.
2. Expose transformed model bounds and select the source-compatible single-object safe position.
3. Populate runtime timelapse placeholders and render the 3MF template after each layer.
4. Smoke-slice KSR, run the focused nextest, rustfmt, and clippy; commit and push.
