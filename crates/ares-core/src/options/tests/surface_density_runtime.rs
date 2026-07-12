use super::super::*;
use serde_json::json;

#[test]
fn surface_densities_default_to_full_solid_surfaces() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.top_surface_density_percent(), 100.0);
    assert_eq!(infill.bottom_surface_density_percent(), 100.0);
}

#[test]
fn parses_top_surface_density_runtime_values() {
    for (value, expected) in [(json!(0), 0.0), (json!(100), 100.0), (json!("75"), 75.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "top_surface_density": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().top_surface_density_percent(),
            expected
        );
    }
}

#[test]
fn parses_bottom_surface_density_runtime_values() {
    for (value, expected) in [(json!(10), 10.0), (json!(100), 100.0), (json!("75"), 75.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "bottom_surface_density": value })).unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .bottom_surface_density_percent(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_top_surface_density_values() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "top_surface_density": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("top_surface_density"), "{err}");
    }
}

#[test]
fn rejects_invalid_bottom_surface_density_values() {
    for value in [
        json!(9.9),
        json!(100.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "bottom_surface_density": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("bottom_surface_density"), "{err}");
    }
}

#[test]
fn elephant_foot_layers_density_defaults_to_full_internal_solid_density() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.elephant_foot_layers_density_percent(), 100.0);
    assert_eq!(infill.elephant_foot_compensation_layers(), 1);
}

#[test]
fn parses_elephant_foot_layers_density_runtime_values() {
    for (value, expected) in [(json!(50), 50.0), (json!(100), 100.0), (json!("75"), 75.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "elefant_foot_layers_density": value })).unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .elephant_foot_layers_density_percent(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_elephant_foot_layers_density_values() {
    for value in [
        json!(49.9),
        json!(100.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "elefant_foot_layers_density": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("elefant_foot_layers_density"), "{err}");
    }
}

#[test]
fn rejects_invalid_elephant_foot_compensation_layers_values() {
    for value in [json!(0), json!(-1), json!("bad"), json!(false), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "elefant_foot_compensation_layers": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("elefant_foot_compensation_layers"), "{err}");
    }
}

#[test]
fn min_width_top_surface_defaults_to_three_times_effective_inner_wall_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "line_width": 0.5,
        "inner_wall_line_width": "80%"
    }))
    .unwrap();

    assert_approx_eq(
        options.infill_options().unwrap().min_width_top_surface_mm(),
        0.96,
    );
}

#[test]
fn parses_min_width_top_surface_mm_and_percent_values() {
    for (value, expected) in [
        (json!(0), 0.0),
        (json!("0.75"), 0.75),
        (json!("250%"), 1.0),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "nozzle_diameter": [0.4],
            "inner_wall_line_width": 0.4,
            "min_width_top_surface": value
        }))
        .unwrap();

        assert_approx_eq(
            options.infill_options().unwrap().min_width_top_surface_mm(),
            expected,
        );
    }
}

#[test]
fn rejects_invalid_min_width_top_surface_values() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "min_width_top_surface": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("min_width_top_surface"), "{err}");
    }
}

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}
