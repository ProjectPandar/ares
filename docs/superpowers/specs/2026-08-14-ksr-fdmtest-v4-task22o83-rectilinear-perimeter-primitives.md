# Task 22O.83 — rectilinear perimeter primitives

Port pinned `FillRectilinear.cpp:38-116,459-685` over O82 retained contours.
Implement directed segment distance, contour arc length, forward/reverse vertex
append, adjacent-line horizontal arc measurement/emission, and same-line
vertical arc measurement/emission.

Focused tests cover forward/reverse, wraparound, same segment, holes,
adjacent/same lines, exact point inclusion, and deterministic f64 length bits.
Separate test modules, every source <400 LOC, no source-splitting macros.

Deferred: corrected link construction, region costs/chaining, entities,
lifecycle, G-code.
