# Filament volume precision rewrite slice

Port `GCodeReader.cpp::parse_line_internal`'s double-to-float axis storage,
processor double endpoint subtraction, and the f32 per-move volume / f64 accumulation boundary of
`GCode/GCodeProcessor.cpp::process_G1`, `UsedFilaments::increase_model_caches`
and `run_post_process` into `gcode_emit/finish.rs::account_used_filament`.
The existing single-filament adapter is a temporary upstream compatibility
shell; ARD-0040 defines included and deferred behavior.

The 22.4 mm cross-zag public CLI fixture already has identical motion after
the phase fix but differs in final length: official 18725.54 mm versus
Rust 18725.53 mm. It must become complete-output GREEN against the unmodified
official binary. Preserve the four earlier fill-grid cases, relative/absolute
mode behavior, and configured filament diameter. Do not loosen comparisons.

Exit requires real red/green evidence, independent six-axis review,
workspace nextest, fmt/clippy and Tier-1 compilation. Global printer/option
coverage remains a separate incomplete goal.
