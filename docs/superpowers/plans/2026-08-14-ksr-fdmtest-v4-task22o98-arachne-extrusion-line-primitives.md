# Task 22O.98 implementation plan

1. Add RED tests for junction/line mutation, width layout, area, deviation, and simplification.
2. Add crate-private Arachne junction and extrusion-line types.
3. Port source integer length, conversion, orientation, and simplification order.
4. Run focused, formatting, strict Clippy, diff, macro, and LOC gates.
5. Update evidence; parent reviews, integrates, and pushes.

## Completed evidence

Ten focused tests pass across the accepted source boundary, including active-
scale tolerances, per-segment truncation, closed spill/closure repair,
replacement payloads, and out-of-range intersection rejection. Rustfmt, strict
core Clippy, diff, macro, and LOC gates pass; implementation/test shards are
336/208 LOC. The module remains an inactive prerequisite; no half-edge,
beading, skeletal, wall-toolpath, concentric-fill, lifecycle, motion, or G-code
behavior is claimed.
