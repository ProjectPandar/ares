use super::*;

#[tokio::test]
async fn idle_standby_startup_temperature_uses_idle_and_signed_delta_for_multitool_startup() {
    let output = slice_idle_standby_output(json!({
        "nozzle_diameter": [0.4, 0.6, 0.8],
        "nozzle_temperature_initial_layer": [210, 230, 240],
        "ooze_prevention": true,
        "idle_temperature": [0, 180, 0],
        "standby_temperature_delta": -10
    }))
    .await
    .unwrap();

    assert_eq!(
        startup_temperature_lines(&output),
        vec![
            "M104 S210 T0 ; set nozzle temperature",
            "M104 S180 T1 ; set nozzle temperature",
            "M104 S230 T2 ; set nozzle temperature",
        ]
    );
}

#[tokio::test]
async fn idle_standby_startup_temperature_default_disabled_uses_first_layer_temperatures() {
    let output = slice_idle_standby_output(json!({
        "nozzle_diameter": [0.4, 0.6, 0.8],
        "nozzle_temperature_initial_layer": [210, 230, 240]
    }))
    .await
    .unwrap();

    assert_eq!(
        startup_temperature_lines(&output),
        vec![
            "M104 S210 T0 ; set nozzle temperature",
            "M104 S230 T1 ; set nozzle temperature",
            "M104 S240 T2 ; set nozzle temperature",
        ]
    );
}

#[tokio::test]
async fn idle_standby_startup_temperature_formats_reprap_tool_axes() {
    let output = slice_idle_standby_output(json!({
        "gcode_flavor": "reprapfirmware",
        "nozzle_diameter": [0.4, 0.6],
        "nozzle_temperature_initial_layer": [210, 230],
        "ooze_prevention": true,
        "idle_temperature": [0, 0],
        "standby_temperature_delta": -15
    }))
    .await
    .unwrap();

    assert_eq!(
        startup_temperature_lines(&output),
        vec![
            "G10 S210 P0 ; set nozzle temperature",
            "G10 S215 P1 ; set nozzle temperature",
        ]
    );
}

#[tokio::test]
async fn idle_standby_startup_temperature_short_vectors_use_orca_first_value_fallback() {
    let output = slice_idle_standby_output(json!({
        "nozzle_diameter": [0.4, 0.6, 0.8],
        "nozzle_temperature_initial_layer": [210, 230],
        "ooze_prevention": true,
        "idle_temperature": [0, 180],
        "standby_temperature_delta": -10
    }))
    .await
    .unwrap();

    assert_eq!(
        startup_temperature_lines(&output),
        vec![
            "M104 S210 T0 ; set nozzle temperature",
            "M104 S180 T1 ; set nozzle temperature",
            "M104 S200 T2 ; set nozzle temperature",
        ]
    );
}

#[tokio::test]
async fn idle_standby_startup_temperature_non_positive_inactive_temperature_suppresses_tool_command()
 {
    let output = slice_idle_standby_output(json!({
        "nozzle_diameter": [0.4, 0.6],
        "nozzle_temperature_initial_layer": [5, 5],
        "ooze_prevention": true,
        "idle_temperature": [0, 0],
        "standby_temperature_delta": -10
    }))
    .await
    .unwrap();

    assert_eq!(
        startup_temperature_lines(&output),
        vec!["M104 S5 T0 ; set nozzle temperature"]
    );
}

#[tokio::test]
async fn idle_standby_startup_temperature_custom_nozzle_gcode_suppresses_automatic_multitool_commands()
 {
    let output = slice_idle_standby_output(json!({
        "machine_start_gcode": "M104 S199",
        "nozzle_diameter": [0.4, 0.6],
        "nozzle_temperature_initial_layer": [210, 230],
        "ooze_prevention": true,
        "idle_temperature": [0, 180],
        "standby_temperature_delta": -10
    }))
    .await
    .unwrap();

    assert_eq!(startup_temperature_lines(&output), vec!["M104 S199"]);
}

#[tokio::test]
async fn idle_standby_startup_temperature_rejects_invalid_inputs_before_suppression_or_branching() {
    for (key, value, extra) in [
        (
            "ooze_prevention",
            json!("yes"),
            json!({"machine_start_gcode": "M104 S199"}),
        ),
        (
            "idle_temperature",
            json!([0, "bad"]),
            json!({"ooze_prevention": false}),
        ),
        (
            "standby_temperature_delta",
            json!("bad"),
            json!({"gcode_flavor": "klipper"}),
        ),
        (
            "nozzle_temperature_initial_layer",
            json!("bad"),
            json!({"machine_start_gcode": "M104 S199"}),
        ),
    ] {
        let err = slice_idle_standby_output(option_case(key, value, extra))
            .await
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{err}");
    }
}

async fn slice_idle_standby_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn startup_temperature_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .take_while(|line| *line != ";LAYER_CHANGE")
        .filter(|line| line.starts_with("M104 S") || line.starts_with("G10 S"))
        .collect()
}

fn option_case(key: &str, value: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    let mut extra = extra.as_object().unwrap().clone();
    extra.insert("nozzle_diameter".into(), json!([0.4, 0.6]));
    extra.insert("nozzle_temperature_initial_layer".into(), json!([210, 230]));
    extra.insert(key.into(), value);
    serde_json::Value::Object(extra)
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
