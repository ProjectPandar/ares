# Task 22O.92 — thin fill append

Port pinned `Fill.cpp:1376-1384`. Move retained layer-region thin-fill entities
after generated fill collections into each aligned O91 layer, preserving entity
order, path/loop nesting, points, roles, and flow metadata.

Focused KSR tests cover nonzero all-layer inventory, exact deterministic
path/loop/point/metadata checksum, source draining, repeatability, disposal, and
public lifecycle. Separate modules, <400 LOC, no source-splitting macros.

Deferred: island ordering, motion, G-code.
