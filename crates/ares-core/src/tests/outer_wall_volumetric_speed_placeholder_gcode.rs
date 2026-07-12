use super::*;

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_renders_uncapped_value() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 50,
        "outer_wall_line_width": 0.5,
        "filament_max_volumetric_speed": 20
    }))
    .await
    .unwrap();

    assert_numeric_line_before(
        &output,
        ";OWVS ",
        50.0 * outer_wall_material_area(0.5, 0.2),
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_caps_to_filament_max() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 80,
        "outer_wall_line_width": 0.6,
        "filament_max_volumetric_speed": 6
    }))
    .await
    .unwrap();

    assert_numeric_line_before(&output, ";OWVS ", 6.0, ";LAYER_CHANGE");
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_respects_zero_filament_max() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 50,
        "outer_wall_line_width": 0.5,
        "filament_max_volumetric_speed": 0
    }))
    .await
    .unwrap();

    assert_numeric_line_before(&output, ";OWVS ", 0.0, ";LAYER_CHANGE");
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_uses_flow_ratios() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 40,
        "outer_wall_line_width": 0.5,
        "filament_max_volumetric_speed": 20,
        "set_other_flow_ratios": true,
        "outer_wall_flow_ratio": 1.25,
        "print_flow_ratio": 1.1,
        "filament_flow_ratio": [0.9]
    }))
    .await
    .unwrap();

    assert_numeric_line_before(
        &output,
        ";OWVS ",
        40.0 * outer_wall_material_area(0.5, 0.2) * 1.25 * 1.1 * 0.9,
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_uses_line_width_fallback() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 50,
        "line_width": 0.45,
        "filament_max_volumetric_speed": 20
    }))
    .await
    .unwrap();

    assert_numeric_line_before(
        &output,
        ";OWVS ",
        50.0 * outer_wall_material_area(0.45, 0.2),
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_uses_nozzle_default_width() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 50,
        "nozzle_diameter": 0.4,
        "filament_max_volumetric_speed": 20
    }))
    .await
    .unwrap();

    assert_numeric_line_before(
        &output,
        ";OWVS ",
        50.0 * outer_wall_material_area(0.45, 0.2),
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_converts_filament_e_back_to_material_volume() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
        "outer_wall_speed": 50,
        "outer_wall_line_width": 0.5,
        "filament_diameter": 2.85,
        "filament_max_volumetric_speed": 20
    }))
    .await
    .unwrap();

    assert_numeric_line_before(
        &output,
        ";OWVS ",
        50.0 * outer_wall_material_area(0.5, 0.2),
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_rejects_invalid_inputs() {
    for (key, value) in [
        ("outer_wall_speed", json!(-1)),
        ("outer_wall_line_width", json!(-0.1)),
        ("filament_max_volumetric_speed", json!(-0.1)),
        ("layer_height", json!(0)),
    ] {
        let err = slice_outer_wall_volumetric_speed_output(json!({
            "machine_start_gcode": ";OWVS [outer_wall_volumetric_speed]",
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[tokio::test]
async fn outer_wall_volumetric_speed_placeholder_stays_literal_in_layer_change_scope() {
    let output = slice_outer_wall_volumetric_speed_output(json!({
        "layer_change_gcode": ";LC [outer_wall_volumetric_speed] [layer_num]",
        "outer_wall_speed": 50,
        "outer_wall_line_width": 0.5,
        "filament_max_volumetric_speed": 20
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [outer_wall_volumetric_speed] 1",
        "; segment_count = 4",
    );
}

async fn slice_outer_wall_volumetric_speed_output(
    extra: serde_json::Value,
) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        extra,
    );
    slice(square_pyramid_ascii_stl(), options)
        .await
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn assert_line_before(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn assert_numeric_line_before(output: &str, prefix: &str, expected: f64, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| line.starts_with(prefix))
        .unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
    let actual = lines[first_index]
        .strip_prefix(prefix)
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

fn outer_wall_material_area(width: f64, layer_height: f64) -> f64 {
    layer_height * (width - layer_height * (1.0 - std::f64::consts::PI / 4.0))
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
