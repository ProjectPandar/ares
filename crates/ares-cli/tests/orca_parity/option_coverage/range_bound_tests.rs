use super::*;

#[test]
fn numeric_bounds_do_not_accept_prefixes_of_unsupported_expressions() {
    for expression in [
        "1 / 2",
        "10 * scale",
        "max_temp / 2",
        "0.3f * scale",
        "1f",
        "0.3ff",
        "1e999f",
    ] {
        let block = format!("def->min = 0; def->max = {expression};");
        assert_eq!(numeric_bounds(&block), None, "{expression}");
        assert_eq!(raw_numeric_bounds(&block), None, "{expression}");
        assert_eq!(upper_bound(&block), None, "{expression}");
        assert!(incomplete_numeric_domain(&block).contains("unsupported"));
    }
}

#[test]
fn omitted_bounds_report_missing_endpoints_not_expression_failures() {
    for (block, missing) in [
        ("def->min = 0;", "upper endpoint"),
        ("def->max = 100;", "lower endpoint"),
        ("", "implicit endpoints"),
    ] {
        let reason = incomplete_numeric_domain(block);
        assert!(reason.contains(missing), "{reason}");
        assert!(reason.contains("full-range seed"), "{reason}");
        assert!(!reason.contains("unsupported"), "{reason}");
    }
}

#[test]
fn numeric_bounds_ignore_comments_and_parse_whole_literals() {
    assert_eq!(numeric_bounds("// def->min = 1;\ndef->max = 10;"), None);
    assert_eq!(
        numeric_bounds("def->min = -2e-3; def->max = +1.5e2;"),
        Some((-0.002, 150.0))
    );
    assert_eq!(
        numeric_bounds("def->min = -max_temp; def->max = max_temp;"),
        Some((-1500.0, 1500.0))
    );
}

#[test]
fn numeric_bounds_accept_complete_suffixed_floating_literals() {
    for (minimum, maximum, expected) in [
        ("0.f", "0.3f", (0.0, 0.3)),
        ("-.005F", "+.5f", (-0.005, 0.5)),
        ("-2e-3f", "1.E2F", (-0.002, 100.0)),
    ] {
        let block = format!("def->min = {minimum}; def->max = {maximum};");
        assert_eq!(numeric_bounds(&block), Some(expected));
        assert_eq!(raw_numeric_bounds(&block), Some(expected));
    }
}
