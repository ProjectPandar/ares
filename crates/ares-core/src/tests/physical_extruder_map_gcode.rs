use super::*;

#[tokio::test]
async fn layer_change_gcode_uses_physical_extruder_map_for_most_used_placeholder() {
    let output = physical_extruder_output(json!({
        "physical_extruder_map": [2],
        "layer_change_gcode": ";PHYS-LAYER {most_used_physical_extruder_id} [most_used_physical_extruder_id]"
    }))
    .await
    .unwrap();

    assert_eq!(
        physical_lines(&output, ";PHYS-LAYER "),
        vec![";PHYS-LAYER 2 2", ";PHYS-LAYER 2 2"]
    );
}

#[tokio::test]
async fn time_lapse_gcode_uses_physical_extruder_map_for_current_and_most_used_placeholders() {
    let output = physical_extruder_output(json!({
        "physical_extruder_map": [3],
        "time_lapse_gcode": ";PHYS-TL {most_used_physical_extruder_id} [curr_physical_extruder_id]"
    }))
    .await
    .unwrap();

    assert_eq!(
        physical_lines(&output, ";PHYS-TL "),
        vec![";PHYS-TL 3 3", ";PHYS-TL 3 3"]
    );
}

#[tokio::test]
async fn wrapping_detection_gcode_uses_physical_extruder_map_placeholders() {
    let output = physical_extruder_output(json!({
        "physical_extruder_map": [4],
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";PHYS-WRAP {most_used_physical_extruder_id} [curr_physical_extruder_id]"
    }))
    .await
    .unwrap();

    assert_eq!(
        physical_lines(&output, ";PHYS-WRAP "),
        vec![";PHYS-WRAP 4 4", ";PHYS-WRAP 4 4"]
    );
}

#[tokio::test]
async fn missing_physical_extruder_map_keeps_logical_zero_default() {
    let output = physical_extruder_output(json!({
        "layer_change_gcode": ";PHYS-DEFAULT {most_used_physical_extruder_id}"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";PHYS-DEFAULT 0", "G1 Z0.2 F7200");
}

#[tokio::test]
async fn invalid_physical_extruder_map_is_rejected() {
    for value in [
        json!([]),
        json!([-1]),
        json!([1.5]),
        json!([true]),
        json!({"extruder": 1}),
        json!("bad"),
    ] {
        let err = physical_extruder_output(json!({
            "physical_extruder_map": value,
            "layer_change_gcode": ";PHYS {most_used_physical_extruder_id}"
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("physical_extruder_map"), "{err}");
    }
}

#[tokio::test]
async fn invalid_physical_extruder_map_is_rejected_without_custom_gcode_consumer() {
    let err = physical_extruder_output(json!({
        "physical_extruder_map": [1.5]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("physical_extruder_map"), "{err}");
}

#[test]
fn physical_extruder_map_uses_last_value_for_out_of_range_logical_extruder() {
    let options: SliceOptions = serde_json::from_value(json!({
        "physical_extruder_map": [2, 7]
    }))
    .unwrap();

    assert_eq!(options.physical_extruder_id_for_logical(0).unwrap(), 2);
    assert_eq!(options.physical_extruder_id_for_logical(1).unwrap(), 7);
    assert_eq!(options.physical_extruder_id_for_logical(5).unwrap(), 7);
}

#[test]
fn physical_extruder_map_accepts_scalar_and_string_forms() {
    let numeric: SliceOptions = serde_json::from_value(json!({
        "physical_extruder_map": 5
    }))
    .unwrap();
    let string: SliceOptions = serde_json::from_value(json!({
        "physical_extruder_map": "6"
    }))
    .unwrap();

    assert_eq!(numeric.physical_extruder_id_for_logical(0).unwrap(), 5);
    assert_eq!(string.physical_extruder_id_for_logical(0).unwrap(), 6);
}

#[test]
fn physical_extruder_map_rejects_invalid_values_directly() {
    for value in [
        json!([]),
        json!([-1]),
        json!([1.5]),
        json!([true]),
        json!({"extruder": 1}),
        json!(true),
        json!("bad"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "physical_extruder_map": value
        }))
        .unwrap();
        let err = options.physical_extruder_id_for_logical(0).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("physical_extruder_map"), "{err}");
    }
}

async fn physical_extruder_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn physical_lines<'a>(output: &'a str, prefix: &str) -> Vec<&'a str> {
    output
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

fn assert_line_after(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();
    assert!(
        second_index < first_index,
        "{second_index} !< {first_index}"
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
