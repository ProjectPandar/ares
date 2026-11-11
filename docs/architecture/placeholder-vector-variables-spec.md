# Placeholder vector variables

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `assignment_statement` and `new_variable_statement` (`PlaceholderParser.cpp:2435-2463`) try
  four right-hand sides in order:
  1. `initializer_list` (`:2464-2470`), `(` expressions separated by `,` `)`, which also takes
     `()` and a single parenthesized expression;
  2. a naked vector reference, `variable_reference` (`:2538-2544`) of an `identifier`
     (`:2373-2375`, not one of the `keywords` of `:2553-2579`). For an assignment the target
     must be an unindexed vector (`is_vector_variable_reference`, `:1747-1749`). For a
     declaration the name must be new or a vector (`could_be_vector_variable_reference`,
     `:1751-1755`);
  3. `conditional_expression`;
  4. `repeat(additive_expression, conditional_expression)`.
- `check_numeric` (`:1382-1385`) throws "Right side is not a numeric expression" through
  `expr::throw_exception`, so a deferred operand reports the deferred message.
- `evaluate_count` (`:1387-1394`), also through `expr::throw_exception`:
  - a count that is not an int throws "Expected number of elements to fill a vector with.";
  - a negative count throws "Negative number of elements specified.".
- `vector_variable_element_assign_scalar` (`:1428-1463`), after `check_writable`:
  - an unindexed target throws "Referencing an output vector variable when scalar is expected";
  - an empty vector throws "Indexing an empty vector variable";
  - an index past the end throws "Index out of range".

  These three errors point at the target. The element then takes the vector type:
  - floats and ints go through `check_numeric`, then `as_d` / `as_i`;
  - strings take `to_string`;
  - bools require a boolean, else "Right side is not a boolean expression" at the right-hand
    side through `ctx->throw_exception`.
- `vector_variable_assign_expr_with_count` (`:1465-1490`) evaluates the count, then fills the
  vector with the value in its type. A non-boolean into a bool vector throws through
  `expr::throw_exception`.
- `vector_variable_new_from_array` (`:1602-1626`) and `vector_variable_assign_array`
  (`:1628-1640`):
  - a scalar target throws "Cannot assign a vector value to a scalar variable." at the value;
  - a new name becomes a vector of the value's type: bool, int, float or string.
- `vector_variable_assign_initializer_list` (`:1651-1694`), after `check_writable`:
  - a scalar target takes a one-element list as a scalar assignment;
  - any other list on a scalar target throws "Cannot assign a vector value to a scalar
    variable." at the target;
  - a vector target is replaced whole, also when it is indexed, with every element in its
    type. The checks report through `expr::throw_exception`.
- `vector_variable_new_from_initializer_list` (`:1696-1745`):
  - an empty list for a new name throws "Cannot create vector variable from an empty
    initializer list, because its type cannot be deduced." at the name;
  - otherwise any string makes a string vector;
  - booleans mixed with numbers throw "Right side is not valid: Mixing numeric and boolean
    types." from the first element;
  - booleans alone make a bool vector, any double makes a float vector, and anything else an
    int vector.
- `copy_vector_variable_to_vector_variable` (`:1757-1786`), after `check_writable`:
  - an indexed or scalar source throws "Cannot assign scalar to a vector" at the target;
  - a nil source throws "Some elements of the right hand side vector variable of optional
    values are undefined (nil)" at the source. `ConfigOptionVector::is_nil` (`Config.hpp:901,
    1085, 1978`) holds when every element is nil, which an empty float, int, percent or bool
    vector also is. A string vector is never nil;
  - a source of another option type throws "Left hand side is a float|an int|a string|a bool
    vector, while the right hand side is not." at the target.
- `vector_variable_new_from_copy` (`:1788-1818`):
  - a new name with an indexed or scalar source falls through to `conditional_expression`;
  - a nil source throws as above;
  - floats, ints, strings and bools are cloned, and percents become floats;
  - any other vector type throws "Duplicating this type of vector variable is not supported" at
    the source.
- `scalar_variable_new_from_scalar_expression` (`:1577-1600`): redeclaring a vector with a
  scalar throws "Cannot assign a scalar value to a vector variable." at the right-hand side.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/template/macro_block/rhs.rs`: the four
  right-hand side alternatives as slices of the block body.
