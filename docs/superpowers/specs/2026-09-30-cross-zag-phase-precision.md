# Cross-zag phase precision rewrite slice

Port the double-expression-to-float-assignment boundary from upstream
`Fill.cpp:1311–1316`, `libslic3r.h::scale_`, and
`FillBase.hpp::FillParams::horiz_move` into
`project_slice/group_fills/params/projection/sticky.rs::apply_pattern_fields`.
The existing Rust projection is a temporary compatibility shell, not a new
Ares pipeline. Architectural constraints are in ARD-0039.

Include cross-zag/locked-zag layer-pair multiplication and alternating phase.
Defer downstream grid arithmetic (already ported), unrelated fill options,
bridge/timelapse work and broader printer/option completion.

Use the public CLI byte-input/full-output oracle seam. A tall slab with
infill_shift_step=0.0655373 must retain that effective option and actual
sparse extrusion through the failing layer, then match complete official
G-code with only the existing generator identity/time normalization.
Retain existing solid, signed cross-zag and locked-zag cases. No fixture
replacement, comparator relaxation or patched official binary is permitted.

Exit requires red/green evidence, fmt/clippy, workspace nextest, Tier-1
compilation checks and independent six-axis review of the isolated slice.
