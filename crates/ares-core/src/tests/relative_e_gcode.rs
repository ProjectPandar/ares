use super::*;

#[tokio::test]
async fn default_slice_uses_relative_e_axis_without_reset() {
    let output = relative_e_output(json!({})).await;

    assert!(
        output
            .lines()
            .any(|line| line == "M83 ; use relative distances for extrusion")
    );
    assert!(!output.lines().any(|line| line == "G92 E0"));
    assert!(extrusion_lines(&output).contains(&"G1 X2.5 Y-2.5 E0.16924"));
    assert!(extrusion_lines(&output).contains(&"G1 X-0.5 Y0 E0.02393"));
    assert!(
        !extrusion_lines(&output)
            .iter()
            .any(|line| line.contains("E0.77272"))
    );
}

#[tokio::test]
async fn explicit_absolute_e_axis_emits_m82_reset_and_cumulative_e() {
    let output = relative_e_output(json!({ "use_relative_e_distances": false })).await;

    assert_preamble_order(&output);
    assert!(extrusion_lines(&output).contains(&"G1 X2.5 Y-2.5 E0.16924"));
    assert!(extrusion_lines(&output).contains(&"G1 X-0.5 Y0 E0.77272"));
}

#[tokio::test]
async fn relative_e_axis_rejects_non_boolean_values() {
    let err = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({ "use_relative_e_distances": "true" })).unwrap(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("use_relative_e_distances must be a boolean")
    );
}

async fn relative_e_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "seam_gap": 0
        }),
        extra,
    );
    String::from_utf8(slice(square_pyramid_ascii_stl(), options).await.unwrap()).unwrap()
}

fn extrusion_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("G1 X") && line.contains(" E"))
        .collect()
}

fn assert_preamble_order(output: &str) {
    let lines: Vec<_> = output.lines().collect();
    let m82 = lines
        .iter()
        .position(|line| *line == "M82 ; use absolute distances for extrusion")
        .unwrap();
    let reset = lines.iter().position(|line| *line == "G92 E0").unwrap();

    assert_eq!(reset, m82 + 1);
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
