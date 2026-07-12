use super::*;

#[tokio::test]
async fn klipper_flavor_reaches_writer_preamble() {
    let output = slice_flavor_output(json!({
        "gcode_flavor": "klipper"
    }))
    .await;

    assert!(output.lines().any(|line| line == "G90"));
    assert!(output.lines().any(|line| line == "G21"));
    assert!(
        output
            .lines()
            .any(|line| line == "M83 ; use relative distances for extrusion")
    );
    assert!(!output.lines().any(|line| line == "G92 E0"));
}

#[tokio::test]
async fn marlin2_absolute_e_keeps_reset() {
    let output = slice_flavor_output(json!({
        "gcode_flavor": "marlin2",
        "use_relative_e_distances": false
    }))
    .await;

    assert_preamble_order(&output);
}

#[tokio::test]
async fn inactive_flavor_is_rejected_before_output() {
    let err = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "gcode_flavor": "makerware"
        }))
        .unwrap(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("invalid value makerware"));
}

async fn slice_flavor_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        extra,
    );
    String::from_utf8(slice(square_pyramid_ascii_stl(), options).await.unwrap()).unwrap()
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
