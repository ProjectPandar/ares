use super::*;

#[tokio::test]
async fn machine_start_max_print_z_renders_ceiled_planned_layer_z() {
    let output = slice_tall_max_print_z_output(json!({
        "machine_start_gcode": ";MAXZ [max_print_z]",
        "layer_height": 0.4,
        "initial_layer_print_height": 0.4
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MAXZ 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn max_print_z_composes_with_machine_start_height_and_layer_count() {
    let output = slice_max_print_z_output(json!({
        "machine_start_gcode": ";START [max_print_z] [max_print_height] [total_layer_count]",
        "printable_height": 256
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 1 256 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn max_print_z_stays_literal_in_layer_change_scope() {
    let output = slice_max_print_z_output(json!({
        "layer_change_gcode": ";LC [max_print_z] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [max_print_z] 1", "; segment_count = 4");
}

async fn slice_max_print_z_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_tall_max_print_z_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "sparse_infill_density": 0
        }),
        extra,
    );
    slice(tall_pyramid_ascii_stl(), options)
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

fn tall_pyramid_ascii_stl() -> Vec<u8> {
    [
        "solid pyramid",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 1.2",
        "vertex 0 1 1.2",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 -1 1.2",
        "vertex 1 0 1.2",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex -1 0 1.2",
        "vertex 0 -1 1.2",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 1 1.2",
        "vertex -1 0 1.2",
        "endloop",
        "endfacet",
        "endsolid pyramid",
    ]
    .join("\n")
    .into_bytes()
}