- `template/macro_block/vector_kind.rs`: the vector type of a variable. It is the declared
  type for a `local` or `global` vector. For a system vector it is the registry option type,
  or the element type of a runtime vector such as `position`.
- `template/macro_block/evaluate.rs` and `evaluate/vector.rs`: the statement dispatch and the
  vector rules above.
  - The render journal records the vector type of each declared variable, so an empty vector
    keeps it.
  - A replaced vector drops the spread `name_index` entries of the old one.
- `crates/ares-core/src/project_slice/gcode_emit/expression.rs` and `expression/type_check.rs`:
  - the evaluated operand of a right-hand side element;
  - the element conversion with the `expr::throw_exception` boolean check;
  - `evaluate_count`.

The previous Ares behavior had several differences, all replaced, not kept as a fallback:
- every list was untyped;
- an element assignment past the end grew the vector with zeros;
- a parenthesized expression assigned or declared a scalar;
- a naked vector on a right-hand side resolved as an expression.

## Included

- Initializer lists for assignments and declarations, with type deduction and per-type
  conversion and errors.
- `repeat(count, value)` for assignments and declarations, with the count errors.
- Copies of a naked vector into a vector, and new vectors cloned from one, with the scalar,
  nil, type and duplicate errors.
- The bounds and type checks of an assigned vector element, also for system output vectors.
- The scalar and vector target errors of the four alternatives.

## Deferred

- Boost.Spirit expectation errors, such as "Expecting tag alternative" for text after an
  initializer list (`{local x = (1 + 2) * 3}`) or after a copied vector (`{local w = v + 1}`).
  Ares keeps its own message for these.
- `empty(vector)` (`is_vector_empty`, `:1830-1840`), which Ares evaluates as a numeric function
  and so rejects for a vector.
- The template config of nil filament overrides such as `filament_retraction_length`. Ares
  builds the placeholder config from the G-code config block, which omits them, so they are
  unknown names. The nil copy errors are therefore tested with empty vectors.
- The index expression of an assigned element; it keeps its current error text, and an
  indexed copy source is not evaluated.
- `global` variables outliving the template that declared them.

## Premises

