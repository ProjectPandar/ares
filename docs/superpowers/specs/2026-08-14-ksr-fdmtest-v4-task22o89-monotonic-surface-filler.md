# Task 22O.89 — monotonic surface filler

Port pinned `FillBase.cpp:255-324` and `FillRectilinear.cpp:2755-2908,3404-3421`.
Given explicit angle/layer/bridge, spacing/overlap/density, solid adjustment,
link, and scale parameters, generate inverse-rotated monotonic or monotonic-line
polylines through O79-O88.

Focused tests cover exact rectangle output, angle/rotation, odd/even layer
alternation, solid spacing adjustment, density, empty erosion, both scales,
range errors, repeatability, and input immutability. Separate modules, <400 LOC,
no source-splitting macros.

Deferred: grouped entities, lifecycle, motion, G-code.
