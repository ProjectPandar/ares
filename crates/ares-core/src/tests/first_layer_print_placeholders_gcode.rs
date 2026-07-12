use super::*;

#[tokio::test]
async fn first_layer_print_placeholders_render_default_first_layer_bounds() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "machine_start_gcode": ";FLP [first_layer_print_min] [first_layer_print_max] [first_layer_print_size]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FLP -2.5,-2.5 2.5,2.5 5,5", ";LAYER_CHANGE");
}

#[tokio::test]
async fn first_layer_center_no_wipe_tower_renders_default_first_layer_center() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "machine_start_gcode": ";CENTER [first_layer_center_no_wipe_tower]"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";CENTER 0,0",
        ";CENTER [first_layer_center_no_wipe_tower]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn first_layer_center_no_wipe_tower_composes_with_first_layer_bounds() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "machine_start_gcode": ";FLP [first_layer_print_min] [first_layer_center_no_wipe_tower] [first_layer_print_max] [first_layer_print_size]"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";FLP -2.5,-2.5 0,0 2.5,2.5 5,5",
        ";FLP -2.5,-2.5 [first_layer_center_no_wipe_tower] 2.5,2.5 5,5",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn first_layer_print_placeholders_render_perimeter_bounds_when_skirt_disabled() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "machine_start_gcode": ";FLP [first_layer_print_min] [first_layer_print_max] [first_layer_print_size]",
        "skirt_loops": 0
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FLP -0.5,-0.5 0.5,0.5 1,1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn first_layer_print_placeholders_compose_with_existing_machine_start_placeholders() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "machine_start_gcode": ";START [first_layer_print_min] [first_layer_print_max] [first_layer_print_size] [print_bed_size] [total_layer_count]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";START -2.5,-2.5 2.5,2.5 5,5 200,200 2",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn first_layer_print_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "layer_change_gcode": ";LC [first_layer_print_min] [first_layer_print_size] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [first_layer_print_min] [first_layer_print_size] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn first_layer_center_no_wipe_tower_stays_literal_in_layer_change_scope() {
    let output = slice_first_layer_print_placeholders_output(json!({
        "layer_change_gcode": ";LC [first_layer_center_no_wipe_tower] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [first_layer_center_no_wipe_tower] 1",
        "; segment_count = 4",
    );
}

async fn slice_first_layer_print_placeholders_output(
    extra: serde_json::Value,
) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0
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
    let literal_present = lines.contains(&literal);
    let rendered_index = lines.iter().position(|line| *line == rendered);
    assert!(
        rendered_index.is_some(),
        "expected rendered line {rendered:?}; literal line present before implementation: {literal_present}"
    );
    let next_index = lines.iter().position(|line| *line == next).unwrap();
    assert!(rendered_index.unwrap() < next_index);
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
