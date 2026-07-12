use super::*;

const WIPE_TOWER_CONFIG_KEYS: [&str; 17] = [
    "wipe_tower_type",
    "purge_in_prime_tower",
    "enable_filament_ramming",
    "tool_change_on_wipe_tower",
    "wipe_tower_no_sparse_layers",
    "support_multi_bed_types",
    "wipe_tower_rotation_angle",
    "wipe_tower_bridging",
    "wipe_tower_extra_flow",
    "wipe_tower_cone_angle",
    "wipe_tower_extra_spacing",
    "wipe_tower_max_purge_speed",
    "wipe_tower_wall_type",
    "wipe_tower_extra_rib_length",
    "wipe_tower_rib_width",
    "wipe_tower_fillet_wall",
    "wipe_tower_filament",
];

#[tokio::test]
async fn wipe_tower_config_values_reach_gcode_config_header() {
    let output = wipe_tower_config_output(json!({
        "wipe_tower_type": "type1",
        "purge_in_prime_tower": true,
        "enable_filament_ramming": true,
        "tool_change_on_wipe_tower": false,
        "wipe_tower_no_sparse_layers": true,
        "support_multi_bed_types": true,
        "wipe_tower_rotation_angle": -15.5,
        "wipe_tower_bridging": -2.25,
        "wipe_tower_extra_flow": 125.0,
        "wipe_tower_cone_angle": 45.0,
        "wipe_tower_extra_spacing": 150.0,
        "wipe_tower_max_purge_speed": 120.0,
        "wipe_tower_wall_type": "rectangle",
        "wipe_tower_extra_rib_length": -12.5,
        "wipe_tower_rib_width": 8.25,
        "wipe_tower_fillet_wall": true,
        "wipe_tower_filament": 3
    }))
    .await
    .unwrap();

    for expected in [
        "; wipe_tower_type = type1",
        "; purge_in_prime_tower = 1",
        "; enable_filament_ramming = 1",
        "; tool_change_on_wipe_tower = 0",
        "; wipe_tower_no_sparse_layers = 1",
        "; support_multi_bed_types = 1",
        "; wipe_tower_rotation_angle = -15.5",
        "; wipe_tower_bridging = -2.25",
        "; wipe_tower_extra_flow = 125",
        "; wipe_tower_cone_angle = 45",
        "; wipe_tower_extra_spacing = 150",
        "; wipe_tower_max_purge_speed = 120",
        "; wipe_tower_wall_type = rectangle",
        "; wipe_tower_extra_rib_length = -12.5",
        "; wipe_tower_rib_width = 8.25",
        "; wipe_tower_fillet_wall = 1",
        "; wipe_tower_filament = 3",
    ] {
        assert!(output.lines().any(|line| line == expected), "{expected}");
    }
}

#[tokio::test]
async fn wipe_tower_config_false_values_reach_gcode_config_header() {
    let output = wipe_tower_config_output(json!({
        "wipe_tower_type": "type2",
        "purge_in_prime_tower": false,
        "enable_filament_ramming": false,
        "tool_change_on_wipe_tower": false,
        "wipe_tower_no_sparse_layers": false,
        "support_multi_bed_types": false,
        "wipe_tower_rotation_angle": 0.0,
        "wipe_tower_bridging": 10.0,
        "wipe_tower_extra_flow": 300.0,
        "wipe_tower_cone_angle": 90.0,
        "wipe_tower_extra_spacing": 100.0,
        "wipe_tower_max_purge_speed": 10.0,
        "wipe_tower_wall_type": "cone",
        "wipe_tower_extra_rib_length": 0.0,
        "wipe_tower_rib_width": 0.0,
        "wipe_tower_fillet_wall": false,
        "wipe_tower_filament": 0
    }))
    .await
    .unwrap();

    for expected in [
        "; wipe_tower_type = type2",
        "; purge_in_prime_tower = 0",
        "; enable_filament_ramming = 0",
        "; tool_change_on_wipe_tower = 0",
        "; wipe_tower_no_sparse_layers = 0",
        "; support_multi_bed_types = 0",
        "; wipe_tower_rotation_angle = 0",
        "; wipe_tower_bridging = 10",
        "; wipe_tower_extra_flow = 300",
        "; wipe_tower_cone_angle = 90",
        "; wipe_tower_extra_spacing = 100",
        "; wipe_tower_max_purge_speed = 10",
        "; wipe_tower_wall_type = cone",
        "; wipe_tower_extra_rib_length = 0",
        "; wipe_tower_rib_width = 0",
        "; wipe_tower_fillet_wall = 0",
        "; wipe_tower_filament = 0",
    ] {
        assert!(output.lines().any(|line| line == expected), "{expected}");
    }
}

