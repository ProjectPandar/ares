# Task 22O.91 — layer fill entity stage

Port pinned `Fill.cpp:1213-1384` project/layer ownership. Consume post-combination
state, materialize every object/layer in aligned order through O90, preserve
complete entity metadata and geometry, and provide transactional disposal.
Advance the public lifecycle once, retaining `ProjectSlicingIncomplete` only
after successful O91 disposal.

Focused tests cover object/layer order, None/empty layers, repeatability,
predecessor ownership, late-error rollback, invocation/disposal counts, and
public lifecycle precedence. Separate modules, <400 LOC, no source-splitting
macros.

Deferred: thin fills, perimeter/fill ordering, motion, G-code.
