use super::{
    assert_delta_eq, assert_invalid_option, first_extrusion_delta, options, output_for_layer,
};
use serde_json::{Value, json};

#[test]
fn support_line_width_changes_support_material_extrusion_delta() {
    let narrow = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.3,
        "support_speed": 37,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let wide = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.6,
        "support_speed": 37,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let narrow_output = output_for_layer(&narrow, 1);
    let wide_output = output_for_layer(&wide, 1);

    assert!(narrow_output.contains(";SPEED:print:support_material:1,0:2220"));
    assert!(wide_output.contains(";SPEED:print:support_material:1,0:2220"));
    assert!(first_extrusion_delta(&wide_output) > first_extrusion_delta(&narrow_output));
}

#[test]
fn support_line_width_percent_matches_absolute_support_material_width() {
    let percent = options(json!({
        "nozzle_diameter": [0.4],
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": "150%",
        "slow_down_for_layer_cooling": false
    }));
    let absolute = options(json!({
        "nozzle_diameter": [0.4],
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0.6,
        "slow_down_for_layer_cooling": false
    }));

    assert_delta_eq(
        first_extrusion_delta(&output_for_layer(&percent, 1)),
        first_extrusion_delta(&output_for_layer(&absolute, 1)),
    );
}

#[test]
fn zero_support_line_width_preserves_support_material_line_width_fallback() {
    let omitted = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "slow_down_for_layer_cooling": false
    }));
    let zero = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_line_width": 0,
        "slow_down_for_layer_cooling": false
    }));

    assert_delta_eq(
        first_extrusion_delta(&output_for_layer(&omitted, 1)),
        first_extrusion_delta(&output_for_layer(&zero, 1)),
    );
}

#[test]
fn initial_layer_line_width_overrides_support_line_width_for_support_material() {
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

#[test]
fn invalid_support_line_width_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!("bad%"),
        json!("wide"),
        json!([]),
        json!(true),
        Value::Null,
    ] {
        assert_invalid_option("support_line_width", value);
    }
}
