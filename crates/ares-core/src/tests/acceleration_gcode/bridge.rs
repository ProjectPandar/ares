use super::*;

#[test]
fn bridge_acceleration_applies_to_bridge_roles() {
    let bridge_output = bridge_acceleration_output(
        PrintPathRole::Bridge,
        1,
        json!({
            "default_acceleration": 700,
            "initial_layer_acceleration": 0,
            "outer_wall_acceleration": 400,
            "bridge_acceleration": 250
        }),
    );
    let internal_bridge_output = bridge_acceleration_output(
        PrintPathRole::InternalBridge,
        1,
        json!({
            "default_acceleration": 700,
            "initial_layer_acceleration": 0,
            "outer_wall_acceleration": 400,
            "bridge_acceleration": 260
        }),
    );

    assert_acceleration_before_move(&bridge_output, "M204 S250", ";MOVE:print:bridge:1,0");
    assert_acceleration_before_move(
        &internal_bridge_output,
        "M204 S260",
        ";MOVE:print:internal_bridge:1,0",
    );
}

#[test]
fn bridge_acceleration_defaults_to_half_outer_wall_acceleration() {
    let output = bridge_acceleration_output(
        PrintPathRole::Bridge,
        1,
        json!({
            "default_acceleration": 900,
            "initial_layer_acceleration": 0,
            "outer_wall_acceleration": 420
        }),
    );

    assert_acceleration_before_move(&output, "M204 S210", ";MOVE:print:bridge:1,0");
}

#[test]
fn percent_bridge_acceleration_uses_outer_wall_base() {
    let output = bridge_acceleration_output(
        PrintPathRole::Bridge,
        1,
        json!({
            "default_acceleration": 900,
            "initial_layer_acceleration": 0,
            "outer_wall_acceleration": 400,
            "bridge_acceleration": "25%"
        }),
    );

    assert_acceleration_before_move(&output, "M204 S100", ";MOVE:print:bridge:1,0");
}

#[test]
fn initial_layer_acceleration_overrides_bridge_acceleration() {
    let output = bridge_acceleration_output(
        PrintPathRole::Bridge,
        0,
        json!({
            "default_acceleration": 700,
            "initial_layer_acceleration": 300,
            "outer_wall_acceleration": 400,
            "bridge_acceleration": 250
        }),
    );

    assert_acceleration_before_move(&output, "M204 S300", ";MOVE:print:bridge:1,0");
}

#[test]
fn zero_bridge_acceleration_falls_back_to_default_print_acceleration() {
    let output = bridge_acceleration_output(
        PrintPathRole::Bridge,
        1,
        json!({
            "default_acceleration": 700,
            "initial_layer_acceleration": 0,
            "outer_wall_acceleration": 400,
            "bridge_acceleration": 0
        }),
    );

    assert_acceleration_before_move(&output, "M204 S700", ";MOVE:print:bridge:1,0");
}

fn bridge_acceleration_output(
    role: PrintPathRole,
    layer_id: usize,
    extra: serde_json::Value,
) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "line_width": 0.4,
            "filament_diameter": [2.0],
            "bridge_speed": 20,
            "internal_bridge_speed": 20,
            "bridge_flow": 1.0,
            "internal_bridge_flow": 1.0
        }),
        extra,
    );
    let pipeline = crate::pipeline::test_support::single_path_pipeline(&options, role, layer_id);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}
