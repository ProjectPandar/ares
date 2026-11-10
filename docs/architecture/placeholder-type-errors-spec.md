# Placeholder operator and function type errors

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

`expr` (`PlaceholderParser.cpp:243-906`) keeps the type of each value: `TYPE_BOOL`,
`TYPE_INT`, `TYPE_DOUBLE` or `TYPE_STRING` (`:305-311`). Only `TYPE_INT` and `TYPE_DOUBLE` are
numeric (`numeric_type`, `:313`). Operators and functions do not convert between types; they
throw at the offending operand (`expr::throw_exception`, `:869-875`), whose `it_range` starts
the caret of the error report:

- `+` (`:491-521`): when neither operand is a string, both must be numeric, left first:
  "Cannot add non-numeric types."
- `-` (`:523-539`): "Cannot subtract non-numeric types."; `*` (`:541-557`): "Cannot multiply
  with non-numeric type."; left first.
- `/` and `%` (`:559-595`): "Cannot divide a non-numeric type." for the left operand, "Cannot
  divide with a non-numeric type." for the right one, then "Division by zero" at the right
  operand when it is an integer 0 or a double 0.
- Unary minus (`:391-406`): "Cannot apply unary minus operator." for a boolean or a string.
- `int`, `round`, `floor`, `ceil` (`:409-472`): "Cannot convert to integer.", "Cannot round a
  non-numeric value.", "Cannot floor a non-numeric value.", "Cannot ceil a non-numeric value."
- `!` and `not` (`:475-488`): "Cannot apply a not operator." for a non-boolean.
- `and`, `or` (`logical_op`, `:845-858`): "Cannot apply logical operation to non-boolean
  operands." unless both are booleans; the caret is at the left operand.
- The ternary condition (`evaluate_boolean`, `:604-612`, used at `:2378-2383`): "Not a boolean
  expression".
- Comparisons (`compare_op`, `:623-662`): "Cannot compare the types." for a boolean ordered by
  `<`, `>`, `<=` or `>=`, or a boolean compared with a number; the caret is at the left operand.
- `min`, `max` (`function_2params`, `:684`) and `random` (`:714`): "Not a numeric type."
  (`:673-677`), first parameter first.
- `digits`, `zdigits` (`:730-740`): "Not a numeric type." for the value, "digits: second
  parameter must be integer", "digits: third parameter must be integer".
- `interpolate_table` (`:1959`): "Interpolation value must be a number." for the lookup value.

