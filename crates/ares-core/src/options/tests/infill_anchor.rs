use super::super::*;
use serde_json::json;

#[test]
fn infill_anchor_defaults_match_orca_runtime_spacing() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.infill_anchor_length_mm(), 8.0);
}

#[test]
fn parses_percent_infill_anchor_over_sparse_spacing() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": "50%"
    }))
    .unwrap();

    assert_eq!(options.infill_options().unwrap().infill_anchor_length_mm(), 0.5);
}

#[test]
fn parses_numeric_string_infill_anchor_values() {
    let anchor: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": "0.75",
        "infill_anchor_max": 20
    }))
    .unwrap();
    let max: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": 2,
        "infill_anchor_max": "0.75"
    }))
    .unwrap();

    assert_eq!(
        anchor.infill_options().unwrap().infill_anchor_length_mm(),
        0.75
    );
    assert_eq!(
        max.infill_options().unwrap().infill_anchor_length_mm(),
        0.75
    );
}

#[test]
fn infill_anchor_max_clamps_effective_anchor_length() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": 2,
        "infill_anchor_max": 0.25
    }))
    .unwrap();

    assert_eq!(
        options.infill_options().unwrap().infill_anchor_length_mm(),
        0.25
    );
}

#[test]
fn zero_infill_anchor_disables_effective_anchor_length() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": 0
    }))
    .unwrap();

    assert_eq!(options.infill_options().unwrap().infill_anchor_length_mm(), 0.0);
}

#[test]
fn zero_infill_anchor_max_disables_effective_anchor_length() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "infill_anchor_max": 0
    }))
    .unwrap();

    assert_eq!(options.infill_options().unwrap().infill_anchor_length_mm(), 0.0);
}

#[test]
fn zero_density_keeps_percent_infill_anchor_parse_finite() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 0,
        "sparse_infill_line_width": 0.5,
        "infill_anchor": "50%"
    }))
    .unwrap();

    assert_eq!(
        options.infill_options().unwrap().infill_anchor_length_mm(),
        0.25
    );
}

#[test]
fn rejects_invalid_infill_anchor_values() {
    for key in ["infill_anchor", "infill_anchor_max"] {
        for value in [
            json!(-0.1),
            json!("bad"),
            json!("NaN"),
            json!("inf"),
            json!(null),
            json!(true),
            json!([1]),
            json!({"value": 1}),
        ] {
            let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

            assert!(matches!(
                options.infill_options(),
                Err(SliceError::InvalidInput(message)) if message.contains(key)
            ));
        }
    }
}
