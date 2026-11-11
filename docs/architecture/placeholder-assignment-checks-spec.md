# Placeholder assignment and declaration checks

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `assignment_statement` (`PlaceholderParser.cpp:2435-2448`) first resolves its target with
  `variable_reference` and `resolve_variable` (`:1123-1145`): a name that is neither a system
  symbol nor an output variable throws "Not a variable name" at the name, before the right-hand
  side is parsed. A system symbol (`resolve_symbol`) resolves read-only; only output variables
  (`resolve_output_symbol`) are writable (`:1131-1135`): the `local` and `global` variables and
  the system output variables `position`, `e_position`, `e_retracted` and `e_restart_extra`
  (`GCode.cpp:2189-2209`).
- After the right-hand side is evaluated, `scalar_variable_assign_scalar_expression`
  (`:1566-1575`) calls `check_writable` (`:1377-1380`), which throws "Cannot modify a read-only
  variable" at the target, also for an indexed vector element.
- `scalar_variable_assign_scalar` (`:1396-1426`) stores the value in the type of the variable:
  - int: `check_numeric` (`:1382-1385`), then `as_i` (`:327`), which truncates a double;
  - float: `check_numeric`, then `as_d`;
  - string: `to_string`;
  - bool: anything but a boolean throws "Right side is not a boolean expression" at the
    right-hand side through `ctx->throw_exception`, so a deferred value gets this message too.

  `check_numeric` throws "Right side is not a numeric expression" through
  `expr::throw_exception` (`:869-875`), so a deferred right-hand side reports the deferred
  message.
