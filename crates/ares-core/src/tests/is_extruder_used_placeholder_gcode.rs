use super::*;

#[tokio::test]
async fn machine_start_is_extruder_used_renders_orca_sized_default_vector() {
    let output = slice_is_extruder_used_output(json!({
        "machine_start_gcode": ";USED [is_extruder_used]"
    }))
    .await
    .unwrap();

    let used = rendered_vector_after_prefix(&output, ";USED ");

    assert_eq!(used.len(), 64);
    assert_eq!(used[0], "1");
    assert!(used[1..].iter().all(|value| *value == "0"));
}

#[tokio::test]
async fn machine_start_is_extruder_used_extends_to_filament_count() {
    let filament_diameter = (0..65)
        .map(|_| serde_json::Value::String("1.75".to_owned()))
        .collect::<Vec<_>>();
    let output = slice_is_extruder_used_output(json!({
        "machine_start_gcode": ";USED [is_extruder_used]",
        "filament_diameter": filament_diameter
    }))
    .await
    .unwrap();

    let used = rendered_vector_after_prefix(&output, ";USED ");

    assert_eq!(used.len(), 65);
    assert_eq!(used[0], "1");
    assert!(used[1..].iter().all(|value| *value == "0"));
}

#[tokio::test]
async fn is_extruder_used_composes_with_num_extruders() {
    let output = slice_is_extruder_used_output(json!({
        "machine_start_gcode": ";START [num_extruders] [is_extruder_used]",
        "nozzle_diameter": ["0.4", "0.6"],
        "filament_diameter": ["1.75", "1.75"]
    }))
    .await
    .unwrap();

    let line = line_with_prefix(&output, ";START ");
    let rendered = line.strip_prefix(";START 2 ").unwrap();
    let used = rendered.split(',').collect::<Vec<_>>();

    assert_eq!(used.len(), 64);
    assert_eq!(used[0], "1");
    assert!(used[1..].iter().all(|value| *value == "0"));
}

#[tokio::test]
async fn is_extruder_used_stays_literal_in_layer_change_scope() {
    let output = slice_is_extruder_used_output(json!({
        "layer_change_gcode": ";LC [is_extruder_used] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [is_extruder_used] 1", "; segment_count = 4");
}

async fn slice_is_extruder_used_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn rendered_vector_after_prefix<'a>(output: &'a str, prefix: &str) -> Vec<&'a str> {
    line_with_prefix(output, prefix)
        .strip_prefix(prefix)
        .unwrap()
        .split(',')
        .collect()
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
