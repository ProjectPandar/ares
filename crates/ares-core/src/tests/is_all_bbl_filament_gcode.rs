use super::*;

#[tokio::test]
async fn machine_start_is_all_bbl_filament_renders_true_for_bambu_vendor() {
    let output = slice_is_all_bbl_filament_output(json!({
        "machine_start_gcode": ";BBL [is_all_bbl_filament]",
        "filament_vendor": ["Bambu Lab"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BBL 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_is_all_bbl_filament_requires_every_vendor_to_be_bambu() {
    let output = slice_is_all_bbl_filament_output(json!({
        "machine_start_gcode": ";BBL [is_all_bbl_filament]",
        "filament_vendor": ["Bambu Lab", "Generic"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BBL 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_is_all_bbl_filament_defaults_to_false() {
    let output = slice_is_all_bbl_filament_output(json!({
        "machine_start_gcode": ";BBL [is_all_bbl_filament]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BBL 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn is_all_bbl_filament_stays_literal_in_layer_change_scope() {
    let output = slice_is_all_bbl_filament_output(json!({
        "layer_change_gcode": ";LC [is_all_bbl_filament] [layer_num]",
        "filament_vendor": ["Bambu Lab"]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [is_all_bbl_filament] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_is_all_bbl_filament_rejects_invalid_vendor_vector() {
    for value in [json!("Bambu Lab"), json!([]), json!(["Bambu Lab", 7])] {
        let err = slice_is_all_bbl_filament_output(json!({
            "machine_start_gcode": ";BBL [is_all_bbl_filament]",
            "filament_vendor": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_vendor"));
    }
}

async fn slice_is_all_bbl_filament_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
