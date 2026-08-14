# Task 22O.97 implementation plan

1. Add RED focused winding/angle/closing/mixed-role/flow tests and KSR oracle.
2. Define pure candidate/perimeter/layer topology.
3. Port source external-loop extraction and `collect_points` semantics.
4. Port counter-clockwise normalization and 0.4 mm arm vertex angles.
5. Freeze KSR inventory/checksum and run focused/dependency/strict gates.
6. Update evidence; parent commits and pushes.

## Completed evidence

Five focused/KSR tests pass, including differing per-region external-flow
ownership. KSR freezes 3,272 perimeters, 62,094 candidates, and FNV-1a checksum
`11805973356074762675`. Strict core Clippy, rustfmt, diff, macro, and LOC gates
pass; implementation/test shards are 233/261 LOC.

No visibility, penalties, selection, alignment, placement/clipping, cursor,
O96 activation, legacy seam fallback, or G-code.
