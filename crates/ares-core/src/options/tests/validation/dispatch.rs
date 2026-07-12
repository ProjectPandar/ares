use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn missing_printer_technology_defaults_to_fff_validation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2
    }))
    .unwrap();

    let errors = options.validate_print_config(true).unwrap();

    assert_eq!(errors["layer_height"], "invalid value -0.2");
}

#[test]
fn explicit_fff_runs_fff_validation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "printer_technology": "FFF",
        "layer_height": -0.2
    }))
    .unwrap();

    let errors = options.validate_print_config(true).unwrap();

    assert_eq!(errors["layer_height"], "invalid value -0.2");
}

#[test]
fn sla_printer_technology_returns_empty_validation_map() {
    let options: SliceOptions = serde_json::from_value(json!({
        "printer_technology": "SLA",
        "layer_height": -0.2,
        "outer_wall_line_width": 1001
    }))
    .unwrap();

    let errors = options.validate_print_config(true).unwrap();

    assert!(errors.is_empty());
}

#[test]
fn dispatch_forwards_under_cli_to_fff_aggregate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "printer_technology": "FFF",
        "spiral_mode": true,
        "wall_loops": 2
    }))
    .unwrap();

    let cli_errors = options.validate_print_config(true).unwrap();
    let non_cli_errors = options.validate_print_config(false).unwrap();

    assert!(cli_errors.contains_key("wall_loops"));
    assert!(!non_cli_errors.contains_key("wall_loops"));
}

#[test]
fn invalid_printer_technology_values_return_invalid_input() {
    for value in [json!("FFF/SLA"), json!(1)] {
        let options: SliceOptions = serde_json::from_value(json!({
            "printer_technology": value
        }))
        .unwrap();

        let error = options.validate_print_config(true).unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn fff_aggregate_api_remains_callable_after_dispatch_addition() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2
    }))
    .unwrap();

    let errors = options.validate_fff_options(true).unwrap();

    assert_eq!(errors["layer_height"], "invalid value -0.2");
}
