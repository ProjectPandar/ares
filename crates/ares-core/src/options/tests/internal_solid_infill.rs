use super::super::*;
use serde_json::json;

#[test]
fn internal_solid_infill_pattern_defaults_to_monotonic() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(
        infill.internal_solid_infill_pattern(),
        InfillPattern::Monotonic
    );
}

#[test]
fn detect_narrow_internal_solid_infill_defaults_to_true_and_parses_bool() {
    let default = SliceOptions::default().infill_options().unwrap();
    assert!(default.detect_narrow_internal_solid_infill());

    let disabled: SliceOptions = serde_json::from_value(json!({
        "detect_narrow_internal_solid_infill": false
    }))
    .unwrap();
    assert!(
        !disabled
            .infill_options()
            .unwrap()
            .detect_narrow_internal_solid_infill()
    );

    let invalid: SliceOptions = serde_json::from_value(json!({
        "detect_narrow_internal_solid_infill": "false"
    }))
    .unwrap();
    let err = invalid.infill_options().unwrap_err();
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("detect_narrow_internal_solid_infill")
    );
}

#[test]
fn top_and_bottom_surface_patterns_default_to_orca_values() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.top_surface_pattern(), InfillPattern::MonotonicLine);
    assert_eq!(infill.bottom_surface_pattern(), InfillPattern::Monotonic);
}

#[test]
fn parses_top_and_bottom_surface_patterns() {
    for key in ["top_surface_pattern", "bottom_surface_pattern"] {
        for (value, expected) in [
            ("rectilinear", InfillPattern::Rectilinear),
            ("alignedrectilinear", InfillPattern::AlignedRectilinear),
            ("monotonic", InfillPattern::Monotonic),
            ("monotonicline", InfillPattern::MonotonicLine),
        ] {
            let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

            let infill = options.infill_options().unwrap();
            let actual = if key == "top_surface_pattern" {
                infill.top_surface_pattern()
            } else {
                infill.bottom_surface_pattern()
            };

            assert_eq!(actual, expected, "{key}={value}");
        }
    }
}

#[test]
fn parses_concentric_solid_surface_patterns() {
    for key in [
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: "concentric" })).unwrap();
        let infill = options.infill_options().unwrap();
        let actual = match key {
            "top_surface_pattern" => infill.top_surface_pattern(),
            "bottom_surface_pattern" => infill.bottom_surface_pattern(),
            "internal_solid_infill_pattern" => infill.internal_solid_infill_pattern(),
            _ => unreachable!(),
        };

        assert_eq!(actual, InfillPattern::Concentric, "{key}");
    }
}

#[test]
fn rejects_unimplemented_top_and_bottom_surface_patterns_with_key_names() {
    for (key, value) in [
        ("top_surface_pattern", json!("hilbertcurve")),
        ("bottom_surface_pattern", json!("octagramspiral")),
        ("bottom_surface_pattern", json!(1)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{err}");
    }
}

#[test]
fn parses_internal_solid_infill_patterns_and_solid_rotation_options() {
    for (value, expected) in [
        ("rectilinear", InfillPattern::Rectilinear),
        ("alignedrectilinear", InfillPattern::AlignedRectilinear),
        ("line", InfillPattern::Line),
        ("grid", InfillPattern::Grid),
        ("zigzag", InfillPattern::ZigZag),
        ("monotonic", InfillPattern::Monotonic),
        ("monotonicline", InfillPattern::MonotonicLine),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "internal_solid_infill_pattern": value,
            "solid_infill_direction": 30,
            "solid_infill_rotate_template": "90,0"
        }))
        .unwrap();
        let infill = options.infill_options().unwrap();

        assert_eq!(infill.internal_solid_infill_pattern(), expected);
        assert_eq!(infill.solid_direction_degrees(), 30.0);
        assert_eq!(infill.solid_infill_rotate_template_degrees(), &[90.0, 0.0]);
    }
}

#[test]
fn rejects_invalid_internal_solid_infill_options_with_key_names() {
    for (key, value) in [
        ("internal_solid_infill_pattern", json!("gyroid")),
        ("internal_solid_infill_pattern", json!("concentric_internal")),
        ("internal_solid_infill_pattern", json!(1)),
        ("solid_infill_direction", json!(361)),
        ("solid_infill_rotate_template", json!("90,")),
        ("solid_infill_rotate_template", json!(90)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{err}");
    }
}

#[test]
fn direct_spiral_mode_exposes_zero_effective_sparse_density() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 100,
        "spiral_mode": true
    }))
    .unwrap();

    assert_eq!(options.infill_options().unwrap().sparse_density_percent(), 0.0);
}
