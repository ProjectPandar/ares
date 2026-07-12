use super::*;

#[tokio::test]
async fn support_filament_type_reaches_gcode_header_display_name() {
    let output = filament_header_output(json!({
        "filament_type": ["PLA"],
        "filament_vendor": ["Orca"],
        "filament_is_support": [true],
        "filament_id": ["GFS00"]
    }))
    .await;

    assert!(output.lines().any(|line| line == "; filament_type = PLA"));
    assert!(
        output
            .lines()
            .any(|line| line == "; filament_vendor = Orca")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; filament_display_type = Sup.PLA")
    );
}

#[tokio::test]
async fn non_support_filament_type_reaches_gcode_header_raw_display_name() {
    let output = filament_header_output(json!({
        "filament_type": ["PETG"],
        "filament_vendor": ["Generic"],
        "filament_is_support": [false]
    }))
    .await;

    assert!(output.lines().any(|line| line == "; filament_type = PETG"));
    assert!(
        output
            .lines()
            .any(|line| line == "; filament_vendor = Generic")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; filament_display_type = PETG")
    );
}

#[tokio::test]
async fn invalid_filament_header_options_reach_slice_error() {
    for (key, extra) in [
        ("filament_type", json!({ "filament_type": [7] })),
        ("filament_vendor", json!({ "filament_vendor": [7] })),
        (
            "filament_is_support",
            json!({
                "filament_type": ["PLA"],
                "filament_vendor": ["Orca"],
                "filament_is_support": ["true"]
            }),
        ),
    ] {
        let err = filament_header_error(extra).await;

        assert!(matches!(err, SliceError::InvalidInput(_)), "{key}");
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
}

async fn filament_header_output(extra: serde_json::Value) -> String {
    let mut options = serde_json::Map::new();
    options.insert("sparse_infill_density".to_owned(), json!(0));
    options.insert("filament_max_volumetric_speed".to_owned(), json!(0.0));
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }

    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(serde_json::Value::Object(options)).unwrap(),
    )
    .await
    .unwrap();

    String::from_utf8(output).unwrap()
}

async fn filament_header_error(extra: serde_json::Value) -> SliceError {
    let mut options = serde_json::Map::new();
    options.insert("sparse_infill_density".to_owned(), json!(0));
    options.insert("filament_max_volumetric_speed".to_owned(), json!(0.0));
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }

    slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(serde_json::Value::Object(options)).unwrap(),
    )
    .await
    .unwrap_err()
}
