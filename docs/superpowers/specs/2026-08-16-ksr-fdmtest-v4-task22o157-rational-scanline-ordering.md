# Spec: Task 22O.157 exact rational scanline ordering

## Observable contract

Rectilinear scanline intersections are ordered by their exact rational Y positions before coordinates are rounded for emitted geometry. Distinct intersections that round to the same integer coordinate retain their geometric order, so contour link construction and monotonic-region traversal see the same alternating intersection kinds as the reference slicer.

The monotonic region builder no longer needs the Ares-only next-zigzag validation gate. Region extension is determined solely by the source right-overlap, vertical-run, and exclusive left-overlap checks.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:120-138`, `759-911`, and `1855-1932`: retain each scanline intersection numerator and denominator, compare rationals before rounding, and use the unextended `generate_montonous_regions` loop.

Ant-chain optimization, fill flow width, cooling, timing, and later exact G-code differences are deferred.
