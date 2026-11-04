# Placeholder int() integer result type

## Source-cited Rust rewrite boundary

Port OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:353-362`,
`expr::unary_integer`, into the `int` arm of
`crates/ares-core/src/project_slice/gcode_emit/expression/functions.rs`.
Upstream TYPE_INT passes through; TYPE_DOUBLE uses `static_cast<int>`
(truncation toward zero) and returns TYPE_INT. The grammar action at
`:2071-2072` calls this conversion. `expr::operator/=` at `:494-510`
uses integer division unless either operand is TYPE_DOUBLE.

The existing Rust `Value::Number(trunc())` is only a temporary compatibility
shell: printing can appear correct but subsequent arithmetic observes the
wrong type. Replace only this arm with Value::Integer, keeping integer
arguments intact and converting floating values through upstream's i32 int.
Existing integer arithmetic requires no change.

## Included and deferred

Include only int() result typing and its existing truncating conversion.
Do not alter round/floor/ceil, digits, other conversions, argument admission,
error reporting, lexer/parser or arithmetic. Out-of-range C++ float-to-int
behavior is deferred; no fallback, new dependency or compatibility layer.

## E2E acceptance

GUI Machine start G-code comment `{5 / 2}` is the control;
`{int(5.1) / 2}` is the positive. Shared Native stream comparison must show
positive line27 mismatch before implementation and full byte equality after
implementation, in official and seeded modes. Existing generator identity/date
normalization only. Both INPUT3MFs are real official Native CLI exports.

This slice builds on the digits() integer rounding slice.
