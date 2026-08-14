# Task 22O.94 — extrusion island assignment

Port pinned `GCode.cpp:4970-5048` for KSR's single region/tool. Assign O93 fills,
thin fills, and perimeters to ordered layer `lslices` by increasing bounding-box
area, half-open bbox containment, and contour containment; retain a final
fallback island. Preserve fill-before-thin source inventory and within-kind
order.

Focused tests freeze KSR island/nonempty/fallback/entity inventory,
repeatability, disposal, and public lifecycle. Separate modules, <400 LOC, no
source-splitting macros.

Deferred: multi-region/tool/wiping/split roles, island chaining, motion, G-code.
