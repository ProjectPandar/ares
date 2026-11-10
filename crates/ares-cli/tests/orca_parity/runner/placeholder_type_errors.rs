//! `PlaceholderParser.cpp:391-748,845-858` operator and function type errors: each operand
//! keeps its own type, so a numeric string or a boolean is not a number and a number is not a
//! boolean.

use super::placeholder_deferred_misuse::assert_rejection_matches_native;

#[test]
fn subtracting_from_a_numeric_string_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/subtract",
        "; X={\"2\" - 1}",
        "Cannot subtract non-numeric types.",
    );
}

#[test]
fn adding_a_number_to_a_boolean_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/add",
        "; X={true + 1}",
        "Cannot add non-numeric types.",
    );
}

#[test]
fn dividing_by_zero_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/divide",
        "; X={1 / 0}",
        "Division by zero",
    );
}

#[test]
fn negating_a_boolean_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/negate",
        "; X={-true}",
        "Cannot apply unary minus operator.",
    );
}

#[test]
fn inverting_a_number_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/not",
        "; X={!1}",
        "Cannot apply a not operator.",
    );
}

#[test]
fn a_logical_operation_on_a_number_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/and",
        "; X={1 and true}",
        "Cannot apply logical operation to non-boolean operands.",
    );
}

#[test]
fn a_numeric_ternary_condition_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/ternary",
        "; X={1 ? 2 : 3}",
        "Not a boolean expression",
    );
}

#[test]
fn ordering_booleans_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/compare",
        "; X={true < false}",
        "Cannot compare the types.",
    );
}

#[test]
fn converting_a_string_to_an_integer_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/int",
        "; X={int(\"3\")}",
        "Cannot convert to integer.",
    );
}

#[test]
fn the_minimum_of_a_string_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/min",
        "; X={min(1, \"2\")}",
        "Not a numeric type.",
    );
}

#[test]
fn a_fractional_digits_width_fails_like_native() {
    assert_rejection_matches_native(
        "placeholder-type-errors/digits",
        "; X={digits(1, 2.5)}",
        "digits: second parameter must be integer",
    );
}
