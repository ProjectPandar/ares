//! `PlaceholderParser.cpp:2435-2470` vector right-hand sides: an initializer list `(...)`
//! (`vector_variable_assign_initializer_list`, `vector_variable_new_from_initializer_list`,
//! `:1651-1745`) and `repeat(count, value)` (`evaluate_count`, `vector_variable_assign_expr_with_count`,
//! `vector_variable_new_from_array`, `vector_variable_assign_array`, `:1387-1394, 1465-1490,
//! 1602-1640`) store a typed vector, and a whole vector takes no scalar (`:1428-1463, 1577-1600`).

use super::placeholder_assignment_checks::assert_output_matches_native;
use super::placeholder_deferred_misuse::{DEFERRED_MESSAGE, assert_rejection_matches_native};

const NOT_NUMERIC: &str = "Right side is not a numeric expression";
const NOT_BOOLEAN: &str = "Right side is not a boolean expression";
const VECTOR_TO_SCALAR: &str = "Cannot assign a vector value to a scalar variable.";
const NOT_A_COUNT: &str = "Expected number of elements to fill a vector with.";
const NEGATIVE_COUNT: &str = "Negative number of elements specified.";

#[test]
fn initializer_lists_declare_vectors_of_the_deduced_type() {
    assert_output_matches_native(
        "vector-variables/deduced-types",
        "{local i = (1, 2, 3)}{local f = (1, 2.5)}{local s = (1, \"a\", true, 2.5)}\
         {local b = (true, false)}{local d = (print_time_sec, 1)}{global g = (1, 2)}\
         ; VP {i[2]} {size(i)} {f[0]} {f[1]} {s[0]}|{s[1]}|{s[2]}|{s[3]} {b[1]} {d[1]} {g[1]}",
    );
}

#[test]
fn assigned_lists_take_the_vector_type() {
    assert_output_matches_native(
        "vector-variables/assigned-types",
        "{local i = (1, 2)}{i = (3.7, 4)}{local f = (1.5)}{f = (1, 2)}{local s = (\"a\")}\
         {s = (true, 1.5)}{local b = (true)}{b = (false, true)}\
         ; VP {i[0]} {i[1]} {f[0]} {s[0]}{s[1]} {b[0]} {b[1]}",
    );
}

#[test]
fn a_one_element_list_assigns_a_scalar_and_declares_a_vector() {
    assert_output_matches_native(
        "vector-variables/one-element",
        "{local s = 1}{s = (5.5)}{local t = 1}{local t = (7.5)}{local v = (5)}\
         ; VP {s} {t} {v[0]} {size(v)}",
    );
}

#[test]
fn an_assigned_list_replaces_the_whole_vector_even_when_indexed() {
    assert_output_matches_native(
        "vector-variables/indexed-list",
        "{local v = (1, 2)}{v[1] = (7)}{local w = (1, 2)}{w[0] = (1, 2, 3)}{local e = (1, 2)}\
         {e = ()}{local r = (1)}{local r = ()}{position = (1, 2, 3)}\
         ; VP {size(v)} {v[0]} {size(w)} {size(e)} {size(r)} {position[0]} {position[2]}",
    );
}

#[test]
fn repeat_fills_vectors_of_the_value_type() {
    assert_output_matches_native(
        "vector-variables/repeat",
        "{local f = repeat(3, 1.5)}{local b = repeat(2, true)}{local s = repeat(2, \"x\")}\
         {local e = repeat(0, 1)}{local v = (2.5)}{local v = repeat(3, 1)}{local w = (1)}\
         {w = repeat(2, 1.5)}{local x = 3}{local c = (1, 2)}{c = repeat(x, x)}\
         ; VP {f[2]} {size(f)} {b[1]} {s[1]} {size(e)} {v[2]} {w[1]} {size(c)}",
    );
}

#[test]
fn an_empty_list_cannot_declare_a_vector() {
    assert_rejection_matches_native(
        "vector-variables/empty-list",
        "{local v = ()}; X",
        "Cannot create vector variable from an empty initializer list",
    );
}

#[test]
fn a_list_mixing_numbers_and_booleans_cannot_declare_a_vector() {
    assert_rejection_matches_native(
        "vector-variables/mixed-list",
        "{local v = (1, true)}; X",
        "Mixing numeric and boolean types.",
    );
}

