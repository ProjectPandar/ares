use super::*;

#[tokio::test]
async fn default_marlin_legacy_slice_emits_machine_limits_after_preamble() {
    let output = slice_machine_limits_output(json!({})).await.unwrap();

    assert_line_before(
        &output,
        "M83 ; use relative distances for extrusion",
        "M201 X1000 Y1000 Z500 E5000",
    );
    assert_line_before(
        &output,
        "M201 X1000 Y1000 Z500 E5000",
        "M203 X500 Y500 Z12 E120",
    );
    assert_line_before(&output, "M203 X500 Y500 Z12 E120", "M204 P1500 R1500 T1500");
    assert_line_before(
        &output,
        "M204 P1500 R1500 T1500",
        "M205 X10.00 Y10.00 Z0.20 E2.50 ; sets the jerk limits, mm/sec",
    );
    assert_line_before(
        &output,
        "M205 X10.00 Y10.00 Z0.20 E2.50 ; sets the jerk limits, mm/sec",
        "M190 S35 ; set bed temperature and wait for it to be reached",
    );
}

#[tokio::test]
async fn marlin_legacy_uses_custom_first_vector_values() {
    let output = slice_machine_limits_output(custom_limit_options(json!({
        "gcode_flavor": "marlin"
    })))
    .await
    .unwrap();

    assert!(output.contains("M201 X111 Y223 Z334 E444\n"));
    assert!(output.contains("M203 X55 Y67 Z8 E88\n"));
    assert!(output.contains("M204 P901 R803 T901\n"));
    assert!(output.contains("M205 X9.10 Y8.20 Z0.33 E4.40 ; sets the jerk limits, mm/sec\n"));
    assert!(!output.contains("M205 J0.025"));
}

#[tokio::test]
async fn marlin_firmware_emits_marlin2_acceleration_and_junction_deviation() {
    let output = slice_machine_limits_output(custom_limit_options(json!({
        "gcode_flavor": "marlin2"
    })))
    .await
    .unwrap();

    assert!(output.contains(
        "M204 P901 R803 T704 ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n"
    ));
    assert!(output.contains("M205 J0.025\n"));
}

#[tokio::test]
async fn reprap_firmware_scales_speed_and_jerk_to_mm_per_minute() {
    let output = slice_machine_limits_output(custom_limit_options(json!({
        "gcode_flavor": "reprapfirmware"
    })))
    .await
    .unwrap();

    assert!(output.contains("M203 X3324 Y3990 Z456 E5304\n"));
    assert!(output.contains("M204 P901 T704 ; sets acceleration (P, T), mm/sec^2\n"));
    assert!(
        output.contains("M566 X546.00 Y492.00 Z19.80 E264.00 ; sets the jerk limits, mm/min\n")
    );
}

#[tokio::test]
async fn disabled_or_unsupported_flavors_skip_machine_limits() {
    let disabled = slice_machine_limits_output(json!({
        "emit_machine_limits_to_gcode": false
    }))
    .await
    .unwrap();
    let klipper = slice_machine_limits_output(json!({
        "gcode_flavor": "klipper"
    }))
    .await
    .unwrap();
    let repetier = slice_machine_limits_output(json!({
        "gcode_flavor": "repetier"
    }))
    .await
    .unwrap();

    for output in [disabled, klipper, repetier] {
        assert!(!output.contains("M201 X1000 Y1000 Z500 E5000"));
        assert!(!output.contains("M203 X500 Y500 Z12 E120"));
        assert!(!output.contains("M204 P1500 R1500 T1500"));
        assert!(!output.contains("M205 X10.00 Y10.00 Z0.20 E2.50"));
        assert!(!output.contains("M566 X"));
    }
}

#[tokio::test]
async fn invalid_machine_limit_rejects_slice_before_bytes() {
    let err = slice(
        square_pyramid_ascii_stl(),
        merged_options(
            base_options(),
            json!({
                "machine_max_speed_x": "fast"
            }),
        ),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("machine_max_speed_x"))
    );
}

async fn slice_machine_limits_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let output = slice(
        square_pyramid_ascii_stl(),
        merged_options(base_options(), extra),
    )
    .await?;
    Ok(String::from_utf8(output).unwrap())
}

fn custom_limit_options(extra: serde_json::Value) -> serde_json::Value {
    merged_json(
        json!({
            "machine_max_acceleration_x": [111.4, 222.0],
            "machine_max_acceleration_y": "222.5;333.0",
            "machine_max_acceleration_z": "333.6",
            "machine_max_acceleration_e": 444.4,
            "machine_max_speed_x": "55.4,99.0",
            "machine_max_speed_y": [66.5, 11.0],
            "machine_max_speed_z": "7.6",
            "machine_max_speed_e": 88.4,
            "machine_max_acceleration_extruding": [901.2, 1.0],
            "machine_max_acceleration_retracting": "802.5;1",
            "machine_max_acceleration_travel": 703.6,
            "machine_max_jerk_x": [9.1, 1.0],
            "machine_max_jerk_y": "8.2;1",
            "machine_max_jerk_z": "0.33",
            "machine_max_jerk_e": 4.4,
            "machine_max_junction_deviation": [0.025, 0.3]
        }),
        extra,
    )
}

fn base_options() -> serde_json::Value {
    json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0
    })
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
    serde_json::from_value(merged_json(base, extra)).unwrap()
}

fn merged_json(base: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::Value::Object(base)
}
