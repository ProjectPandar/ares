# ARD-0040: Preserve processor filament-volume precision

The upstream boundary is `GCodeReader.cpp::parse_line_internal`,
`GCode/GCodeProcessor.cpp::process_G1`,
`UsedFilaments::increase_model_caches` and `run_post_process`'s volume-to-mm
conversion. The Rust destination is
`project_slice/gcode_emit/finish.rs::account_used_filament` and its emitter
caller. This is a temporary single-filament compatibility shell around
upstream processor accounting, not an Ares-owned statistics pipeline.

Parse axis numbers as f64 then store as f32 before promotion into processor
f64 positions, matching `GCodeReader::GCodeLine`'s axis storage.
Update the double endpoint before subtracting the prior position, including
relative moves; do not bypass that arithmetic by using the operand as delta.
For currently classified extrusion moves, multiply the f64 E displacement
by the widened f32 filament cross-sectional area, narrow the product to f32,
then widen each f32
volume for f64 accumulation. Convert total volume to length using upstream's
f64 area computed from the stored f32 filament diameter. Do not replace this
with algebraically equivalent direct length accumulation or fused arithmetic.

Include the precision boundary and configured filament diameter. Defer
multi-filament accounting, move classification, inch units and arc
discretization to their own source slices; do not claim full processor parity.
Complete official G-code comparison remains the acceptance gate, including
statistics. Existing phase-regression output must pass without normalization
changes. Existing positioning-mode tests retain their behavioral coverage.
