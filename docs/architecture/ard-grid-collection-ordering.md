# ARD: separate Grid path ordering from reversibility

Non-negotiable upstream contract:
`Fill::no_sort` (`FillBase.hpp:166`) remains false for Grid, while
`Layer::make_fills` (`Fill.cpp:1324-1325`) sets `params.can_reverse=false`.
`Fill::fill_surface_extrusion` (`FillBase.cpp:164,183-185`) propagates both
independently. `FillGrid::fill_surface` still applies its odd-layer reversal
before materialization (`FillRectilinear.cpp:3441-3443`).

Rust's Grid compatibility shell currently uses no_sort=true to avoid reversal,
which also freezes multi-path order. Use its existing Oriented path variant
and no_sort=false instead. Do not add another flag/model, change generic chaining,
or modify generation orientation. The existing Oriented variant implements the
needed non-reversible entity semantics; there is no new Ares planning algorithm.

The first isolated no_sort=false experiment regressed111/823 controls, exposing
why both independent semantics matter. It is rejected, not silently retained.
The corrected two-choice mapping must pass original complete output and all
controls. Timing/progress are included in acceptance, never filtered away.
All adjacent fill geometry, bridge, cooling and perimeter behavior is deferred.
