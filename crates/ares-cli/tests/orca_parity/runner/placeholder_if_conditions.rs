//! `PlaceholderParser.cpp:2341-2349` `if_else_output`: every `if` and `elsif` condition must be
//! a boolean (`bool_expr_eval`, `:2408`), and condition and assignment failures are reported at
//! their operand.

use super::placeholder_deferred_misuse::{DEFERRED_MESSAGE, assert_rejection_matches_native};
use crate::runner::OrcaRunner;
use crate::{self as parity};

const NOT_BOOLEAN: &str = "Not a boolean expression";
const SUBTRACT: &str = "Cannot subtract non-numeric types.";

#[test]
fn a_numeric_if_condition_fails_like_native() {
    assert_rejection_matches_native("if-conditions/numeric", "{if 1}; ONE{endif}", NOT_BOOLEAN);
}

#[test]
fn an_elsif_after_a_taken_arm_is_still_checked() {
    assert_rejection_matches_native(
        "if-conditions/elsif-after-taken",
        "{if true}; A{elsif 1}; B{endif}",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_multiline_elsif_reports_its_own_line() {
    assert_rejection_matches_native(
        "if-conditions/multiline-elsif",
        "{if true}\n; A\n{elsif 1}\n; B\n{endif}",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_numeric_macro_if_condition_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/macro-numeric",
        "{if 1 then local x = 1; endif}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_macro_elsif_after_a_taken_arm_is_still_checked() {
    assert_rejection_matches_native(
        "if-conditions/macro-elsif-after-taken",
        "{if true then local x = 1; elsif 2 then local y = 2; endif}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_type_error_in_an_if_condition_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/type-error",
        "{if \"2\" - 1 > 0}; X{endif}",
        SUBTRACT,
    );
}

#[test]
fn a_deferred_value_in_an_if_condition_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/deferred",
        "{if print_time_sec > 0}; X{endif}",
        DEFERRED_MESSAGE,
    );
}

#[test]
fn a_bare_deferred_if_condition_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/bare-deferred",
        "{if print_time_sec}; X{endif}",
        DEFERRED_MESSAGE,
    );
}

#[test]
fn an_unknown_variable_in_an_if_condition_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/unknown-variable",
        "{if no_such_variable}; X{endif}",
        "Not a variable name",
    );
}

#[test]
fn a_type_error_in_a_declaration_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/declaration",
        "{local x = \"2\" - 1}; X={x}",
        SUBTRACT,
    );
}

#[test]
fn a_type_error_in_an_assignment_fails_like_native() {
    assert_rejection_matches_native(
        "if-conditions/assignment",
        "{local x = 1}{x = \"2\" - 1}; X={x}",
        SUBTRACT,
    );
}

#[test]
fn a_block_inside_a_skipped_arm_is_not_evaluated() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = super::placeholder_deferred_misuse::export(
        &runner,
        "if-conditions/skipped-block",
        "{if false}{if 1}; A{endif}{endif}; B",
    );
    let case = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
