use super::{assert_delta_eq, first_extrusion_delta, options, output_for_layer};
use serde_json::json;

#[test]
fn support_line_width_changes_support_interface_extrusion_delta() {
    let narrow = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.3,
        "support_interface_speed": 37,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let wide = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.6,
        "support_interface_speed": 37,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let narrow_output = output_for_layer(&narrow, 1);
    let wide_output = output_for_layer(&wide, 1);

    assert!(narrow_output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(wide_output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(first_extrusion_delta(&wide_output) > first_extrusion_delta(&narrow_output));
}

#[test]
fn initial_layer_line_width_overrides_support_line_width_for_support_interface() {
    let first_layer = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.3,
        "initial_layer_line_width": 0.6,
        "slow_down_for_layer_cooling": false
    }));
    let same_width_non_first_layer = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.6,
        "slow_down_for_layer_cooling": false
    }));

    assert_delta_eq(
        first_extrusion_delta(&output_for_layer(&first_layer, 0)),
        first_extrusion_delta(&output_for_layer(&same_width_non_first_layer, 1)),
    );
}
