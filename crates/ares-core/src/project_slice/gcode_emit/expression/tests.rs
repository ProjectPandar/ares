use super::*;
use crate::project_slice::gcode_emit::value::Config;

fn config() -> Config {
    Config::from_block(b"; n = 2\n; values = 3,5\n; type = PLA\n")
}

#[test]
fn expression_supports_indexing_arithmetic_and_functions() {
    assert_eq!(
        evaluate("values[1]", &config()).unwrap().as_number(),
        Some(5.0)
    );
    assert_eq!(
        evaluate("ceil(n / 2)", &config()).unwrap().as_number(),
        Some(1.0)
    );
    assert_eq!(
        evaluate("values[1] + ceil(n / 2)", &config())
            .unwrap()
            .as_number(),
        Some(6.0)
    );
    assert!(
        evaluate("type == \"PLA\" && max(n, 3) == 3", &config())
            .unwrap()
            .as_bool()
    );
}

#[test]
fn expression_interpolates_piecewise_table_tuples() {
    assert_eq!(
        evaluate(
            "interpolate_table(n, (0,4000), (4,2000), (10,1000))",
            &config()
        )
        .unwrap()
        .as_number(),
        Some(3000.0)
    );
}

#[test]
fn expression_random_is_bounded_and_advances_state() {
    let config = config();
    let first = evaluate("random(-160, -152)", &config)
        .unwrap()
        .as_number()
        .unwrap();
    let second = evaluate("random(-160, -152)", &config)
        .unwrap()
        .as_number()
        .unwrap();
    assert!((-160.0..=-152.0).contains(&first));
    assert!((-160.0..=-152.0).contains(&second));
    assert_eq!(first.fract(), 0.0);
    assert_ne!(first, second);
}

#[test]
fn expression_supports_regex_match_and_nonmatch() {
    let config = Config::from_block(b"; notes = PRINTER_MODEL_MINIIS HF_NOZZLE\n");
    assert!(
        evaluate("notes=~/.*PRINTER_MODEL_MINI.*/", &config)
            .unwrap()
            .as_bool()
    );
    assert!(evaluate("notes!~/.*MK4S.*/", &config).unwrap().as_bool());
}

#[test]
fn expression_formats_digits_with_width_and_decimals() {
    assert_eq!(
        evaluate("digits(values[1], 8, 2)", &config())
            .unwrap()
            .as_string(),
        "    5.00"
    );
    assert_eq!(
        evaluate("digits(values[1], 3)", &config())
            .unwrap()
            .as_string(),
        "  5"
    );
}

#[test]
fn expression_supports_word_operators_and_boolean_literals() {
    assert!(evaluate("true and not false", &config()).unwrap().as_bool());
    assert!(evaluate("false or n == 2", &config()).unwrap().as_bool());
    assert!(!evaluate("n != 2 or false", &config()).unwrap().as_bool());
}

#[test]
fn expression_supports_modulo_unary_plus_and_conditionals() {
    assert_eq!(
        evaluate("+values[1] % 2", &config()).unwrap().as_number(),
        Some(1.0)
    );
    assert_eq!(
        evaluate("n > 1 ? values[0] : values[1]", &config())
            .unwrap()
            .as_number(),
        Some(3.0)
    );
}
