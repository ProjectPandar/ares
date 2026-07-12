use super::*;

#[tokio::test]
async fn filament_z_shrinkage_reaches_fixed_height_layer_change_and_travel_z_gcode() {
    let output = filament_z_shrinkage_output(json!({
        "filament_shrinkage_compensation_z": "80%",
        "gcode_comments": true
    }))
    .await
    .unwrap();

    assert!(output.contains(";LAYER_CHANGE\n;LAYER:5\n;Z:1.2\nG1 Z1.2 F7200 ; move to layer Z"));
    assert!(output.contains("; layer_count = 6"));
}

#[tokio::test]
async fn precise_filament_z_shrinkage_reaches_compensated_top_z_gcode() {
    let output = filament_z_shrinkage_output(json!({
        "filament_shrinkage_compensation_z": "80%",
        "precise_z_height": true,
        "gcode_comments": true
    }))
    .await
    .unwrap();

    assert!(output.contains(";LAYER_CHANGE\n;LAYER:5\n;Z:1.25\nG1 Z1.25 F7200 ; move to layer Z"));
    assert!(output.contains("; layer_count = 6"));
}

#[tokio::test]
async fn invalid_filament_z_shrinkage_reaches_slice_error_with_key() {
    let err = filament_z_shrinkage_output(json!({
        "filament_shrinkage_compensation_z": 151
    }))
    .await
    .unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("filament_shrinkage_compensation_z"))
    );
}

async fn filament_z_shrinkage_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        extra,
    );
    slice(one_mm_tall_ascii_stl(), options)
        .await
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

fn one_mm_tall_ascii_stl() -> Vec<u8> {
    [
        "solid one_mm",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 1",
        "vertex 0 1 1",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 -1 1",
        "vertex 1 0 1",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex -1 0 1",
        "vertex 0 -1 1",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 1 1",
        "vertex -1 0 1",
        "endloop",
        "endfacet",
        "endsolid one_mm",
    ]
    .join("\n")
    .into_bytes()
}
