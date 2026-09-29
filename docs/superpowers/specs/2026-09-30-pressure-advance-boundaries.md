# Active pressure advance boundary coverage

## Source boundary and destination

Exercise the rewrite of `PrintConfig.cpp:2345` (`pressure_advance`, coFloats,
upper bound 2), `GCode.cpp:3156-3157` (enable gate) and
`GCodeWriter.cpp:370-393::set_pressure_advance` (negative suppresses output,
zero emits a reset, positive emits the configured value).
Rust implementation is `project_slice/gcode_emit/layer_gcode.rs`; this slice
corrects its CLI parity coverage at `option_coverage/domains/probes.rs` and
`activation.rs`, without adding slicing behavior. The existing emitter
remains a temporary compatibility shell around upstream GCode concepts.

Replace hand-picked 0.01/0.2 probes with the existing source-derived ceiling
and deterministic random interior. Enable the filament's PA and disable
adaptive PA so the value is actually consumed. A separate public slicing
regression also covers zero and negative suppression. No declared minimum
exists; zero is a behavioral boundary, not an invented schema minimum.

Deferred: adaptive PA, other firmware dialects and multi-filament combinations,
other incomplete option domains, and untouched default-printer coverage.

## Acceptance

First reproduce missing maximum/activation with a public full-output test.
Verify requested values and dependencies in the exported project, check the
official PA command and compare complete Ares output. Run focused and workspace
Nextest, fmt and Clippy. Independent six-aspect review must approve before
code/tests/docs are committed and pushed together.
