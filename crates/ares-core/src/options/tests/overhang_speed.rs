use super::super::*;
use crate::{PrintPathRole, ToolpathMoveKind};
use serde_json::json;

#[test]
fn percent_overhang_4_4_speed_resolves_over_outer_wall_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "outer_wall_speed": 80,
        "bridge_speed": 25,
        "enable_overhang_speed": true,
        "overhang_4_4_speed": "50%"
    }))
    .unwrap();

    assert_eq!(
        options.speed_options().unwrap().speed_for_role(
            ToolpathMoveKind::Print,
            PrintPathRole::OverhangPerimeter
        ),
        40.0
    );
}

#[test]
fn zero_or_disabled_overhang_speed_uses_bridge_fallback() {
    for extra in [
        json!({ "overhang_4_4_speed": 0 }),
        json!({ "enable_overhang_speed": false, "overhang_4_4_speed": 35 }),
    ] {
        let mut value = json!({
            "outer_wall_speed": 80,
            "bridge_speed": 25
        });
        for (key, extra_value) in extra.as_object().unwrap() {
            value[key] = extra_value.clone();
        }
        let options: SliceOptions = serde_json::from_value(value).unwrap();

        assert_eq!(
            options.speed_options().unwrap().speed_for_role(
                ToolpathMoveKind::Print,
                PrintPathRole::OverhangPerimeter
            ),
            25.0
        );
    }
}

#[test]
fn overhang_speed_bands_parse_numeric_and_percent_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "outer_wall_speed": 80,
        "bridge_speed": 25,
        "enable_overhang_speed": true,
        "overhang_1_4_speed": "75%",
        "overhang_2_4_speed": 35,
        "overhang_3_4_speed": "50%",
        "overhang_4_4_speed": 20
    }))
    .unwrap();
    let speed_options = options.speed_options().unwrap();

    assert_eq!(
        speed_options.overhang_speed_for_unsupported_span_mm(0.1),
        Some(60.0)
    );
    assert_eq!(
        speed_options.overhang_speed_for_unsupported_span_mm(0.2),
        Some(35.0)
    );
    assert_eq!(
        speed_options.overhang_speed_for_unsupported_span_mm(0.3),
        Some(40.0)
    );
    assert_eq!(
        speed_options.overhang_speed_for_unsupported_span_mm(0.5),
        Some(20.0)
    );
}

#[test]
fn disabled_overhang_speed_bands_return_none() {
    let options: SliceOptions = serde_json::from_value(json!({
        "outer_wall_speed": 80,
        "bridge_speed": 25,
        "enable_overhang_speed": false,
        "overhang_1_4_speed": 60,
        "overhang_2_4_speed": 50,
        "overhang_3_4_speed": 40,
        "overhang_4_4_speed": 30
    }))
    .unwrap();

    assert_eq!(
        options
            .speed_options()
            .unwrap()
            .overhang_speed_for_unsupported_span_mm(0.1),
        None
    );
}

#[test]
fn slowdown_for_curled_perimeters_defaults_to_final_overhang_band() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "outer_wall_speed": 80,
        "bridge_speed": 25,
        "overhang_4_4_speed": 20
    }))
    .unwrap();

    assert_eq!(
        options
            .speed_options()
            .unwrap()
            .overhang_speed_for_unsupported_span_mm(0.5),
        Some(20.0)
    );
}

#[test]
fn disabled_slowdown_for_curled_perimeters_uses_bridge_speed_for_final_bucket() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "outer_wall_speed": 80,
        "bridge_speed": 25,
        "slowdown_for_curled_perimeters": false,
        "overhang_4_4_speed": 20
    }))
    .unwrap();

    assert_eq!(
        options
            .speed_options()
            .unwrap()
            .overhang_speed_for_unsupported_span_mm(0.5),
        Some(25.0)
    );
}

#[test]
fn invalid_slowdown_for_curled_perimeters_is_rejected_when_overhang_speed_is_disabled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "enable_overhang_speed": false,
        "slowdown_for_curled_perimeters": "false"
    }))
    .unwrap();

    assert!(matches!(
        options.speed_options(),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn rejects_invalid_overhang_speed_values() {
    for (key, value) in [
        ("enable_overhang_speed", json!("true")),
        ("enable_overhang_speed", json!(1)),
        ("enable_overhang_speed", json!([true])),
        ("enable_overhang_speed", json!({ "enabled": true })),
        ("slowdown_for_curled_perimeters", json!("false")),
        ("slowdown_for_curled_perimeters", json!(0)),
        ("slowdown_for_curled_perimeters", json!([false])),
        (
            "slowdown_for_curled_perimeters",
            json!({ "enabled": false }),
        ),
        ("overhang_1_4_speed", json!(-1)),
        ("overhang_1_4_speed", json!("-1%")),
        ("overhang_1_4_speed", json!("fast")),
        ("overhang_1_4_speed", json!([20])),
        ("overhang_1_4_speed", json!(true)),
        ("overhang_1_4_speed", json!({ "speed": 20 })),
        ("overhang_1_4_speed", json!("NaN")),
        ("overhang_1_4_speed", json!("inf")),
        ("overhang_2_4_speed", json!(-1)),
        ("overhang_2_4_speed", json!("-1%")),
        ("overhang_2_4_speed", json!("fast")),
        ("overhang_2_4_speed", json!([20])),
        ("overhang_2_4_speed", json!(true)),
        ("overhang_2_4_speed", json!({ "speed": 20 })),
        ("overhang_2_4_speed", json!("NaN")),
        ("overhang_2_4_speed", json!("inf")),
        ("overhang_3_4_speed", json!(-1)),
        ("overhang_3_4_speed", json!("-1%")),
        ("overhang_3_4_speed", json!("fast")),
        ("overhang_3_4_speed", json!([20])),
        ("overhang_3_4_speed", json!(true)),
        ("overhang_3_4_speed", json!({ "speed": 20 })),
        ("overhang_3_4_speed", json!("NaN")),
        ("overhang_3_4_speed", json!("inf")),
        ("overhang_4_4_speed", json!(-1)),
        ("overhang_4_4_speed", json!("-1%")),
        ("overhang_4_4_speed", json!("fast")),
        ("overhang_4_4_speed", json!([20])),
        ("overhang_4_4_speed", json!(true)),
        ("overhang_4_4_speed", json!({ "speed": 20 })),
        ("overhang_4_4_speed", json!("NaN")),
        ("overhang_4_4_speed", json!("inf")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        assert!(matches!(
            options.speed_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