- `new_variable_statement` (`:2450-2462`) calls `new_old_variable` (`:1529-1564`) on the name
  before the right-hand side:
  - a system symbol throws "Symbol is already defined in read-only system dictionary";
  - a system output variable throws "Symbol is already defined as system output variable";
  - `global` over a local variable throws "Variable name already defined in local scope";
  - `local` over a global variable throws "Variable name already defined in global scope".

  The caret is at the name. A name already declared in the same scope is assigned in its type by
  `scalar_variable_new_from_scalar_expression` (`:1577-1600`) through
  `scalar_variable_assign_scalar`. A new name takes the type of its value.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/expression.rs` and
  `expression/type_check.rs`: evaluation of a right-hand side converted to the type of the
  assigned variable, with the failures at the operand.
- `crates/ares-core/src/project_slice/gcode_emit/template/macro_block.rs` and
  `template/macro_block/evaluate.rs`, where the statement evaluation moves: the journal of
  output variables records the scope of each declared name, the system output variables are
  writable, and declarations and assignments apply the checks above.
- `crates/ares-core/src/project_slice/gcode_emit/template.rs`: an assignment placeholder
  (`{x = 1}`) is evaluated as a statement block, so both forms share the checks.

The previous Ares behavior assigned any name, including system symbols and unknown names, and
stored the right-hand side with its own type. It is replaced, not kept as a fallback.

## Included

- "Not a variable name" for an unknown assignment target, before the right-hand side.
- "Cannot modify a read-only variable" for an assignment to a system symbol or its element,
  after the right-hand side.
- The four declaration errors of `new_old_variable`.
- The typed assignment of `local` and `global` scalar variables, with its two errors.

## Deferred

- Vector variables: initializer lists, `repeat`, vector copies and the type checks of an
  assigned vector element (`:1428-1525`, `:1601-1760`).
- The index expression of an assigned vector element (`variable_reference`, `:2436`), which keeps
  its current error text.
- `global` variables outliving the template that declared them (`context_data->global_config`).
- `as_i` of a double outside the `int` range, which is undefined behavior upstream and is not
  ported; no fixture uses it.

## Premises

1. Native rejects each of these `machine_start_gcode` lines with CLI exit 156, the message and
   the caret below:
   - `{layer_height = 0.3}; X`, `{current_extruder = 1}; X`, `{nozzle_diameter[0] = 0.6}; X` and
     `{layer_height = print_time_sec}; X`, "Cannot modify a read-only variable", at the target;
   - `{layer_height = "a" - 1}; X`, "Cannot subtract non-numeric types.", at `"a"`;
   - `{zzz = 1}; X` and `{zzz = "a" - 1}; X`, "Not a variable name", at `zzz`;
   - `{local layer_height = 1}; X` and `{local current_extruder = 1}; X`, "Symbol is already
     defined in read-only system dictionary", at the name;
   - `{local position = 1}; X` and `{global e_retracted = 1}; X`, "Symbol is already defined as
     system output variable", at the name;
   - `{global g = 1}{local g = 2}; X`, "Variable name already defined in global scope", and
     `{local y = 1}{global y = 2}; X`, "Variable name already defined in local scope", at the
     second name;
   - `{local x = 1}{x = "a"}; X`, `{local x = 1}{x = true}; X`, `{local x = 1}{local x = "a"}; X`
     and `{local x = 1; x = "a"}; X`, "Right side is not a numeric expression", at the
     right-hand side;
   - `{local b = true}{b = 1}; X` and `{local b = true}{b = print_time_sec}; X`, "Right side
     is not a boolean expression", at the right-hand side;
   - `{local x = 1}{x = print_time_sec}; X`, the deferred message, at `print_time_sec`.
2. Native prints `; X=1` for `{local x = 1}{x = 1.7}; X={x}`, `; X=-1` for
   `{local x = 1}{x = -1.7}; X={x}`, `; X=2` for `{local x = 1}{local x = 2.9}; X={x}` and for
   `{local x = 1; x = 2.5}; X={x}`, `; G=2` for `{global g = 1}{global g = 2.5}; G={g}`,
   `; X=0.5` for `{local x = 1.5}{x = 2}; X={x / 4}` and `; S=51` for
   `{local s = "a"}{s = 5}; S={s + 1}`.
3. Native and Ares agree on `{layer_height = "a" - 1}; X`, which reports the right-hand side
   before the read-only target, and on `{position[2] = 7}; Z={position[2]}`,
   `{global g = 1}; G={g}`, `{local x = 1}{local x = 2}; X={x}`, `{local s = "a"}{s = true}; S={s}`
   and `{local s = "a"}{s = 0.1 + 0.2}; S={s}`.

## Observable behavior

An assignment or declaration that upstream rejects fails with Native's message and caret, and
an assigned `local` or `global` variable keeps the type it was declared with.

## Failure modes

- Writing a system symbol: `{layer_height = 0.3}` changes the value later templates read.
- Creating a variable by assignment: `{zzz = 1}` succeeds.
- Checking the target before the right-hand side for a read-only symbol, or after it for an
  unknown name: the other message is reported.
- Keeping the type of the right-hand side: `{x = 1.7}` on an int prints `1.7`.
- Rounding instead of truncating: `-1.7` becomes `-2`.
- Reporting the deferred message for a boolean variable.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::processor_time::placeholder::placeholder_assignment_checks::assigning_a_config_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::assigning_a_runtime_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::assigning_a_config_vector_element_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_deferred_value_assigned_to_a_config_value_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_type_error_assigned_to_a_config_value_fails_like_native` | P | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::assigning_an_unknown_name_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::an_unknown_name_is_reported_before_its_value` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::declaring_a_config_name_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::declaring_a_runtime_name_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::declaring_a_system_output_name_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::declaring_a_global_system_output_name_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_local_over_a_global_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_global_over_a_local_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_assigned_to_an_int_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_bool_assigned_to_an_int_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_redeclaring_an_int_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_assigned_to_an_int_in_a_block_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_number_assigned_to_a_bool_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_deferred_value_assigned_to_a_bool_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_deferred_value_assigned_to_an_int_fails_like_native` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::an_int_variable_truncates_an_assigned_double` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::an_int_variable_truncates_a_negative_double_toward_zero` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_redeclared_int_keeps_its_type` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::an_int_assigned_in_a_block_keeps_its_type` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_redeclared_global_keeps_its_type` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_float_variable_keeps_an_assigned_int_a_double` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_variable_keeps_an_assigned_number_a_string` | F | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::assigning_a_system_output_element_is_allowed` | P | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_global_variable_prints_like_native` | P | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_redeclared_local_takes_the_new_value` | P | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_variable_takes_a_bool_as_text` | P | P |
| `runner::processor_time::placeholder::placeholder_assignment_checks::a_string_variable_takes_a_double_as_text` | P | P |

Each test prepends one line to the stock Creality Ender-3 V3 SE machine start template and
slices the cube fixture. The rejection tests compare the CLI exit code, stdout, stderr and
result file with Native byte for byte; the other tests compare the whole output.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