1. Native rejects each of these `machine_start_gcode` lines with CLI exit 156, the message and
   the caret below:
   - `{local v = ()}; X`, the empty list message, at `v`;
   - `{local v = (1, true)}; X`, the mixing message, at `1`;
   - `{local v = (1, 2)}{v = ("a")}; X` and `{local v = (1, 2)}{v = (1, true)}; X`, "Right side
     is not a numeric expression", at the offending element;
   - `{local v = (true)}{v = (1)}; X`, "Right side is not a boolean expression", at `1`;
   - `{local v = (true)}{v = (print_time_sec)}; X`, the deferred message;
   - `{local s = 1}{s = (1, 2)}; X` and `{local s = 1}{local s = (1, 2)}; X`, "Cannot assign a
     vector value to a scalar variable.", at `s`;
   - `{local s = 1}{s = repeat(2, 1)}; X` and `{local s = 1}{local s = repeat(2, 1)}; X`, the
     same message, at the value `1`;
   - `{local v = repeat(1.5, 2)}; X` and `{local v = (1, 2)}{v = repeat("a", 2)}; X`, "Expected
     number of elements to fill a vector with.", at the count;
   - `{local v = repeat(print_time_sec, 1)}; X`, the deferred message, at the count;
   - `{local v = repeat(-1, 2)}; X` and `{local v = (1, 2)}{local v = repeat(-2, 1)}; X`,
     "Negative number of elements specified.", at the count;
   - `{local v = (true)}{v = repeat(2, 1)}; X`, "Right side is not a boolean expression", at
     the value;
   - `{e_retracted = repeat(1, 2)}; X`, "\"e_retracted\" output variable must not be resized by
     the script.";
   - `{local v = (1, 2)}{v = 3}; X` and `{local v = (1, 2)}{v = min(1, 2)}; X`, "Referencing an
     output vector variable when scalar is expected", at `v`;
   - `{local v = (1, 2)}{local v = 3}; X`, "Cannot assign a scalar value to a vector
     variable.", at `3`;
   - `{local v = (1, 2)}{v = nozzle_diameter}; X`, `{local v = (1, 2)}{local v =
     nozzle_diameter}; X` and `{local v = (1, 2)}{v = position}; X`, the int vector mismatch, at
     the target;
   - `{local v = (1.5)}{v = filament_shrink}; X`, the float vector mismatch;
   - `{local v = (1, 2)}{v = layer_height}; X`, `{local v = (1, 2)}{v = nozzle_diameter[0]}; X`
     and `{local v = (1, 2)}{v = nozzle_diameter[0] + 1}; X`, "Cannot assign scalar to a
     vector", at `v`;
   - `{local v = (1, 2)}{v = zzz}; X`, "Not a variable name", at `zzz`;
   - the nil message at the source for these lines:
     - `{local v = filament_retraction_length}; X`;
     - `{local e = repeat(0, 1.5)}{local f = e}; X`;
     - `{local e = repeat(0, 1)}{local v = (1, 2)}{v = e}; X`;
     - `{local e = repeat(0, true)}{local v = (false)}{local v = e}; X`;
   - `{local v = printable_area}; X` and `{local v = overhang_fan_threshold}; X`, "Duplicating
     this type of vector variable is not supported", at the source;
   - `{local v = (1, 2)}{v[5] = 3}; X` and `{position[9] = 5}; X`, "Index out of range", at the
     target;
   - `{local v = repeat(0, 1)}{v[0] = 3}; X`, "Indexing an empty vector variable";
   - `{local v = (1, 2)}{v[0] = true}; X` and `{e_retracted[0] = "a"}; X`, "Right side is not a
     numeric expression";
   - `{local v = (true)}{v[0] = 1}; X` and `{local v = (true)}{v[0] = print_time_sec}; X`,
     "Right side is not a boolean expression", without the deferred message;
   - `{local x = (1 + 2) * 3}; X` and `{local v = (1, 2)}{local w = v + 1}; X`, "Expecting tag
     alternative" (deferred above).
2. Native prints these values:
   - `1 2 3 3` for `(1, 2, 3)` and its size, `1 2.5` for `(1, 2.5)`, `1|a|true|2.5` for
     `(1, "a", true, 2.5)` and `true false` for `(true, false)`;
   - `3 4` after an int vector takes `(3.7, 4)`, `7` after `v[1] = 7.9`, and `1` after a float
     vector takes `repeat(2, 1.5)` into an int vector;
   - `5` after `{local s = 1}{s = (5.5)}`, `7` after `{local s = 1}{local s = (7.5)}`, and
     `5 1` for `{local v = (5)}`;
   - `1 7` after `{local v = (1, 2)}{v[1] = (7)}` and `3` after `{v[0] = (1, 2, 3)}`;
   - `1.5 3` for `repeat(3, 1.5)`, `true true`, `x`, and `0 true` for `repeat(0, 1)` with
     `empty`;
   - `0.2 1` for a copy of `nozzle_diameter`, `100` for `filament_shrink`, `ac` for a copied
     string vector changed in the copy, `PLA` and `false` for copies of `filament_type` and
     `filament_soluble`, and `15` for `wipe_tower_x`;
   - `2` for `{local w = v[0] + 1}`, and the scalar `nozzle_diameter` value for a scalar
     redeclared or assigned from it;
   - `1.5` for `{s[0] = 1.5}` and `5` for `{t[0] = 5}` on string vectors;
   - `1 3` for `{position = (1, 2, 3)}`;
   - an empty string vector is copied without the nil error.

