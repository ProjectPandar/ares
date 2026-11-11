//! `PlaceholderParser.cpp:2435-2463` vector copies and element assignments: a naked vector
//! reference copies a vector of the same type (`copy_vector_variable_to_vector_variable`,
//! `vector_variable_new_from_copy`, `:1747-1818`), and an element assignment stays inside the
//! vector and takes its type (`vector_variable_element_assign_scalar`, `:1428-1463`).

use super::placeholder_assignment_checks::assert_output_matches_native;
use super::placeholder_deferred_misuse::assert_rejection_matches_native;

const SCALAR_TO_VECTOR: &str = "Cannot assign scalar to a vector";
const NIL_ELEMENTS: &str =
    "Some elements of the right hand side vector variable of optional values are undefined (nil)";
const NOT_DUPLICABLE: &str = "Duplicating this type of vector variable is not supported";
const INT_VECTOR: &str = "Left hand side is an int vector, while the right hand side is not.";
const INDEX_OUT_OF_RANGE: &str = "Index out of range";
const NOT_NUMERIC: &str = "Right side is not a numeric expression";
const NOT_BOOLEAN: &str = "Right side is not a boolean expression";

#[test]
fn naked_vector_references_copy_the_vector() {
    assert_output_matches_native(
        "vector-copies/copies",
        "{local n = nozzle_diameter}{local p = filament_shrink}{local t = filament_type}\
         {local w = t}{w[0] = \"c\"}{local b = (true)}{b = filament_soluble}{local f = (1.5, 2)}\
         {local g = f}{g = (1, 2)}{local x = wipe_tower_x}\
         ; VP {n[0]} {size(n)} {p[0]} {t[0]}{w[0]} {b[0]} {g[0]} {x[0]}",
    );
}

#[test]
fn indexed_and_scalar_references_declare_scalars() {
    assert_output_matches_native(
        "vector-copies/scalar-references",
        "{local v = (1, 2)}{local w = v[0] + 1}{local s = 1}{local s = nozzle_diameter}\
         {local t = 1}{t = nozzle_diameter}{local u = layer_height}; VP {w} {s} {t} {u}",
    );
}

#[test]
fn assigned_elements_take_the_vector_type() {
    assert_output_matches_native(
        "vector-copies/element-types",
        "{local i = (1, 2)}{i[1] = 7.9}{local s = (\"a\", 1)}{s[0] = 1.5}{local t = filament_type}\
         {t[0] = 5}; VP {i[1]} {s[0]} {t[0]}",
    );
}

#[test]
fn a_float_config_vector_cannot_be_copied_into_an_int_vector() {
    assert_rejection_matches_native(
        "vector-copies/floats-into-ints",
        "{local v = (1, 2)}{v = nozzle_diameter}; X",
        INT_VECTOR,
    );
}

#[test]
fn a_float_config_vector_cannot_redeclare_an_int_vector() {
    assert_rejection_matches_native(
        "vector-copies/floats-redeclare-ints",
        "{local v = (1, 2)}{local v = nozzle_diameter}; X",
        INT_VECTOR,
    );
}

#[test]
fn an_output_vector_cannot_be_copied_into_an_int_vector() {
    assert_rejection_matches_native(
        "vector-copies/output-into-ints",
        "{local v = (1, 2)}{v = position}; X",
        INT_VECTOR,
    );
}

#[test]
fn a_percent_vector_cannot_be_copied_into_a_float_vector() {
    assert_rejection_matches_native(
        "vector-copies/percents-into-floats",
        "{local v = (1.5)}{v = filament_shrink}; X",
        "Left hand side is a float vector, while the right hand side is not.",
    );
}

#[test]
fn a_scalar_config_value_cannot_be_copied_into_a_vector() {
    assert_rejection_matches_native(
        "vector-copies/scalar-into-vector",
        "{local v = (1, 2)}{v = layer_height}; X",
        SCALAR_TO_VECTOR,
    );
}

