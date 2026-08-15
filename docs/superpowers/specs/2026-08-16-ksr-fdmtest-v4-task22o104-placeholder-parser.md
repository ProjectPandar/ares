# Task 22O.104: typed project placeholder parser

## Requirements

- Source-cite Orca `PlaceholderParser.hpp/.cpp` and the project G-code call
  sites; do not invent fixture branches or read reference G-code.
- Parse values from the resolved typed project configuration, preserving scalar
  strings, numbers, vectors, and indexed vector access.
- Render nested conditional template blocks and replace `{expression}` and
  `[name]` placeholders with deterministic values.
- Support the arithmetic/comparison/boolean/function subset exercised by the
  3MF machine-start template (`min`, `max`, `ceil`, indexing, `&&`, `||`,
  equality and relational operators).
- Keep parser, renderer, and tests in separate ordinary modules below 400 LOC;
  no `include!` or `include_bytes!` source splitting.
- Preserve malformed template errors at the project boundary rather than
  silently emitting unresolved control directives.

## Deferred

Full Orca dynamic user-variable mutation, random functions, all custom G-code
contexts, time/stat placeholders, machine-end orchestration, and writer/motion
parity are deferred.

## Acceptance

Unit tests cover nested branch selection, indexed vector arithmetic, functions,
unknown placeholders, and syntax errors. The KSR golden first difference moves
past the machine-start template; exact parity remains deferred to later writer
and timing slices.
