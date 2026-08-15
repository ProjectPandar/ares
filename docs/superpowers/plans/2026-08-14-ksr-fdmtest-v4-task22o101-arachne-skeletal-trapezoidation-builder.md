# Task 22O.101 implementation plan

## Source boundary

Port pinned OrcaSlicer
`Arachne/SkeletalTrapezoidation.hpp` and
`SkeletalTrapezoidation.cpp:98-745` into crate-private
`ares-core::arachne::trapezoidation`, with only the consumed polygon index and
Voronoi utility behavior. Reuse O99/O100. Stop before transition generation and
leave all runtime slicing inactive.

## Steps

1. Add source-worked tests for both coordinate scales, holes, curve
   discretization, pointy-node splitting, central recursion, and noncentral
   dissolution.
2. Add stable polygon segment indices and pinned source-category/range helpers
   over the existing `boostvoronoi` dependency.
3. Transfer valid cell edges into O100 while preserving construction order,
   discretized ribs, twin reconstruction, endpoint ownership, and collapse.
4. Port point/segment and point/point discretization with source conversion
   order.
5. Port initial central marking, recursive filters, bead counts, and short
   noncentral-region dissolution through the exact O101 stop line.
6. Run focused Nextest, rustfmt, strict core Clippy, diff, macro, and LOC gates;
   keep O101 inactive and record evidence.

## Implementation state

The inactive implementation and six focused tests are written. This managed
worker exposes file tools but no command runner, so compilation and quality
gates remain pending parent validation.
