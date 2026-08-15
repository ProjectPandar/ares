# Task 22O.104 architecture decision record

## Source boundary

Port the typed project-facing portion of OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/PlaceholderParser.hpp/.cpp`, as consumed by
`src/libslic3r/GCode.cpp` machine-start and machine-end template call sites.

## Decision

Add a crate-private renderer for the resolved project G-code templates. Its
variables are built from the typed 3MF configuration block plus derived runtime
values; it supports nested `if`/`elsif`/`else`/`endif`, scalar and indexed
values, arithmetic, comparisons, boolean operators, and the source functions
needed by the project template. It must not read the reference G-code or carry
fixture-specific values.

The renderer replaces the existing direct string replacement shell only for the
project lifecycle. Legacy `SliceOptions` placeholder behavior remains isolated
until its own source-cited migration. Machine-start/end orchestration,
timing/statistics, and exact writer semantics remain separate follow-up slices.
