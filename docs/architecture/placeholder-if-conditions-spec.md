# Placeholder `{if}` conditions and statement errors

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `if_else_output` (`PlaceholderParser.cpp:2341-2349`) parses `{if}`, every `{elsif}` and the
  optional `{else}` of one block, in the text form (`if_text_block`, `:2351`) and in the macro
  form (`if ... then`, `if_macros` and `else_macros`, `:2320-2322`).
- Each `if` and `elsif` condition is `bool_expr_eval` (`:2408`): `conditional_expression`
  followed by `expr::evaluate_boolean` (`:604-612`), which throws "Not a boolean expression"
  at the condition unless it is a boolean.
- `MyContext::block_enter` (`:966-970`) suppresses the arm body when its condition is false or
  an earlier arm was taken (`_1 && _a`, `:2345`), but the `elsif` condition itself is still
  evaluated. A whole block inside a suppressed arm is skipped: its conditions are `TYPE_EMPTY`
  and `evaluate_boolean` does nothing.
- Evaluation errors of a condition, of the right-hand side of an assignment
  (`assignment_statement`, `:2435-2448`; `scalar_variable_assign_scalar_expression`, `:1566-1575`) and
  of a `local`/`global` declaration (`new_variable_statement`, `:2450-2462`; `new_old_variable`,
  `:1529`) throw at their operand (`expr::throw_exception`, `:869-875`), and an unknown
  variable throws at its name. `process_error_message` (`:1895`) reports them like any other
  placeholder error: the line number, the message, the line and a caret at the operand.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/expression.rs`: `evaluate_condition` evaluates
  a condition on the brace path and requires a boolean; right-hand sides and expression
  statements use `evaluate_path`, which gives the offset of the failing operand.
- `crates/ares-core/src/project_slice/gcode_emit/template/parser.rs`: each `{if}` and
  `{elsif}` condition keeps its template offset.
- `crates/ares-core/src/project_slice/gcode_emit/template.rs`: a condition must be a boolean;
  every condition of a block is evaluated in order and the first true arm is rendered when its
  condition is reached; condition and assignment failures go through the line and caret report.
- `crates/ares-core/src/project_slice/gcode_emit/template/macro_block.rs`: the same for the
  macro form, with conditions, right-hand sides and expression statements kept as slices of
  the block so their offsets are known.

The previous Ares behavior treated any condition by its truth value, stopped at the first true
arm, and reported these failures as "MESSAGE in EXPRESSION" without the line and caret. It is
replaced, not kept as a fallback.

## Included

- "Not a boolean expression" for an `if` or `elsif` condition that is not a boolean, in the
  text and macro forms.
- Evaluation of the `elsif` conditions that follow a taken arm.
- The line and caret report for failures in conditions, assignments, declarations and macro
  expression statements.

## Deferred

- The index expression of an assigned vector element (`variable_reference` in
  `assignment_statement`, `:2436`), which keeps its current error text.
- The checks of the assignment itself (`scalar_variable_assign_scalar_expression`,
  `:1566-1575`): read-only variables and the type of the assigned value.

## Premises

1. Native rejects each of these `machine_start_gcode` lines with CLI exit 156, the message and
   a caret at the condition or operand:
   - `{if 1}; ONE{endif}`, "Not a boolean expression", caret at `1`;
   - `{if true}; A{elsif 1}; B{endif}`, "Not a boolean expression", caret at the `elsif` `1`;
   - the same block over five lines, reported at line 3;
   - `{if 1 then local x = 1; endif}; X` and
     `{if true then local x = 1; elsif 2 then local y = 2; endif}; X`, "Not a boolean
     expression";
   - `{if "2" - 1 > 0}; X{endif}`, "Cannot subtract non-numeric types.", caret at `"2"`;
   - `{if print_time_sec > 0}; X{endif}`, the deferred message, caret at `print_time_sec`;
   - `{if print_time_sec}; X{endif}`, the deferred message: `evaluate_boolean` throws through
     `expr::throw_exception`, which reports a deferred value with the deferred message;
   - `{if no_such_variable}; X{endif}`, "Not a variable name";
   - `{local x = "2" - 1}; X={x}` and `{local x = 1}{x = "2" - 1}; X={x}`, "Cannot subtract
     non-numeric types.", caret at `"2"`.
2. Native accepts `{if false}{if 1}; A{endif}{endif}; B`: the inner condition is skipped.
3. At the base, Ares accepts the first five lines and the bare deferred condition, and rejects
   the others without the line and caret report.

## Observable behavior

A template whose condition is not a boolean, or whose condition or assignment fails, is
rejected with Native's message and caret. Templates with boolean conditions are unchanged.

## Failure modes

- Treating a number as true: `{if 1}` renders its arm.
- Stopping at the taken arm: a bad `elsif` after it is never checked.
- Evaluating every condition before rendering the taken arm: an error in that arm's body would
  be reported after a later condition's error, unlike upstream.
- Evaluating blocks inside a skipped arm: the accepted nested block would fail.
- Reporting the caret at the start of the directive instead of the operand.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::processor_time::placeholder::placeholder_if_conditions::a_numeric_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::an_elsif_after_a_taken_arm_is_still_checked` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_multiline_elsif_reports_its_own_line` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_numeric_macro_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_macro_elsif_after_a_taken_arm_is_still_checked` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_type_error_in_an_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_deferred_value_in_an_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_bare_deferred_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::an_unknown_variable_in_an_if_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_type_error_in_a_declaration_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_type_error_in_an_assignment_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_if_conditions::a_block_inside_a_skipped_arm_is_not_evaluated` | P | P |

Each test prepends one line to the stock Creality Ender-3 V3 SE machine start template and
slices the cube fixture. The rejection tests compare the CLI exit code, stdout, stderr and
result file with Native byte for byte; the skipped block test compares the whole output.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
