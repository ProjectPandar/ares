use super::*;

#[tokio::test]
async fn print_bed_placeholders_render_default_printable_area() {
    let output = slice_print_bed_placeholders_output(json!({
        "machine_start_gcode": ";BED [print_bed_min] [print_bed_max] [print_bed_size]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BED 0,0 200,200 200,200", ";LAYER_CHANGE");
}

#[tokio::test]
async fn print_bed_placeholders_render_configured_string_area() {
    let output = slice_print_bed_placeholders_output(json!({
        "machine_start_gcode": ";BED [print_bed_min] [print_bed_max] [print_bed_size]",
        "printable_area": "-5.5x1.25,205x1.25,205x215.75,-5.5x215.75"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";BED -5.5,1.25 205,215.75 210.5,214.5",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn print_bed_placeholders_render_configured_json_area() {
    let output = slice_print_bed_placeholders_output(json!({
        "machine_start_gcode": ";BED [print_bed_min] [print_bed_max] [print_bed_size]",
        "printable_area": [[10, 20], [230, 20], [230, 225], [10, 225]]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BED 10,20 230,225 220,205", ";LAYER_CHANGE");
}

#[tokio::test]
async fn print_bed_placeholders_accept_numeric_string_json_points() {
    let output = slice_print_bed_placeholders_output(json!({
        "machine_start_gcode": ";BED [print_bed_min] [print_bed_max] [print_bed_size]",
        "printable_area": [["-5.5", "1.25"], ["205", "1.25"], ["205", "215.75"], ["-5.5", "215.75"]]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";BED -5.5,1.25 205,215.75 210.5,214.5",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn print_bed_placeholders_reject_invalid_printable_area_values() {
    for (case, value) in [
        ("empty string", json!("")),
        ("empty array", json!([])),
        ("one point string", json!("0x0")),
        ("one point array", json!([[0, 0]])),
        ("malformed string point", json!("0,1x1")),
        ("repeated x separator", json!("0x0x1,1x1")),
        ("json point not pair", json!([[0, 0, 0], [1, 1]])),
        ("non numeric coordinate", json!([["bad", 0], [1, 1]])),
        ("non finite coordinate", json!("NaNx0,1x1")),
        ("wrong top-level type", json!({"x": 0, "y": 0})),
    ] {
        let err = slice_print_bed_placeholders_output(json!({
            "machine_start_gcode": ";BED [print_bed_min]",
            "printable_area": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)), "{case}: {err}");
        assert!(err.to_string().contains("printable_area"), "{case}: {err}");
    }
}

#[tokio::test]
async fn print_bed_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_print_bed_placeholders_output(json!({
        "layer_change_gcode": ";LC [print_bed_min] [print_bed_size] [layer_num]",
        "printable_area": [[10, 20], [230, 20], [230, 225], [10, 225]]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [print_bed_min] [print_bed_size] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn invalid_printable_area_is_ignored_when_print_bed_placeholders_are_unused() {
    let output = slice_print_bed_placeholders_output(json!({
        "machine_start_gcode": ";START",
        "printable_area": "0x0x1,1x1"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START", ";LAYER_CHANGE");
}

async fn slice_print_bed_placeholders_output(
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
