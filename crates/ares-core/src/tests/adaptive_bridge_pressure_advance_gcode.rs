use super::*;

#[test]
fn adaptive_bridge_pressure_advance_overrides_bridge_moves_and_restores_base() {
    let output = adaptive_bridge_pressure_advance_output(
        json!({
            "enable_pressure_advance": [true],
            "pressure_advance": [0.04],
            "adaptive_pressure_advance": [true],
            "adaptive_pressure_advance_bridges": [0.012]
        }),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    )
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec![
            "M900 K0.04; Override pressure advance value",
            "M900 K0.012; Override pressure advance value",
            "M900 K0.04; Override pressure advance value",
        ]
    );
    assert_line_before_prefix(
        &output,
        "M900 K0.012; Override pressure advance value",
        ";EXTRUSION:print:bridge:",
    );
    assert_last_line_before_prefix(
        &output,
        "M900 K0.04; Override pressure advance value",
        ";EXTRUSION:print:sparse_infill:",
    );
}

#[test]
fn adaptive_bridge_pressure_advance_applies_to_internal_bridge_and_overhang() {
    for role in [
        PrintPathRole::InternalBridge,
        PrintPathRole::OverhangPerimeter,
    ] {
        let output = adaptive_bridge_pressure_advance_output(
            json!({
                "enable_pressure_advance": true,
                "pressure_advance": 0.03,
                "adaptive_pressure_advance": true,
                "adaptive_pressure_advance_bridges": 0.011
            }),
            vec![role, PrintPathRole::SparseInfill],
        )
        .unwrap();

        assert_eq!(
            pressure_advance_lines(&output),
            vec![
                "M900 K0.03; Override pressure advance value",
                "M900 K0.011; Override pressure advance value",
                "M900 K0.03; Override pressure advance value",
            ]
        );
        assert_line_before_prefix(
            &output,
            "M900 K0.011; Override pressure advance value",
            &format!(";EXTRUSION:print:{}:", role.as_str()),
        );
    }
}

#[test]
fn adaptive_bridge_pressure_advance_ignores_non_bridge_roles() {
    let output = adaptive_bridge_pressure_advance_output(
        json!({
            "enable_pressure_advance": true,
            "pressure_advance": 0.04,
            "adaptive_pressure_advance": true,
            "adaptive_pressure_advance_bridges": 0.012
        }),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    )
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec!["M900 K0.04; Override pressure advance value"]
    );
}

#[test]
fn zero_bridge_pressure_advance_disables_runtime_override() {
    let output = adaptive_bridge_pressure_advance_output(
        json!({
            "enable_pressure_advance": true,
            "pressure_advance": 0.04,
            "adaptive_pressure_advance": true,
            "adaptive_pressure_advance_bridges": 0.0
        }),
        vec![PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec!["M900 K0.04; Override pressure advance value"]
    );
}

#[test]
fn adaptive_bridge_pressure_advance_requires_base_and_adaptive_enablement() {
    let cases = [
        (
            json!({
                "enable_pressure_advance": false,
                "pressure_advance": 0.04,
                "adaptive_pressure_advance": true,
                "adaptive_pressure_advance_bridges": 0.012
            }),
            Vec::<&str>::new(),
        ),
        (
            json!({
                "enable_pressure_advance": true,
                "pressure_advance": 0.04,
                "adaptive_pressure_advance": false,
                "adaptive_pressure_advance_bridges": 0.012
            }),
            vec!["M900 K0.04; Override pressure advance value"],
        ),
    ];

    for (extra, expected) in cases {
        let output = adaptive_bridge_pressure_advance_output(
            extra,
            vec![PrintPathRole::Bridge, PrintPathRole::SparseInfill],
        )
        .unwrap();

        assert_eq!(pressure_advance_lines(&output), expected);
    }
}

#[test]
fn adaptive_bridge_pressure_advance_uses_first_numeric_value_from_supported_forms() {
    let cases = [
        (
            json!("0.013, 0.014"),
            "M900 K0.013; Override pressure advance value",
        ),
        (
            json!("0.015; 0.016"),
            "M900 K0.015; Override pressure advance value",
        ),
        (
            json!([0.017, 0.018]),
            "M900 K0.017; Override pressure advance value",
        ),
        (
            json!(["0.019", "bad"]),
            "M900 K0.019; Override pressure advance value",
        ),
    ];

    for (value, expected_bridge) in cases {
        let output = adaptive_bridge_pressure_advance_output(
            json!({
                "enable_pressure_advance": true,
                "pressure_advance": 0.04,
                "adaptive_pressure_advance": true,
                "adaptive_pressure_advance_bridges": value
            }),
            vec![PrintPathRole::Bridge, PrintPathRole::SparseInfill],
        )
        .unwrap();

        assert!(pressure_advance_lines(&output).contains(&expected_bridge));
    }
}

#[test]
fn adaptive_bridge_pressure_advance_suppresses_duplicate_bridge_state_commands() {
    let output = adaptive_bridge_pressure_advance_output(
        json!({
            "enable_pressure_advance": true,
            "pressure_advance": 0.04,
            "adaptive_pressure_advance": true,
            "adaptive_pressure_advance_bridges": 0.012
        }),
        vec![
            PrintPathRole::Bridge,
            PrintPathRole::InternalBridge,
            PrintPathRole::OverhangPerimeter,
            PrintPathRole::SparseInfill,
        ],
    )
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec![
            "M900 K0.04; Override pressure advance value",
            "M900 K0.012; Override pressure advance value",
            "M900 K0.04; Override pressure advance value",
        ]
    );
}

#[test]
fn adaptive_pressure_advance_rejects_invalid_enabled_values() {
    for value in [json!(1), json!("true"), json!([]), json!(["bad", true])] {
        let err = adaptive_bridge_pressure_advance_output(
            json!({
                "enable_pressure_advance": true,
                "pressure_advance": 0.04,
                "adaptive_pressure_advance": value,
                "adaptive_pressure_advance_bridges": 0.012
            }),
            vec![PrintPathRole::Bridge],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("adaptive_pressure_advance"));
    }
}

#[test]
fn adaptive_bridge_pressure_advance_rejects_invalid_values() {
    for value in [
        json!(-0.01),
        json!(2.01),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!([]),
    ] {
        let err = adaptive_bridge_pressure_advance_output(
            json!({
                "enable_pressure_advance": true,
                "pressure_advance": 0.04,
                "adaptive_pressure_advance": true,
                "adaptive_pressure_advance_bridges": value
            }),
            vec![PrintPathRole::Bridge],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("adaptive_pressure_advance_bridges")
        );
    }
}

fn adaptive_bridge_pressure_advance_output(
    extra: serde_json::Value,
    roles: Vec<PrintPathRole>,
) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "line_width": 0.4,
            "filament_diameter": [2.0]
        }),
        extra,
    );
    let pipeline =
        crate::pipeline::layer_change_test_support::role_layers_pipeline(&options, vec![roles]);
    crate::gcode::format_gcode(&pipeline, &options).map(|bytes| String::from_utf8(bytes).unwrap())
}

fn pressure_advance_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| {
            line.starts_with("M900 K")
                || line.starts_with("SET_PRESSURE_ADVANCE")
                || line.starts_with("M572 D0 S")
                || line.starts_with("M233 X")
        })
        .collect()
}

fn assert_line_before_prefix(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn assert_last_line_before_prefix(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().rposition(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