## Observable behavior

Assignments and declarations with a vector right-hand side store vectors of upstream's type
and fail with Native's message and caret where upstream rejects them.

## Failure modes

- An untyped list: `(3.7, 4)` assigned to an int vector prints `3.7`.
- Growing a vector on an index past its end instead of failing.
- `(5)` declaring a scalar, or `(5.5)` keeping its double on an int scalar.
- A naked vector on the right-hand side evaluated as an expression: a string vector copy fails
  with "Referencing a vector variable when scalar is expected".
- Comparing element values instead of option types: copying `filament_shrink` into a float
  vector succeeds.
- Losing the type of an empty vector: `repeat(0, 1.5)` then `("a")` succeeds.
- Reporting the deferred message for a bool element, or omitting it for a bool list element.
- Counting an empty string vector as nil, or an empty float vector as not nil.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::processor_time::placeholder::placeholder_vector_variables::initializer_lists_declare_vectors_of_the_deduced_type` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::assigned_lists_take_the_vector_type` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_one_element_list_assigns_a_scalar_and_declares_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::an_assigned_list_replaces_the_whole_vector_even_when_indexed` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_fills_vectors_of_the_value_type` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::an_empty_list_cannot_declare_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_list_mixing_numbers_and_booleans_cannot_declare_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_string_cannot_join_an_int_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_boolean_cannot_follow_a_number_in_an_int_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_number_cannot_join_a_bool_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_deferred_value_cannot_join_a_bool_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_type_error_in_a_list_element_fails_like_native` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_list_of_two_cannot_be_assigned_to_a_scalar` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_list_of_two_cannot_redeclare_a_scalar` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_list_cannot_be_assigned_to_a_config_vector` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_needs_an_integer_count` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_into_a_vector_needs_an_integer_count` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_needs_a_known_count` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_rejects_a_negative_count` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_redeclaring_a_vector_rejects_a_negative_count` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_cannot_be_assigned_to_a_scalar` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_cannot_redeclare_a_scalar` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_cannot_fill_a_bool_vector_with_a_number` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::repeat_cannot_resize_an_output_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_scalar_cannot_be_assigned_to_a_whole_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_function_result_cannot_be_assigned_to_a_whole_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_variables::a_scalar_cannot_redeclare_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::naked_vector_references_copy_the_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::indexed_and_scalar_references_declare_scalars` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::assigned_elements_take_the_vector_type` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_float_config_vector_cannot_be_copied_into_an_int_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_float_config_vector_cannot_redeclare_an_int_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_output_vector_cannot_be_copied_into_an_int_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_percent_vector_cannot_be_copied_into_a_float_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_scalar_config_value_cannot_be_copied_into_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_vector_element_cannot_be_copied_into_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_vector_element_in_an_expression_cannot_be_copied_into_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::copying_an_unknown_name_fails_like_native` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_empty_int_vector_counts_as_nil_and_cannot_be_copied` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_empty_bool_vector_counts_as_nil_and_cannot_redeclare_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_empty_float_vector_counts_as_nil_and_cannot_declare_a_vector` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_empty_string_vector_is_not_nil_and_is_copied` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_point_vector_cannot_be_duplicated` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_enum_vector_cannot_be_duplicated` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_element_index_past_the_end_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_output_element_index_past_the_end_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::an_element_of_an_empty_vector_cannot_be_assigned` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_boolean_cannot_be_assigned_to_an_int_element` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_string_cannot_be_assigned_to_an_output_element` | F | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_number_cannot_be_assigned_to_a_bool_element` | P | P |
| `runner::processor_time::placeholder::placeholder_vector_copies::a_deferred_value_assigned_to_a_bool_element_is_not_a_boolean` | P | P |

Each test prepends one line to the stock Creality Ender-3 V3 SE machine start template and
slices the cube fixture. The rejection tests compare the CLI exit code, stdout, stderr and
result file with Native byte for byte; the other tests compare the whole output.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
