use super::*;

#[tokio::test]
async fn machine_start_wipe_tower_placeholders_render_current_no_wipe_tower_state() {
    let output = slice_wipe_tower_placeholders_output(json!({
        "machine_start_gcode": ";WT [has_wipe_tower] [has_single_extruder_multi_material_priming] [total_toolchanges]"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";WT 0 0 0",
        ";WT [has_wipe_tower] [has_single_extruder_multi_material_priming] [total_toolchanges]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn wipe_tower_placeholders_compose_with_multi_nozzle_context() {
    let output = slice_wipe_tower_placeholders_output(json!({
        "machine_start_gcode": ";WT [num_extruders] [has_wipe_tower] [total_toolchanges] [total_layer_count]",
        "nozzle_diameter": ["0.4", "0.6", "0.8"]
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";WT 3 0 0 2",
        ";WT 3 [has_wipe_tower] [total_toolchanges] 2",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn single_extruder_priming_placeholder_preserves_no_wipe_tower_state() {
    for priming in [false, true] {
        let output = slice_wipe_tower_placeholders_output(json!({
            "machine_start_gcode": ";WT [has_single_extruder_multi_material_priming]",
            "single_extruder_multi_material_priming": priming
        }))
        .await
        .unwrap();

        assert_rendered_line_before(
            &output,
            ";WT 0",
            ";WT [has_single_extruder_multi_material_priming]",
            ";LAYER_CHANGE",
        );
    }
}

#[tokio::test]
async fn invalid_single_extruder_priming_is_rejected_before_gcode_output() {
    let err = slice_wipe_tower_placeholders_output(json!({
        "machine_start_gcode": ";WT [has_single_extruder_multi_material_priming]",
        "single_extruder_multi_material_priming": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("single_extruder_multi_material_priming")
    );
}

#[tokio::test]
async fn wipe_tower_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_wipe_tower_placeholders_output(json!({
        "layer_change_gcode": ";LC [has_wipe_tower] [has_single_extruder_multi_material_priming] [total_toolchanges] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [has_wipe_tower] [has_single_extruder_multi_material_priming] [total_toolchanges] 1",
        "; segment_count = 4",
    );
}

async fn slice_wipe_tower_placeholders_output(
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

fn assert_rendered_line_before(output: &str, rendered: &str, literal: &str, next: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let rendered_index = lines.iter().position(|line| *line == rendered);
    let literal_present = lines.contains(&literal);

    assert!(
        rendered_index.is_some(),
        "missing rendered line {rendered:?}; literal present: {literal_present}"
    );

    let next_index = lines.iter().position(|line| *line == next).unwrap();
    assert!(
        rendered_index.unwrap() < next_index,
        "{} !< {next_index}",
        rendered_index.unwrap()
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
