# Plan: Task 22o.113 layer-change fan-speed marker

1. Add a failing project-output test for the exact KSR `M991`, blank separator, fan marker, and acceleration sequence, plus the 460-layer marker count.
2. Port the unconditional marker and conditional post-template separator from `GCode::process_layer` into `append_layer_change`.
3. Run the focused project-output test, smoke-slice the KSR 3MF, then run rustfmt and clippy before committing and pushing.
