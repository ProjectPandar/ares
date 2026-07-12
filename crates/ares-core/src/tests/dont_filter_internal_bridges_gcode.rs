use super::*;

#[tokio::test]
async fn invalid_dont_filter_internal_bridges_values_reach_slice_error_before_gcode() {
    for value in [json!("unknown"), json!(true)] {
        let err = internal_bridge_filter_output(json!({
            "dont_filter_internal_bridges": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("dont_filter_internal_bridges"));
    }
}

async fn internal_bridge_filter_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let mut options = serde_json::Map::new();
    options.insert("sparse_infill_density".to_owned(), json!(100));
    options.insert("internal_bridge_density".to_owned(), json!(50));
    options.insert("minimum_sparse_infill_area".to_owned(), json!(0));
    options.insert("wall_loops".to_owned(), json!(0));
    options.insert("filament_max_volumetric_speed".to_owned(), json!(0.0));
    options.insert("slow_down_for_layer_cooling".to_owned(), json!(false));
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }

    slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(serde_json::Value::Object(options)).unwrap(),
    )
    .await
    .map(|bytes| String::from_utf8(bytes).unwrap())
}