#[test]
fn a_vector_element_cannot_be_copied_into_a_vector() {
    assert_rejection_matches_native(
        "vector-copies/element-into-vector",
        "{local v = (1, 2)}{v = nozzle_diameter[0]}; X",
        SCALAR_TO_VECTOR,
    );
}

#[test]
fn a_vector_element_in_an_expression_cannot_be_copied_into_a_vector() {
    assert_rejection_matches_native(
        "vector-copies/element-expression-into-vector",
        "{local v = (1, 2)}{v = nozzle_diameter[0] + 1}; X",
        SCALAR_TO_VECTOR,
    );
}

#[test]
fn copying_an_unknown_name_fails_like_native() {
    assert_rejection_matches_native(
        "vector-copies/unknown-name",
        "{local v = (1, 2)}{v = zzz}; X",
        "Not a variable name",
    );
}

#[test]
fn an_empty_int_vector_counts_as_nil_and_cannot_be_copied() {
    assert_rejection_matches_native(
        "vector-copies/empty-ints-copied",
        "{local e = repeat(0, 1)}{local v = (1, 2)}{v = e}; X",
        NIL_ELEMENTS,
    );
}

#[test]
fn an_empty_bool_vector_counts_as_nil_and_cannot_redeclare_a_vector() {
    assert_rejection_matches_native(
        "vector-copies/empty-bools-redeclare",
        "{local e = repeat(0, true)}{local v = (false)}{local v = e}; X",
        NIL_ELEMENTS,
    );
}

#[test]
fn an_empty_float_vector_counts_as_nil_and_cannot_declare_a_vector() {
    assert_rejection_matches_native(
        "vector-copies/empty-floats",
        "{local e = repeat(0, 1.5)}{local f = e}; X",
        NIL_ELEMENTS,
    );
}

#[test]
fn a_point_vector_cannot_be_duplicated() {
    assert_rejection_matches_native(
        "vector-copies/points",
        "{local v = printable_area}; X",
        NOT_DUPLICABLE,
    );
}

#[test]
fn an_enum_vector_cannot_be_duplicated() {
    assert_rejection_matches_native(
        "vector-copies/enums",
        "{local v = overhang_fan_threshold}; X",
        NOT_DUPLICABLE,
    );
}

#[test]
fn an_element_index_past_the_end_fails_like_native() {
    assert_rejection_matches_native(
        "vector-copies/index-past-end",
        "{local v = (1, 2)}{v[5] = 3}; X",
        INDEX_OUT_OF_RANGE,
    );
}

#[test]
fn an_output_element_index_past_the_end_fails_like_native() {
    assert_rejection_matches_native(
        "vector-copies/output-index-past-end",
        "{position[9] = 5}; X",
        INDEX_OUT_OF_RANGE,
    );
}

#[test]
fn an_element_of_an_empty_vector_cannot_be_assigned() {
    assert_rejection_matches_native(
        "vector-copies/empty-vector-element",
        "{local v = repeat(0, 1)}{v[0] = 3}; X",
        "Indexing an empty vector variable",
    );
}

#[test]
fn a_boolean_cannot_be_assigned_to_an_int_element() {
    assert_rejection_matches_native(
        "vector-copies/bool-into-int-element",
        "{local v = (1, 2)}{v[0] = true}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_string_cannot_be_assigned_to_an_output_element() {
    assert_rejection_matches_native(
        "vector-copies/string-into-output-element",
        "{e_retracted[0] = \"a\"}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_number_cannot_be_assigned_to_a_bool_element() {
    assert_rejection_matches_native(
        "vector-copies/number-into-bool-element",
        "{local v = (true)}{v[0] = 1}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_deferred_value_assigned_to_a_bool_element_is_not_a_boolean() {
    assert_rejection_matches_native(
        "vector-copies/deferred-into-bool-element",
        "{local v = (true)}{v[0] = print_time_sec}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn an_empty_string_vector_is_not_nil_and_is_copied() {
    assert_output_matches_native(
        "vector-copies/empty-strings",
        "{local e = repeat(0, \"a\")}{local v = e}{local w = (\"b\")}{w = e}\
         ; VP {size(v)} {size(w)}",
    );
}
