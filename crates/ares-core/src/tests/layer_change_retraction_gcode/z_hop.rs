use super::*;
use crate::PrintPathRole;

#[tokio::test]
async fn default_z_hop_lifts_after_second_layer_z_and_restores_before_unretract() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let lift = line_index(second, "G1 Z0.8 F7200 ; lift Z");
    let restore = line_index(second, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(retract < z);
    assert!(z < lift);
    assert!(lift < restore);
    assert!(restore < unretract);
    assert!(unretract < first_print);
    assert!(
        !layer_section(&output, 0)
            .lines()
            .any(|line| line.contains("lift Z") || line.contains("restore layer Z"))
    );
}

#[tokio::test]
async fn zero_z_hop_preserves_no_hop_layer_change_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");

    assert!(z < unretract);
    assert!(!second.lines().any(|line| line.contains("lift Z")));
    assert!(!second.lines().any(|line| line.contains("restore layer Z")));
}

#[tokio::test]
async fn z_hop_lift_gates_use_pre_change_z() {
    let below_lower = output_for(json!({
        "retract_when_changing_layer": true,
        "z_hop": 0.3,
        "retract_lift_above": 0.3,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;
    let above_upper = output_for(json!({
        "retract_when_changing_layer": true,
        "z_hop": 0.3,
        "retract_lift_below": 0.1,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;
    let inside = output_for(json!({
        "retract_when_changing_layer": true,
        "z_hop": 0.3,
        "retract_lift_above": 0.2,
        "retract_lift_below": 0.2,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;

    assert!(
        !layer_section(&below_lower, 1)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
    assert!(
        !layer_section(&above_upper, 1)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
    assert!(
        layer_section(&inside, 1)
            .lines()
            .any(|line| line == "G1 Z0.7 F7200 ; lift Z")
    );
}

#[tokio::test]
async fn firmware_retraction_z_hop_keeps_firmware_commands_and_no_e_retract() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "use_firmware_retraction": true,
        "z_hop": 0.2,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G10 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let lift = line_index(second, "G1 Z0.6 F7200 ; lift Z");
    let restore = line_index(second, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = line_index(second, "G11 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(retract < z);
    assert!(z < lift);
    assert!(lift < restore);
    assert!(restore < unretract);
    assert!(unretract < first_print);
    assert!(!second.lines().any(|line| line.starts_with("G1 E-")));
    assert!(
        !second
            .lines()
            .any(|line| line.starts_with("G1 E") && line.contains("unretract"))
    );
}

#[tokio::test]
async fn z_hop_invalid_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("z_hop", json!([])),
        ("z_hop", json!(-0.1)),
        ("z_hop", json!("inf")),
        ("retract_lift_above", json!(-0.1)),
        ("retract_lift_below", json!("inf")),
    ] {
        let err = output_result(json!({
            "retract_when_changing_layer": true,
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains(key),
            "{key} was missing from {err}"
        );
    }
}

#[test]
fn retract_lift_enforce_all_surfaces_keeps_lift_after_non_top_role() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "All Surfaces",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        layer_section(&output, 1)
            .lines()
            .any(|line| line == "G1 Z0.8 F7200 ; lift Z")
    );
}

#[test]
fn retract_lift_enforce_top_only_suppresses_non_top_role() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        !layer_section(&output, 1)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
}

#[test]
fn retract_lift_enforce_top_only_allows_after_top_role() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::TopSolidInfill],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        layer_section(&output, 1)
            .lines()
            .any(|line| line == "G1 Z0.8 F7200 ; lift Z")
    );
}

#[test]
fn retract_lift_enforce_bottom_only_lifts_only_first_layer_change() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Bottom Only",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        layer_section(&output, 1)
            .lines()
            .any(|line| line == "G1 Z0.8 F7200 ; lift Z")
    );
    assert!(
        !layer_section(&output, 2)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
}

#[test]
fn retract_lift_enforce_top_and_bottom_lifts_after_top_or_first_layer() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Top and Bottom",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::TopSolidInfill],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        layer_section(&output, 1)
            .lines()
            .any(|line| line == "G1 Z0.8 F7200 ; lift Z")
    );
    assert!(
        !layer_section(&output, 2)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
    assert!(
        layer_section(&output, 3)
            .lines()
            .any(|line| line == "G1 Z1.2 F7200 ; lift Z")
    );
}

#[test]
fn retract_lift_enforce_string_array_uses_index_zero() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": ["Top Only", "All Surfaces"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        !layer_section(&output, 1)
            .lines()
            .any(|line| line.contains("lift Z"))
    );
}

#[test]
fn retract_lift_enforce_gap_fill_preserves_previous_non_gap_fill_role() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::TopSolidInfill],
            vec![PrintPathRole::GapFill],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();

    assert!(
        layer_section(&output, 2)
            .lines()
            .any(|line| line == "G1 Z1 F7200 ; lift Z")
    );
}

#[test]
fn retract_lift_enforce_invalid_values_are_rejected_with_option_key() {
    for value in [
        json!("bad"),
        json!(""),
        json!([]),
        json!(["All Surfaces", "bad"]),
        json!([1]),
    ] {
        let err = synthetic_role_layers_output(
            json!({
                "retract_when_changing_layer": true,
                "retract_lift_enforce": value
            }),
            vec![
                vec![PrintPathRole::ExternalPerimeter],
                vec![PrintPathRole::ExternalPerimeter],
            ],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("retract_lift_enforce"),
            "retract_lift_enforce was missing from {err}"
        );
    }
}