The checks depend on the type a variable reads as. `scalar_variable_to_expr` (`:1170-1232`) and
`vector_element_to_expr` (`:1355-1361`) take it from the option type: `coInt` and `coInts` are
`TYPE_INT`, `coFloat`, `coPercent`, `coFloats` and `coPercents` are `TYPE_DOUBLE` (a percent is
its number), `coBool` and `coBools` are `TYPE_BOOL`, and `coString`, `coEnum` and `coStrings`
are `TYPE_STRING`, whatever their text. Placeholders set at run time keep the type they are set
with: `PlaceholderParser::set` (`PlaceholderParser.hpp:54-62`) stores an `int` as
`ConfigOptionInt`, for example `num_extruders` (`GCode.cpp:3804`), `total_layer_count`
(`:3771`), the timestamp fields (`PlaceholderParser.cpp:137-142`), `layer_num` and `next_extruder`
(`GCode.cpp:1194`, `:1273`), and the physical extruder ids (`:6541-6542`).

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/expression/type_check.rs` (new): the type
  checks above, as the failing operand start and message. A deferred operand reports the
  deferred message instead (`:872-873`), so the deferred checks of the previous slice that go
  through `throw_exception` move here.
- `crates/ares-core/src/project_slice/gcode_emit/expression/parser.rs`, `parser/call.rs`: run the
  checks and report through the existing line and caret report. `comparison.rs` returns its
  message for the caret report.
- `crates/ares-core/src/project_slice/gcode_emit/expression/deferred.rs`: keeps the deferred
  state and the checks that are not type errors (`throw_if_deferred` and `operator+=`).
- `crates/ares-core/src/project_slice/gcode_emit/value.rs`: `Config::from_block` types each
  option value by its registry kind as above, instead of reading any number-like text as a
  number. The integer placeholders set by the start, layer change and end templates
  (`placeholders.rs`, `placeholders/timestamp.rs`, `machine/start_config.rs`, `layer_gcode.rs`,
  `layers/`, `finish.rs`) are integers.

The previous Ares behavior was a temporary scaffold that converted operands: numeric strings and
booleans were numbers, and numbers and strings were truthy. It is replaced, not kept as a
fallback.

## Included

- The type errors listed above, with their messages and carets, for brace placeholders.
- The value types of options and of the integer placeholders listed above.

## Deferred

- `{if}` and `{elsif}` conditions (`bool_expr_eval`, `:2343-2345`, `:2408`) and assignments
  (`:1566`): Ares evaluates them without the line and caret report. An `{if}` condition
  that is not a boolean is still treated by its truth value.
- The table points of `interpolate_table` (`:1963-1968`): Ares parses them as generic tuples.
- `coEnums` elements, which upstream reads as the enum value (`:1361`), stay text.
- `coFloatOrPercent` and `coFloatsOrPercents` values and their `ratio_over` resolution
  (`:1188-1225`, `:1243-1310`).
- The integer placeholders of the timelapse, wipe tower and tool change templates
  (`timelapse.rs`, `wipe_tower/`, `tool_transition/`), and the `timelapse_type` number
  that `finish.rs` sets for the end template.
- The string check of a regular expression match (`regex_op`, `:760-765`), vector indices
  (`:1107`, `:1851`), and `regex_replace`, `one_of`, `filament_change`.

## Premises

1. Native rejects each of these `machine_start_gcode` lines with CLI exit 156 and the listed
   message: `{"2" - 1}` (subtract), `{true + 1}` (add), `{1 / 0}` (division by zero, caret at
   `0`), `{-true}` (unary minus), `{!1}` (not), `{1 and true}` (logical operation),
   `{1 ? 2 : 3}` (not a boolean), `{true < false}` (compare, caret at `true`), `{int("3")}`
   (convert to integer), `{min(1, "2")}` (not a numeric type), `{digits(1, 2.5)}` (second
   parameter).
2. At the base, Ares accepts nine of them by converting the operands. It rejects `{1 / 0}` and
   `{true < false}` with its own text and without the line and caret report.
3. Native accepts `{wall_loops / 4}` (integer division), `{digits(1, wall_loops, 0)}`,
   `{sparse_infill_density + 1}` (a number), `{printer_variant + 1}` and `{timelapse_type + 1}`
   (text concatenation), and `{num_extruders / 2}` (integer division). At the base, Ares prints
   other values for all but the `digits` line, because it reads every number-like text as a
   double; the type checks above alone would reject the `digits` line.
4. Stock templates use only boolean conditions and numeric arithmetic: the template, G-code
   emission and statistics tests keep passing with the conversions removed.

## Observable behavior

A template that mixes types fails with Native's message and caret instead of printing a
converted value. Templates that Native accepts are unchanged.

## Failure modes

- Keeping the string-to-number conversion: `"2" - 1` prints `1`.
- Treating booleans as numbers or numbers as booleans: `true + 1` prints `2`, `1 and true`
  prints `true`.
- Checking the right operand first: the caret and the `/` message differ.
- Reporting the type error of a deferred operand with the type message instead of the deferred
  message.
- Checking inside an inactive ternary branch: upstream skips evaluation there.
- Reading an integer option or placeholder as a double: `digits` rejects it as a width and `/`
  does not truncate.
- Reading a percent as text or numeric text as a number: `+` concatenates or adds the wrong
  way.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::processor_time::placeholder::placeholder_type_errors::subtracting_from_a_numeric_string_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::adding_a_number_to_a_boolean_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::dividing_by_zero_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::negating_a_boolean_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::inverting_a_number_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::a_logical_operation_on_a_number_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::a_numeric_ternary_condition_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::ordering_booleans_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::converting_a_string_to_an_integer_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::the_minimum_of_a_string_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_type_errors::a_fractional_digits_width_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_value_types::an_integer_option_divides_as_an_integer` | F | P |
| `runner::processor_time::placeholder::placeholder_value_types::an_integer_option_is_an_integer_digits_width` | P | P |
| `runner::processor_time::placeholder::placeholder_value_types::a_percent_option_is_a_number` | F | P |
| `runner::processor_time::placeholder::placeholder_value_types::a_numeric_string_option_stays_text` | F | P |
| `runner::processor_time::placeholder::placeholder_value_types::a_numeric_enum_option_stays_text` | F | P |
| `runner::processor_time::placeholder::placeholder_value_types::an_integer_runtime_placeholder_divides_as_an_integer` | F | P |

Each test prepends one line to the stock Creality Ender-3 V3 SE machine start template, slices
the cube fixture, and compares the CLI exit code, stdout, stderr and result file with Native
byte for byte; the value type tests compare the whole output. The deferred placeholder tests
share the same helper and must keep passing.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
