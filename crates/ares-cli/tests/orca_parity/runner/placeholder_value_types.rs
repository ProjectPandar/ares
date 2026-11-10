//! `PlaceholderParser.cpp:1170-1232` (`scalar_variable_to_expr`) and `:1355-1361`
//! (`vector_element_to_expr`): a variable reaches an expression with the type of its option.

use super::placeholder_deferred_misuse::export;
use crate::runner::OrcaRunner;
use crate::{self as parity};

fn assert_output_matches_native(label: &str, line: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(&runner, label, line);
    let case = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn an_integer_option_divides_as_an_integer() {
    assert_output_matches_native(
        "placeholder-value-types/integer-option",
        "; QUARTER={wall_loops / 4}",
    );
}

#[test]
fn an_integer_option_is_an_integer_digits_width() {
    assert_output_matches_native(
        "placeholder-value-types/integer-width",
        "; WALLS={digits(1, wall_loops, 0)}",
    );
}

#[test]
fn a_percent_option_is_a_number() {
    assert_output_matches_native(
        "placeholder-value-types/percent-option",
        "; DENSITY={sparse_infill_density + 1}",
    );
}

#[test]
fn a_numeric_string_option_stays_text() {
    assert_output_matches_native(
        "placeholder-value-types/string-option",
        "; VARIANT={printer_variant + 1}",
    );
}

#[test]
fn a_numeric_enum_option_stays_text() {
    assert_output_matches_native(
        "placeholder-value-types/enum-option",
        "; TIMELAPSE={timelapse_type + 1}",
    );
}

#[test]
fn an_integer_runtime_placeholder_divides_as_an_integer() {
    assert_output_matches_native(
        "placeholder-value-types/integer-placeholder",
        "; HALF={num_extruders / 2}",
    );
}