#[test]
fn a_string_cannot_join_an_int_vector() {
    assert_rejection_matches_native(
        "vector-variables/string-into-ints",
        "{local v = (1, 2)}{v = (\"a\")}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_boolean_cannot_follow_a_number_in_an_int_vector() {
    assert_rejection_matches_native(
        "vector-variables/bool-into-ints",
        "{local v = (1, 2)}{v = (1, true)}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_number_cannot_join_a_bool_vector() {
    assert_rejection_matches_native(
        "vector-variables/number-into-bools",
        "{local v = (true)}{v = (1)}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_deferred_value_cannot_join_a_bool_vector() {
    assert_rejection_matches_native(
        "vector-variables/deferred-into-bools",
        "{local v = (true)}{v = (print_time_sec)}; X",
        DEFERRED_MESSAGE,
    );
}

#[test]
fn a_type_error_in_a_list_element_fails_like_native() {
    assert_rejection_matches_native(
        "vector-variables/element-type-error",
        "{local v = (1, 2)}{v = (1 - \"a\")}; X",
        "Cannot subtract non-numeric types.",
    );
}

#[test]
fn a_list_of_two_cannot_be_assigned_to_a_scalar() {
    assert_rejection_matches_native(
        "vector-variables/list-to-scalar",
        "{local s = 1}{s = (1, 2)}; X",
        VECTOR_TO_SCALAR,
    );
}

#[test]
fn a_list_of_two_cannot_redeclare_a_scalar() {
    assert_rejection_matches_native(
        "vector-variables/list-redeclares-scalar",
        "{local s = 1}{local s = (1, 2)}; X",
        VECTOR_TO_SCALAR,
    );
}

#[test]
fn a_list_cannot_be_assigned_to_a_config_vector() {
    assert_rejection_matches_native(
        "vector-variables/list-to-config",
        "{nozzle_diameter = (1)}; X",
        "Cannot modify a read-only variable",
    );
}

#[test]
fn repeat_needs_an_integer_count() {
    assert_rejection_matches_native(
        "vector-variables/fractional-count",
        "{local v = repeat(1.5, 2)}; X",
        NOT_A_COUNT,
    );
}

#[test]
fn repeat_into_a_vector_needs_an_integer_count() {
    assert_rejection_matches_native(
        "vector-variables/string-count",
        "{local v = (1, 2)}{v = repeat(\"a\", 2)}; X",
        NOT_A_COUNT,
    );
}

#[test]
fn repeat_needs_a_known_count() {
    assert_rejection_matches_native(
        "vector-variables/deferred-count",
        "{local v = repeat(print_time_sec, 1)}; X",
        DEFERRED_MESSAGE,
    );
}

#[test]
fn repeat_rejects_a_negative_count() {
    assert_rejection_matches_native(
        "vector-variables/negative-count",
        "{local v = repeat(-1, 2)}; X",
        NEGATIVE_COUNT,
    );
}

#[test]
fn repeat_redeclaring_a_vector_rejects_a_negative_count() {
    assert_rejection_matches_native(
        "vector-variables/negative-count-redeclared",
        "{local v = (1, 2)}{local v = repeat(-2, 1)}; X",
        NEGATIVE_COUNT,
    );
}

#[test]
fn repeat_cannot_be_assigned_to_a_scalar() {
    assert_rejection_matches_native(
        "vector-variables/repeat-to-scalar",
        "{local s = 1}{s = repeat(2, 1)}; X",
        VECTOR_TO_SCALAR,
    );
}

#[test]
fn repeat_cannot_redeclare_a_scalar() {
    assert_rejection_matches_native(
        "vector-variables/repeat-redeclares-scalar",
        "{local s = 1}{local s = repeat(2, 1)}; X",
        VECTOR_TO_SCALAR,
    );
}

#[test]
fn repeat_cannot_fill_a_bool_vector_with_a_number() {
    assert_rejection_matches_native(
        "vector-variables/repeat-number-into-bools",
        "{local v = (true)}{v = repeat(2, 1)}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn repeat_cannot_resize_an_output_vector() {
    assert_rejection_matches_native(
        "vector-variables/repeat-resizes-output",
        "{e_retracted = repeat(1, 2)}; X",
        "\"e_retracted\" output variable must not be resized by the script.",
    );
}

#[test]
fn a_scalar_cannot_be_assigned_to_a_whole_vector() {
    assert_rejection_matches_native(
        "vector-variables/scalar-to-vector",
        "{local v = (1, 2)}{v = 3}; X",
        "Referencing an output vector variable when scalar is expected",
    );
}

#[test]
fn a_function_result_cannot_be_assigned_to_a_whole_vector() {
    assert_rejection_matches_native(
        "vector-variables/function-to-vector",
        "{local v = (1, 2)}{v = min(1, 2)}; X",
        "Referencing an output vector variable when scalar is expected",
    );
}

#[test]
fn a_scalar_cannot_redeclare_a_vector() {
    assert_rejection_matches_native(
        "vector-variables/scalar-redeclares-vector",
        "{local v = (1, 2)}{local v = 3}; X",
        "Cannot assign a scalar value to a vector variable.",
    );
}