#[tokio::test]
async fn wipe_tower_speed_spacing_boundary_values_reach_gcode_config_header() {
    let output = wipe_tower_config_output(json!({
        "wipe_tower_extra_flow": 100.0,
        "wipe_tower_cone_angle": 0.0,
        "wipe_tower_extra_spacing": 300.0,
        "wipe_tower_max_purge_speed": 10.0
    }))
    .await
    .unwrap();

    for expected in [
        "; wipe_tower_extra_flow = 100",
        "; wipe_tower_cone_angle = 0",
        "; wipe_tower_extra_spacing = 300",
        "; wipe_tower_max_purge_speed = 10",
    ] {
        assert!(output.lines().any(|line| line == expected), "{expected}");
    }
}

#[tokio::test]
async fn wipe_tower_config_follows_stamping_in_header_order() {
    let output = wipe_tower_config_output(json!({
        "filament_stamping_distance": [4.0],
        "wipe_tower_type": "type1",
        "purge_in_prime_tower": true,
        "enable_filament_ramming": false,
        "tool_change_on_wipe_tower": true,
        "wipe_tower_no_sparse_layers": true,
        "support_multi_bed_types": false,
        "wipe_tower_rotation_angle": -15.5,
        "wipe_tower_bridging": 10.0,
        "wipe_tower_extra_flow": 125.0,
        "wipe_tower_cone_angle": 45.0,
        "wipe_tower_extra_spacing": 150.0,
        "wipe_tower_max_purge_speed": 120.0,
        "wipe_tower_wall_type": "rib",
        "wipe_tower_extra_rib_length": 11.0,
        "wipe_tower_rib_width": 8.0,
        "wipe_tower_fillet_wall": true,
        "wipe_tower_filament": 2,
        "small_area_infill_flow_compensation_model": ["0,0", "\n0.2,0.4444", "\n10,1"],
        "filament_colour": ["#123456"]
    }))
    .await
    .unwrap();
    let lines = output.lines().collect::<Vec<_>>();
    let stamping_index = lines
        .iter()
        .position(|line| *line == "; filament_stamping_distance = 4")
        .unwrap();

    assert_eq!(
        lines.get(stamping_index + 1),
        Some(&"; wipe_tower_type = type1")
    );
    assert_eq!(
        lines.get(stamping_index + 2),
        Some(&"; purge_in_prime_tower = 1")
    );
    assert_eq!(
        lines.get(stamping_index + 3),
        Some(&"; enable_filament_ramming = 0")
    );
    assert_eq!(
        lines.get(stamping_index + 4),
        Some(&"; tool_change_on_wipe_tower = 1")
    );
    assert_eq!(
        lines.get(stamping_index + 5),
        Some(&"; wipe_tower_no_sparse_layers = 1")
    );
    assert_eq!(
        lines.get(stamping_index + 6),
        Some(&"; support_multi_bed_types = 0")
    );
    assert_eq!(
        lines.get(stamping_index + 7),
        Some(&"; wipe_tower_rotation_angle = -15.5")
    );
    assert_eq!(
        lines.get(stamping_index + 8),
        Some(&"; wipe_tower_bridging = 10")
    );
    assert_eq!(
        lines.get(stamping_index + 9),
        Some(&"; wipe_tower_extra_flow = 125")
    );
    assert_eq!(
        lines.get(stamping_index + 10),
        Some(&"; wipe_tower_cone_angle = 45")
    );
    assert_eq!(
        lines.get(stamping_index + 11),
        Some(&"; wipe_tower_extra_spacing = 150")
    );
    assert_eq!(
        lines.get(stamping_index + 12),
        Some(&"; wipe_tower_max_purge_speed = 120")
    );
    assert_eq!(
        lines.get(stamping_index + 13),
        Some(&"; wipe_tower_wall_type = rib")
    );
    assert_eq!(
        lines.get(stamping_index + 14),
        Some(&"; wipe_tower_extra_rib_length = 11")
    );
    assert_eq!(
        lines.get(stamping_index + 15),
        Some(&"; wipe_tower_rib_width = 8")
    );
    assert_eq!(
        lines.get(stamping_index + 16),
        Some(&"; wipe_tower_fillet_wall = 1")
    );
    assert_eq!(
        lines.get(stamping_index + 17),
        Some(&"; wipe_tower_filament = 2")
    );
    assert_eq!(
        lines.get(stamping_index + 18),
        Some(&"; small_area_infill_flow_compensation_model = 0,0;0.2,0.4444;10,1")
    );
    assert_eq!(
        lines.get(stamping_index + 19),
        Some(&"; filament_colour = #123456")
    );
}

#[tokio::test]
async fn absent_wipe_tower_config_values_preserve_header() {
    let output = wipe_tower_config_output(json!({})).await.unwrap();

    for key in WIPE_TOWER_CONFIG_KEYS {
        assert!(
            !output
                .lines()
                .any(|line| line.starts_with(&format!("; {key} = "))),
            "{key}"
        );
    }
    assert!(output.lines().any(|line| line == "; generated by Ares"));
}

