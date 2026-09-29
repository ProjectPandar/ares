# ARD-0038: Preserve upstream float rectilinear grid shifts

The source boundary is `FillBase.hpp::FillParams::horiz_move` and
`FillRectilinear.cpp::fill_surface_by_lines` (2811–2816). The Rust boundary
is `fill/rectilinear/surface.rs::MonotonicFillParams` and its callers in
`project_slice/fill_entities`, raft and sparse anchoring. These existing
fill adapters are compatibility shells around the upstream fill concepts,
not an independently designed Ares fill pipeline.

Keep horizontal movement as f32 through the caller chain. Compute gap-line
division in f32, truncate to upstream int, retain integer multiplication
before conversion back to f32, and perform the final x0 addition in f32
before conversion to coord_t. Execute this arithmetic even for zero shift.
Do not fuse operations, add coordinate compensation, or special-case KSR.
Odd gap-line shifts retain upstream's extra grid column.

Acceptance is complete G-code parity against the unmodified official oracle,
through the existing byte-input/output test seam. Debugger counterfactuals
establish causality, not acceptance. No new normalization of movement,
extrusion, time, M73 or configuration is allowed. Existing KSR reference and
unrelated upstream undefined behavior remain separately tracked gates.
