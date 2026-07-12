use super::*;
use crate::PrintPathRole;

#[test]
fn filament_retract_lift_enforce_top_only_overrides_unprefixed_all_surfaces() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "All Surfaces",
            "filament_retract_lift_enforce": ["Top Only", "All Surfaces"],
            "z_hop_types": ["Normal Lift"],
            "gcode_comments": true
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    )
    .unwrap();
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
    assert!(!second.lines().any(|line| line.contains("lift Z")));
    assert!(!second.lines().any(|line| line.contains("restore layer Z")));
}

#[test]
fn nil_filament_retract_lift_enforce_falls_back_to_unprefixed_mode() {
    let output = synthetic_role_layers_output(
        json!({
            "retract_when_changing_layer": true,
            "retract_lift_enforce": "Top Only",
            "filament_retract_lift_enforce": "nil,All Surfaces",
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
fn invalid_filament_retract_lift_gate_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("filament_retract_lift_above", json!([])),
        ("filament_retract_lift_above", json!(-0.1)),
        ("filament_retract_lift_above", json!("inf")),
        ("filament_retract_lift_above", json!("bad")),
        ("filament_retract_lift_above", json!([0.0, "bad"])),
        ("filament_retract_lift_below", json!(-0.1)),
        ("filament_retract_lift_below", json!([0.0, -0.1])),
        ("filament_retract_lift_enforce", json!([])),
        ("filament_retract_lift_enforce", json!("bad")),
        ("filament_retract_lift_enforce", json!("")),
        ("filament_retract_lift_enforce", json!("All Surfaces,bad")),
        (
            "filament_retract_lift_enforce",
            json!(["All Surfaces", "bad"]),
        ),
        ("filament_retract_lift_enforce", json!([null, "bad"])),
        ("filament_retract_lift_enforce", json!([1])),
    ] {
        let err = synthetic_role_layers_output(
            json!({
                "retract_when_changing_layer": true,
                key: value
            }),
            vec![
                vec![PrintPathRole::ExternalPerimeter],
                vec![PrintPathRole::ExternalPerimeter],
            ],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key} missing from {err}");
    }
}
