# Placeholder round/floor/ceil integer results

## Upstream rewrite boundary

Port OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:371-415`,
`expr::round`, `expr::floor`, `expr::ceil`, to the same named function arms in
`crates/ares-core/src/project_slice/gcode_emit/expression/functions.rs`.
Each upstream method passes TYPE_INT through and converts TYPE_DOUBLE with
`static_cast<int>(std::round|floor|ceil(d))`, yielding TYPE_INT. Round uses
half-away-from-zero rounding. `operator/=` at :494-510 uses integer division
unless either operand is TYPE_DOUBLE.

Ares currently returns Value::Number for all three. This is a temporary
compatibility shell: direct formatting can agree but arithmetic sees the
wrong type. Replace those arms with Value::Integer, preserving existing
integers and casting rounded floating results through upstream i32 int,
following the approved int() slice. No arithmetic or value-model changes.

## Scope and exclusions

Include ONLY result typing/conversion for these three functions. Keep int(),
digits(), parser, argument admission and other functions unchanged. Defer
out-of-range C++ conversion behavior, errors and unrelated numeric semantics.
No dependencies, fallback, include!, allow attributes or expected-output data.

## E2E acceptance

Use real official Native-exported INPUT3MFs with GUI Machine start G-code
comments. Shared comparator control `{int(5.9) / 2}` must pass. Positive
`{round(5.5) / 4}`, `{floor(5.9) / 2}` and `{ceil(5.1) / 4}` must each
show a real Native/Ares byte difference before implementation and pass after,
in official and seeded modes. Half-tie round input also exercises upstream
half-away behavior. Only existing generator identity/date normalization.

Initial floor discovery provenance:
`${COORD_ROOT}/evidence/w5-808db-floor-discovery/PROPOSAL-RR.md`.
Stack on approved digits+int tip 808dbd460 for combined integration.
