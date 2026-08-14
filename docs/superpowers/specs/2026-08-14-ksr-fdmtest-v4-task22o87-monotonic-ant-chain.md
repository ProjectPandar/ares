# Task 22O.87 — monotonic ant chain

Port pinned `FillRectilinear.cpp:2190-2582` over O81 dependencies, O85 costs,
and O86 matrix. Implement default MT19937-64, greedy initialization, source ant
round/queue/probability/pheromone order, strict best replacement, and no-op
3-opt. Return every region once with precedence preserved and source-selected
orientation.

Focused tests cover empty/single, linear precedence, branching joins,
deterministic exact path/orientation, repeatability, and immutable inputs.
Separate modules, <400 LOC, no source-splitting macros.

Deferred: polyline/entity output, lifecycle, G-code.
