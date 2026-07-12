use super::*;

#[tokio::test]
async fn machine_start_enable_high_low_temp_mix_defaults_to_disabled() {
    let output = slice_enable_high_low_temp_mix_output(json!({
        "machine_start_gcode": ";MIX [enable_high_low_temp_mix]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MIX 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_enable_high_low_temp_mix_renders_enabled_preference() {
    let output = slice_enable_high_low_temp_mix_output(json!({
        "machine_start_gcode": ";MIX [enable_high_low_temp_mix]",
        "enable_high_low_temp_mixed_printing": true
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MIX 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_enable_high_low_temp_mix_renders_disabled_preference() {
    let output = slice_enable_high_low_temp_mix_output(json!({
        "machine_start_gcode": ";MIX [enable_high_low_temp_mix]",
        "enable_high_low_temp_mixed_printing": false
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MIX 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn enable_high_low_temp_mix_rejects_non_boolean_preference() {
    for value in [json!("true"), json!(1), json!(null)] {
        let err = slice_enable_high_low_temp_mix_output(json!({
            "machine_start_gcode": ";MIX [enable_high_low_temp_mix]",
            "enable_high_low_temp_mixed_printing": value
        }))
        .await
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("enable_high_low_temp_mixed_printing")
        );
    }
}

#[tokio::test]
async fn enable_high_low_temp_mix_stays_literal_in_layer_change_scope() {
    let output = slice_enable_high_low_temp_mix_output(json!({
        "layer_change_gcode": ";LC [enable_high_low_temp_mix] [layer_num]",
        "enable_high_low_temp_mixed_printing": true
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [enable_high_low_temp_mix] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn enable_high_low_temp_mix_composes_with_is_extruder_used() {
    let output = slice_enable_high_low_temp_mix_output(json!({
        "machine_start_gcode": ";START [enable_high_low_temp_mix] [is_extruder_used]",
        "enable_high_low_temp_mixed_printing": true
    }))
    .await
    .unwrap();

    let line = line_with_prefix(&output, ";START ");
    let rendered = line.strip_prefix(";START 1 ").unwrap();
    let used = rendered.split(',').collect::<Vec<_>>();

    assert_eq!(used.len(), 64);
    assert_eq!(used[0], "1");
    assert!(used[1..].iter().all(|value| *value == "0"));
}

async fn slice_enable_high_low_temp_mix_output(
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

fn line_with_prefix<'a>(output: &'a str, prefix: &str) -> &'a str {
    output
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
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
