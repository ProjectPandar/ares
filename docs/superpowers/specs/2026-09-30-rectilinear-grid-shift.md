# Rectilinear float grid-shift parity

## Source and destination

Port `FillBase.hpp::FillParams::horiz_move` and the grid-origin shift in
`FillRectilinear.cpp::fill_surface_by_lines` to Rust
`fill/rectilinear/surface.rs`, preserving f32 through its existing callers.
The existing fill adapters remain temporary upstream compatibility shells.

## Included behavior

- Zero shifts still round the grid origin through f32.
- Signed nonzero shifts preserve f32 division and subtraction, integer
  gap-line multiplication, final f32 addition and integer truncation.
- Odd gap lines add a column; fractional shifts are not discarded in callers.
- Monotonic, rectilinear, cross-zag and locked-zag use the same source rule.

## Deferred behavior

Other KSR discrepancies, CLI configuration bookkeeping, upstream uninitialized
Arachne fields, unrelated fill algorithms and the full printer/option sweep
remain open. This repair does not redefine their completion criteria.
The separate `Fill.cpp:1311–1316` double-to-float phase construction is also
deferred: `group_fills/params/projection/sticky.rs` still narrows before
multiplying by the layer-pair index. This slice preserves the supplied f32
phase through callers and consumes it correctly; it does not claim that
phase generation has identical precision for every representable step.

## Acceptance

At the already agreed public slicing seam, a wide solid model must reproduce
the bug before repair and match complete official G-code after repair.
Exercise signed cross-zag shifts and existing fill callers, then run nextest
workspace, fmt, clippy and Tier-1 cross-target checks. Independent review must
validate numeric order and actual runtime output before commit/push with docs.