#[tokio::test]
async fn invalid_wipe_tower_config_values_reach_slice_error() {
    for (key, value) in [
        ("wipe_tower_type", json!("type3")),
        ("wipe_tower_type", json!(1)),
        ("wipe_tower_type", json!(["type1"])),
        ("wipe_tower_type", serde_json::Value::Null),
        ("purge_in_prime_tower", json!(1)),
        ("purge_in_prime_tower", json!("true")),
        ("purge_in_prime_tower", json!([true])),
        ("enable_filament_ramming", json!(0)),
        ("enable_filament_ramming", json!("false")),
        ("tool_change_on_wipe_tower", json!(["false"])),
        ("tool_change_on_wipe_tower", serde_json::Value::Null),
        ("wipe_tower_no_sparse_layers", json!(0)),
        ("wipe_tower_no_sparse_layers", json!("false")),
        ("wipe_tower_no_sparse_layers", json!([false])),
        ("wipe_tower_no_sparse_layers", serde_json::Value::Null),
        ("support_multi_bed_types", json!({"value": true})),
        ("support_multi_bed_types", json!("0")),
        ("wipe_tower_rotation_angle", json!("0")),
        ("wipe_tower_rotation_angle", json!([0.0])),
        ("wipe_tower_rotation_angle", serde_json::Value::Null),
        ("wipe_tower_bridging", json!("10")),
        ("wipe_tower_bridging", json!([10.0])),
        ("wipe_tower_bridging", serde_json::Value::Null),
        ("wipe_tower_extra_flow", json!(99.999)),
        ("wipe_tower_extra_flow", json!(300.001)),
        ("wipe_tower_extra_flow", json!("125")),
        ("wipe_tower_extra_flow", json!([125.0])),
        ("wipe_tower_extra_flow", serde_json::Value::Null),
        ("wipe_tower_cone_angle", json!(-0.001)),
        ("wipe_tower_cone_angle", json!(90.001)),
        ("wipe_tower_cone_angle", json!("45")),
        ("wipe_tower_cone_angle", json!([45.0])),
        ("wipe_tower_cone_angle", serde_json::Value::Null),
        ("wipe_tower_extra_spacing", json!(99.999)),
        ("wipe_tower_extra_spacing", json!(300.001)),
        ("wipe_tower_extra_spacing", json!("150")),
        ("wipe_tower_extra_spacing", json!([150.0])),
        ("wipe_tower_extra_spacing", serde_json::Value::Null),
        ("wipe_tower_max_purge_speed", json!(9.999)),
        ("wipe_tower_max_purge_speed", json!("90")),
        ("wipe_tower_max_purge_speed", json!([90.0])),
        ("wipe_tower_max_purge_speed", serde_json::Value::Null),
        ("wipe_tower_wall_type", json!("square")),
        ("wipe_tower_wall_type", json!(1)),
        ("wipe_tower_wall_type", json!(["rib"])),
        ("wipe_tower_wall_type", serde_json::Value::Null),
        ("wipe_tower_extra_rib_length", json!(300.001)),
        ("wipe_tower_extra_rib_length", json!("12.5")),
        ("wipe_tower_extra_rib_length", json!([0.0])),
        ("wipe_tower_extra_rib_length", serde_json::Value::Null),
        ("wipe_tower_rib_width", json!(-0.001)),
        ("wipe_tower_rib_width", json!(300.001)),
        ("wipe_tower_rib_width", json!("8")),
        ("wipe_tower_rib_width", json!([8.0])),
        ("wipe_tower_rib_width", serde_json::Value::Null),
        ("wipe_tower_fillet_wall", json!(1)),
        ("wipe_tower_fillet_wall", json!("true")),
        ("wipe_tower_fillet_wall", json!([true])),
        ("wipe_tower_fillet_wall", serde_json::Value::Null),
        ("wipe_tower_filament", json!(-1)),
        ("wipe_tower_filament", json!(1.25)),
        ("wipe_tower_filament", json!(2147483648_i64)),
        ("wipe_tower_filament", json!("3")),
        ("wipe_tower_filament", json!([3])),
        ("wipe_tower_filament", serde_json::Value::Null),
    ] {
        let err = wipe_tower_config_output(json!({
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[tokio::test]
async fn invalid_wipe_tower_config_is_rejected_when_header_is_skipped() {
    let err = wipe_tower_config_output(json!({
        "thumbnails": "7x8/BTT_TFT",
        "wipe_tower_extra_spacing": 99.999
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("wipe_tower_extra_spacing"));
}

async fn wipe_tower_config_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
    .map(|bytes| String::from_utf8(bytes).unwrap())
}
