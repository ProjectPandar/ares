use super::*;

#[tokio::test]
async fn machine_start_gcode_renders_adaptive_bed_mesh_placeholders() {
    let output = slice_adaptive_bed_mesh_output(json!({
        "machine_start_gcode": ";MESH MIN=[adaptive_bed_mesh_min] X=[adaptive_bed_mesh_min_0] Y=[adaptive_bed_mesh_min_1]\n;MESH MAX=[adaptive_bed_mesh_max]\n;PROBE=[bed_mesh_probe_count] ALGO=[bed_mesh_algo]",
        "bed_mesh_min": "-99999x-99999",
        "bed_mesh_max": "99999x99999",
        "bed_mesh_probe_distance": "50x50"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";MESH MIN=-2.5,-2.5 X=-2.5 Y=-2.5",
        ";LAYER_CHANGE",
    );
    assert_line_before(&output, ";MESH MAX=2.5,2.5", ";LAYER_CHANGE");
    assert_line_before(&output, ";PROBE=3,3 ALGO=bicubic", ";LAYER_CHANGE");
}

#[tokio::test]
async fn adaptive_bed_mesh_margin_expands_first_layer_bounds() {
    let output = slice_adaptive_bed_mesh_output(json!({
        "machine_start_gcode": ";MESH [adaptive_bed_mesh_min] [adaptive_bed_mesh_max]",
        "adaptive_bed_mesh_margin": 2.5
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MESH -5,-5 5,5", ";LAYER_CHANGE");
}

#[tokio::test]
async fn adaptive_bed_mesh_min_and_max_clamp_expanded_bounds() {
    let output = slice_adaptive_bed_mesh_output(json!({
        "machine_start_gcode": ";MESH [adaptive_bed_mesh_min] [adaptive_bed_mesh_max]",
        "bed_mesh_min": [-0.25, -0.5],
        "bed_mesh_max": [[0.5, 0.25]],
        "adaptive_bed_mesh_margin": 2.5
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MESH -0.25,-0.5 0.5,0.25", ";LAYER_CHANGE");
}

#[tokio::test]
async fn klipper_bicubic_adaptive_bed_mesh_uses_four_probe_points_per_axis() {
    let output = slice_adaptive_bed_mesh_output(json!({
        "gcode_flavor": "klipper",
        "machine_start_gcode": ";PROBE [bed_mesh_probe_count] [bed_mesh_algo]",
        "bed_mesh_min": "-99999x-99999",
        "bed_mesh_max": "99999x99999",
        "bed_mesh_probe_distance": "50x50"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";PROBE 4,4 bicubic", ";LAYER_CHANGE");
}

#[tokio::test]
async fn adaptive_bed_mesh_rendering_keeps_startup_suppression_consistent() {
    let output = slice_adaptive_bed_mesh_output(json!({
        "machine_start_gcode": "M1[adaptive_bed_mesh_max_1]0 S60 ; bed\nM1[adaptive_bed_mesh_min_0]4 S210 ; nozzle\nM1[adaptive_bed_mesh_max_1]1 S45 ; chamber",
        "bed_mesh_min": [0.0, -99999.0],
        "adaptive_bed_mesh_margin": 1.5,
        "bed_temperature_initial_layer": [60],
        "nozzle_temperature_initial_layer": [210],
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .await
    .unwrap();

    assert_line_before(&output, "M140 S60 ; bed", "M104 S210 ; nozzle");
    assert_line_before(&output, "M104 S210 ; nozzle", "M141 S45 ; chamber");
    assert_line_before(&output, "M141 S45 ; chamber", ";LAYER_CHANGE");
    assert!(
        !output
            .lines()
            .any(|line| line == "M190 S60 ; set bed temperature and wait for it to be reached")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "M104 S210 ; set nozzle temperature")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "M191 S45 ;set chamber_temperature and wait for it to be reached")
    );
}

#[tokio::test]
async fn malformed_adaptive_bed_mesh_options_reach_slice_error() {
    for (key, value) in [
        ("bed_mesh_min", json!("bad")),
        ("bed_mesh_max", json!([0.0])),
        ("bed_mesh_probe_distance", json!({"x": 50.0, "y": 50.0})),
        ("adaptive_bed_mesh_margin", json!(-1.0)),
    ] {
        let mut options = json!({
            "machine_start_gcode": ";MESH [adaptive_bed_mesh_min]"
        });
        options[key] = value;
        let err = slice_adaptive_bed_mesh_output(options).await.unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{err}");
    }
}

async fn slice_adaptive_bed_mesh_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
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
