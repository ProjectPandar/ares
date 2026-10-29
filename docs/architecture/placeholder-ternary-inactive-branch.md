# Ternary inactive-branch suppression rewrite slice

## Source boundary

This slice ports the conditional-expression branch suppression of
`OrcaSlicer/src/libslic3r/PlaceholderParser.cpp`: `conditional_expression`
at lines 2215-2221 wraps each `?`/`:` branch with
`MyContext::block_enter`/`block_exit_ternary`, which maintain
`m_depth_suppressed` (lines 851-871). While skipping, every evaluation
action no-ops instead of throwing: `variable_value` (1348-1355),
`vector_element_to_expr` (1092-1094), the `expr` arithmetic operators
(435-530), `compare_op` (558-562), `function_2params`/`digits`/`random`
(617-668), and `evaluate_index` (1699-1703). Grammar-level failures are
not suppressed: the parse itself still has to succeed.

Rust destination:
`ares-core/src/project_slice/gcode_emit/expression.rs::Parser`. The
existing recursive-descent expression parser remains the compatibility
shell for the upstream grammar; this is not a new template pipeline.

## Diagnostic result (Native-first, official and seed0 replay)

Probes appended to the baseline `machine_start_gcode` (CPU17):

| Dead-branch expression | Native result |
| --- | --- |
| `1/0`, `5 % 0` | suppressed; live arm emitted |
| unknown symbol, unknown symbol indexed, `layer_num[3]` | suppressed |
| bare string-vector `filament_type` | suppressed |
| `1 > "a"`, `min(1, "a")`, `-unknown` | suppressed |
| `size(3)` | rejected: `Expecting tag variable reference` |
| `(1+)` | rejected: `Expecting an expression` |
| live `1/0` (selected arm) | rejected: `Division by zero` |

Ares today reports the dead-branch failure itself (`division by zero in
true ? 7 : 1/0`) because `parse_or` eagerly evaluates both arms before
selecting. The fix mirrors upstream: parse the inactive branch for
structure, suppress its evaluation failures, and keep only the selected
arm's value.

## Included / deferred

Included: suppression of evaluation failures (unknown symbols, vector
scalar-context violations, index failures, arithmetic failures including
division by zero, comparison and regex-build failures, function
failures, unary minus on non-numbers) inside inactive `?:` branches of
brace expressions, at the expression evaluator level. Grammar-level
failures (missing `:`/`)`/`]`, malformed sub-expressions, `size()` not
given a variable reference) still reject, matching Native.

Deferred: upstream's `TYPE_EMPTY` value propagation (a suppressed value
is a discarded dummy here because only the selected arm's value is
kept), `{if}/{elsif}/{else}` block suppression in the template layer
(unselected arms are never rendered by construction), the statement
grammar (`local`/`global`/assignments), and `interpolate_table`.

## Regression contract

Freeze `tests/parity/template-ternary/dead-branch.3mf`:
`tests/parity/adaptive-octree-spacing/baseline.3mf` with only the
project-setting `machine_start_gcode` extended by ten neutral
`;TERNARY_*` probe lines. All other ZIP entry payloads are byte-equal to
the baseline. Commit the input fixture only, never Native output.

The E2E test first qualifies two ordered Native streams, asserts the ten
marker lines show the live arms (`7`, `8`, `9` pattern), then compares
the complete public API and CLI output with the canonical comparator.
The same test runs against official and seed0 replay Native,
CPU17/thread1. The RED must fail because Ares rejects the dead-branch
division; GREEN must preserve whole-stream equality, not merely survive
the probes.
