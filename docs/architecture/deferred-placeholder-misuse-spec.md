# Deferred placeholder values in expressions

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `PlaceholderParser::set_deferred` (`PlaceholderParser.hpp:65`) stores the reserved tag as the
  value of a key and records the tag. `process` hands the recorded tags to the parser context
  (`PlaceholderParser.cpp:2695`).
- `expr::Deferred` (`PlaceholderParser.cpp:314-317`): a string value is either one deferred
  value (`Value`) or text that contains at least one (`Text`). Copies and moves keep the state
  (`:246-292`); assigning a new string clears it (`:336-356`).
- `MyContext::mark_deferred` (`:1024-1034`) sets the state when a variable is read
  (`variable_value`, `:1492-1502`): `Value` when the string is a recorded tag, `Text` when it
  contains one, `None` otherwise. The match is by value (`:944-945`), so a local or global
  variable that copied a deferred value is deferred too. String literals are never marked.
- `expr::throw_exception` (`:869-875`) reports `deferred_message()` (`:864-867`) in place of its
  own message whenever the value is deferred: "This value is known only after the G-code is
  generated. It can be printed, but not used in a calculation or a comparison."
  `throw_if_deferred` (`:884-888`) throws it for any deferred value.
- The operators:
  - `operator+=` (`:491-512`): a deferred `Value` added to a number, or to another deferred
    `Value`, throws at the left operand; a number plus a deferred `Value` throws at the right
    operand. Otherwise string concatenation applies and the result is `Text` when either
    operand is deferred.
  - `-`, `*`, `/`, `%` (`:523-596`): `throw_if_not_numeric` on the left, then the right operand,
    so a deferred operand reports the deferred message at that operand.
  - Unary minus, `int`, `round`, `floor`, `ceil` and `not` (`:391-488`): the type error of a
    deferred operand reports the deferred message.
  - Comparisons (`compare_op`, `:623-662`), regular expression matches (`regex_op`, `:754-759`)
    and `and` / `or` (`logical_op`, `:845-858`) throw for a deferred operand, left first.
  - `min`, `max` (`function_2params`, `:684`), `random` (`:714`) and `digits`, `zdigits`
    (`:730`) check their parameters in order; a deferred parameter reports the deferred
    message.
  - The ternary condition (`evaluate_boolean`, `:604-612`, used at `:2378-2383`) reports the
    deferred message for a deferred condition.
  - `interpolate_table` (`:1959`): a deferred lookup value reports the deferred message.
- The caret of the error report points at the start of the failing operand (`it_range`,
  `:2472-2476`); for a variable that is the variable name.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/expression/deferred.rs` (new): the deferred
  state, the marking of variable values by the tags of `processor::DEFERRED_PLACEHOLDERS`, and
  the operator checks above.
- `crates/ares-core/src/project_slice/gcode_emit/expression/parser.rs`: every parse step carries
  the deferred state and the start offset of its value, and runs the checks. The variable
  lookup and the function calls move to `expression/parser/variable.rs` and
  `expression/parser/call.rs` so the parser stays under the file size limit.
- `crates/ares-core/src/project_slice/gcode_emit/expression.rs`: the deferred failure reports
  its operand offset through the existing line and caret report.

## Included

- The deferred state, its marking on variable reads, and its propagation through parentheses,
  unary plus, the ternary branches and string concatenation.
- The deferred checks of every operator and function listed above that Ares implements.
- The line and caret report for brace placeholders, matching Native byte for byte.

## Deferred

- `{if}` and `{elsif}` conditions and assignments (`bool_expr_eval`, `:2343-2345`, `:2408`;
  `scalar_variable_assign_scalar_expression`, `:1566`). Ares evaluates them without the
  line and caret report, so a deferred value used there fails with the deferred message but not
  in Native's format. A bare deferred value as an `{if}` condition is still treated as true,
  because Ares does not yet require a boolean there for any value.
- `regex_replace` (`:785`, `:1882-1886`), `one_of` (`:804-842`) and `filament_change`
  (`:1871-1878`): Ares does not implement these functions yet.
- The table points of `interpolate_table` (`:1963-1968`): Ares parses them as generic tuples,
  which may hold deferred text in other contexts.
- Templates rendered before `GCode.cpp:4086` see no recorded tags upstream. Ares inserts the
  tags in every template, as the previous slice does, so the marking applies everywhere.

## Premises

1. Native rejects each of these `machine_start_gcode` lines with the deferred message, the
   caret at the deferred operand, and CLI exit 156:
   - `; TOTAL={print_time_total_sec + 1}`;
   - `; SECONDS={2 * print_time_sec}` (caret at `print_time_sec`);
   - `; LONG={print_time_minute > 5}`;
   - `{local hours = print_time_hour}; HOURS={hours - 1}` (caret at `hours`).
2. At the base, Ares accepts the first and third lines: `+` concatenates the tag text with the
   number, and `>` compares the tag text. It rejects the second and fourth with "numeric
   expression expected in rhs/lhs in ...", without the line and caret report.
3. `; SECONDS={"s=" + print_time_sec + 1}` is accepted by both, because `"s=" + print_time_sec`
   is deferred `Text`, which concatenates. Both print the seconds followed by `1`.

## Observable behavior

A deferred placeholder used in a calculation, a comparison or a logical operation fails with
Native's message and caret. Printing it, alone or concatenated into text, is unchanged.

## Failure modes

- Marking by key name instead of by value: a variable that copied a deferred value is not
  deferred.
- Marking string literals: a literal that spells a tag would fail.
- Treating `Text` like `Value` in `+`: text that holds a deferred value could no longer have
  numbers appended.
- Checking only the left operand: `2 * print_time_sec` keeps the old error.
- Reporting at the expression start instead of the operand: the caret differs.
- Checking inside an inactive ternary branch: upstream skips evaluation there.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::processor_time::placeholder::placeholder_deferred_misuse::adding_a_number_to_a_deferred_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_deferred_misuse::multiplying_by_a_deferred_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_deferred_misuse::comparing_a_deferred_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_deferred_misuse::a_variable_holding_a_deferred_value_is_deferred_too` | F | P |
| `runner::processor_time::placeholder::placeholder_deferred_misuse::text_holding_a_deferred_value_is_still_concatenated` | P | P |

Each test uses the stock Creality Ender-3 V3 SE profile and the cube fixture, with one line
prepended to the machine start template. The rejection tests compare the CLI exit code, stdout,
stderr and result file with Native byte for byte; the text test compares the whole output.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
