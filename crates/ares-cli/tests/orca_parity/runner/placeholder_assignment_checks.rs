//! `PlaceholderParser.cpp:2435-2462` assignment and declaration statements: only `local` and
//! `global` variables are writable (`check_writable`, `:1377-1380`), declarations must not
//! shadow another scope (`new_old_variable`, `:1529-1564`), and an assigned variable keeps its
//! type (`scalar_variable_assign_scalar`, `:1396-1426`).

use super::placeholder_deferred_misuse::{DEFERRED_MESSAGE, assert_rejection_matches_native};
use crate::runner::OrcaRunner;
use crate::{self as parity};

const READ_ONLY: &str = "Cannot modify a read-only variable";
const SYSTEM_SYMBOL: &str = "Symbol is already defined in read-only system dictionary";
const SYSTEM_OUTPUT: &str = "Symbol is already defined as system output variable";
const NOT_NUMERIC: &str = "Right side is not a numeric expression";
const NOT_BOOLEAN: &str = "Right side is not a boolean expression";

fn assert_output_matches_native(label: &str, line: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = super::placeholder_deferred_misuse::export(&runner, label, line);
    let case = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn assigning_a_config_value_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/config-value",
        "{layer_height = 0.3}; X",
        READ_ONLY,
    );
}

#[test]
fn assigning_a_runtime_value_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/runtime-value",
        "{current_extruder = 1}; X",
        READ_ONLY,
    );
}

#[test]
fn assigning_a_config_vector_element_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/config-element",
        "{nozzle_diameter[0] = 0.6}; X",
        READ_ONLY,
    );
}

#[test]
fn a_deferred_value_assigned_to_a_config_value_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/config-deferred",
        "{layer_height = print_time_sec}; X",
        READ_ONLY,
    );
}

#[test]
fn a_type_error_assigned_to_a_config_value_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/config-type-error",
        "{layer_height = \"a\" - 1}; X",
        "Cannot subtract non-numeric types.",
    );
}

#[test]
fn assigning_an_unknown_name_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/unknown-name",
        "{zzz = 1}; X",
        "Not a variable name",
    );
}

#[test]
fn an_unknown_name_is_reported_before_its_value() {
    assert_rejection_matches_native(
        "assignment-checks/unknown-name-type-error",
        "{zzz = \"a\" - 1}; X",
        "Not a variable name",
    );
}

#[test]
fn declaring_a_config_name_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/declare-config",
        "{local layer_height = 1}; X",
        SYSTEM_SYMBOL,
    );
}

#[test]
fn declaring_a_runtime_name_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/declare-runtime",
        "{local current_extruder = 1}; X",
        SYSTEM_SYMBOL,
    );
}

#[test]
fn declaring_a_system_output_name_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/declare-output",
        "{local position = 1}; X",
        SYSTEM_OUTPUT,
    );
}

#[test]
fn declaring_a_global_system_output_name_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/declare-global-output",
        "{global e_retracted = 1}; X",
        SYSTEM_OUTPUT,
    );
}

#[test]
fn a_local_over_a_global_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/local-over-global",
        "{global g = 1}{local g = 2}; X",
        "Variable name already defined in global scope",
    );
}

#[test]
fn a_global_over_a_local_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/global-over-local",
        "{local y = 1}{global y = 2}; X",
        "Variable name already defined in local scope",
    );
}

#[test]
fn a_string_assigned_to_an_int_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/int-from-string",
        "{local x = 1}{x = \"a\"}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_bool_assigned_to_an_int_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/int-from-bool",
        "{local x = 1}{x = true}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_string_redeclaring_an_int_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/redeclare-int-string",
        "{local x = 1}{local x = \"a\"}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_string_assigned_to_an_int_in_a_block_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/block-int-from-string",
        "{local x = 1; x = \"a\"}; X",
        NOT_NUMERIC,
    );
}

#[test]
fn a_number_assigned_to_a_bool_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/bool-from-number",
        "{local b = true}{b = 1}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_deferred_value_assigned_to_a_bool_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/bool-from-deferred",
        "{local b = true}{b = print_time_sec}; X",
        NOT_BOOLEAN,
    );
}

#[test]
fn a_deferred_value_assigned_to_an_int_fails_like_native() {
    assert_rejection_matches_native(
        "assignment-checks/int-from-deferred",
        "{local x = 1}{x = print_time_sec}; X",
        DEFERRED_MESSAGE,
    );
}

#[test]
fn an_int_variable_truncates_an_assigned_double() {
    assert_output_matches_native(
        "assignment-checks/int-truncates",
        "{local x = 1}{x = 1.7}; X={x}",
    );
}

#[test]
fn an_int_variable_truncates_a_negative_double_toward_zero() {
    assert_output_matches_native(
        "assignment-checks/int-truncates-negative",
        "{local x = 1}{x = -1.7}; X={x}",
    );
}

#[test]
fn a_redeclared_int_keeps_its_type() {
    assert_output_matches_native(
        "assignment-checks/redeclared-int",
        "{local x = 1}{local x = 2.9}; X={x}",
    );
}

#[test]
fn an_int_assigned_in_a_block_keeps_its_type() {
    assert_output_matches_native(
        "assignment-checks/block-int",
        "{local x = 1; x = 2.5}; X={x}",
    );
}

#[test]
fn a_redeclared_global_keeps_its_type() {
    assert_output_matches_native(
        "assignment-checks/redeclared-global",
        "{global g = 1}{global g = 2.5}; G={g}",
    );
}

#[test]
fn a_float_variable_keeps_an_assigned_int_a_double() {
    assert_output_matches_native(
        "assignment-checks/float-from-int",
        "{local x = 1.5}{x = 2}; X={x / 4}",
    );
}

#[test]
fn a_string_variable_keeps_an_assigned_number_a_string() {
    assert_output_matches_native(
        "assignment-checks/string-from-number",
        "{local s = \"a\"}{s = 5}; S={s + 1}",
    );
}

#[test]
fn assigning_a_system_output_element_is_allowed() {
    assert_output_matches_native(
        "assignment-checks/output-element",
        "{position[2] = 7}; Z={position[2]}",
    );
}

#[test]
fn a_global_variable_prints_like_native() {
    assert_output_matches_native("assignment-checks/global", "{global g = 1}; G={g}");
}

#[test]
fn a_redeclared_local_takes_the_new_value() {
    assert_output_matches_native(
        "assignment-checks/redeclared-local",
        "{local x = 1}{local x = 2}; X={x}",
    );
}

#[test]
fn a_string_variable_takes_a_bool_as_text() {
    assert_output_matches_native(
        "assignment-checks/string-from-bool",
        "{local s = \"a\"}{s = true}; S={s}",
    );
}

#[test]
fn a_string_variable_takes_a_double_as_text() {
    assert_output_matches_native(
        "assignment-checks/string-from-double",
        "{local s = \"a\"}{s = 0.1 + 0.2}; S={s}",
    );
}
