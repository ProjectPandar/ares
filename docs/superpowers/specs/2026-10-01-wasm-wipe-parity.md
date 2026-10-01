# WASM wipe parity investigation

Current source boundary: `GCode/SeamPlacer.cpp:42-59::gauss/compute_angle_penalty` and
`:1237-1392::SeamPlacer::align_seam_points`. Actual destinations are
`project_slice/seam_placement.rs::angle_penalty` and
`seam_placement/alignment.rs::finalize_string`. Include portable f64 evaluation
of dynamic exp and the cube with one cast to the upstream float result;
preserve surrounding f32 arithmetic. Wipe/clip/format and QR remain unchanged.
Initial investigated boundary was `GCode.cpp::Wipe::wipe`,
`Polyline.cpp::clip_end`, and `GCodeWriter.cpp::GCodeFormatter::emit_axis`;
captured inputs eliminate those owners for the first divergence. Existing
export scaffolding verifies the cited upstream behavior, not an independent
Ares pipeline. The numerical implementation targets the identified native
math behavior, not a guarantee of equivalence for every possible libm/input.

Include identifying and fixing the reproducible first KSR WASM wipe difference
without comparator changes, epsilon bias, reference regeneration or native
output substitution. Public input/complete-output tests remain the approved
seam. Retain full independent native and actual WASM results. Any adjacent
geometry/control differences remain explicit open gates until verified.

Require causal reproduction, witnessed RED/GREEN, removal of temporary
diagnostics, Rust files below400LOC/separate test modules, fmt/Clippy,
workspace Nextest, Tier-1 evidence and independent six-axis review before
code/docs commit together. All-printer/default/all-option acceptance remains
the overall goal, not proven by this narrow repair.
