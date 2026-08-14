# Task 22O.84 — source rectilinear links

Port pinned `FillRectilinear.cpp:994-1214`, replacing O78 approximations. Use
O82 contours and O83 directed distances/arc lengths. Preserve adjacent and
same-line candidate order, strict ties, wraparound, link type, skipped-inner and
same-side invalidation, `dont_connect`, exact `link_max_length`, symmetric
invalid quality, and immutable geometry.

Focused tests cover wraparound selection, concave/multiple same-line cases,
horizontal symmetry, valid/invalid/too-long quality, deterministic repeatability,
and O79-O81 corrected-topology regressions. Separate test modules, <400 LOC, no
source-splitting macros.

Deferred: region costs/chaining, polylines/entities, lifecycle, G-code.
