# ARD-0039: Preserve upstream cross-zag phase precision

The rewrite boundary is `Fill.cpp:1311–1316` in
`Layer::make_fills`, `libslic3r.h::scale_`, and
`FillBase.hpp::FillParams::horiz_move`. The Rust destination is
`project_slice/group_fills/params/projection/sticky.rs::apply_pattern_fields`.
This adapter remains a temporary compatibility shell around upstream fill
parameter construction, not an independently designed slicing pipeline.

For cross-zag and locked-zag, scale the double option and multiply by the
integer layer-pair index in f64 before narrowing the completed phase to f32.
Retain the alternating sign and upstream float storage. Do not round the
step early, fuse operations, or compensate output coordinates.

Included behavior is phase generation only. Rectilinear phase consumption
is covered by ARD-0038. Other fill patterns, locked-zag depth parameters,
bridge ordering, adaptive pressure advance and remaining KSR differences
are deferred. Acceptance requires complete G-code against the unmodified
official oracle; neither private arithmetic tests nor source pinning suffice.
