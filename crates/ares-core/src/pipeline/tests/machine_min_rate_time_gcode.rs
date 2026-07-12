use crate::{
    SliceError, SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline,
};
use serde_json::{Value, json};

#[test]
fn machine_min_extruding_rate_reduces_time_cost_without_changing_movement_commands() {
    let baseline_options = options(json!({
        "time_cost": 3600.0
    }));
    let clamped_options = options(json!({
        "time_cost": 3600.0,
        "machine_min_extruding_rate": [10_000.0, 0.0]
    }));

    let baseline = gcode(&baseline_options);
    let clamped = gcode(&clamped_options);

    assert!(total_filament_cost(&clamped) < total_filament_cost(&baseline));
    assert_eq!(command_lines(&baseline), command_lines(&clamped));
}

#[test]
fn machine_min_travel_rate_reduces_time_cost_without_changing_movement_commands() {
    let baseline_options = options(json!({
        "time_cost": 3600.0,
        "skirt_loops": 1
    }));
    let clamped_options = options(json!({
        "time_cost": 3600.0,
        "skirt_loops": 1,
        "machine_min_travel_rate": [10_000.0, 0.0]
    }));

    let baseline = gcode(&baseline_options);
    let clamped = gcode(&clamped_options);

    assert!(total_filament_cost(&clamped) < total_filament_cost(&baseline));
    assert_eq!(command_lines(&baseline), command_lines(&clamped));
}

#[test]
fn machine_min_rates_use_first_normal_value_for_time_cost() {
    let baseline_options = options(json!({
        "time_cost": 3600.0
    }));
    let stealth_only_options = options(json!({
        "time_cost": 3600.0,
        "machine_min_extruding_rate": [0.0, 10_000.0],
        "machine_min_travel_rate": [0.0, 10_000.0]
    }));

    assert_eq!(
        total_filament_cost(&gcode(&baseline_options)),
        total_filament_cost(&gcode(&stealth_only_options))
    );
}

#[test]
fn machine_min_rates_reject_invalid_values() {
    for (key, invalid) in [
        ("machine_min_extruding_rate", json!(-0.01)),
        ("machine_min_extruding_rate", json!("NaN")),
        ("machine_min_extruding_rate", json!("bad")),
        ("machine_min_extruding_rate", json!([])),
        ("machine_min_travel_rate", json!(-0.01)),
        ("machine_min_travel_rate", json!("inf")),
        ("machine_min_travel_rate", json!([0.0, "NaN"])),
        ("machine_min_travel_rate", Value::Null),
    ] {
        let options = options(json!({ key: invalid }));
        let err = format_gcode(&rectangular_pipeline(&options), &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(message) if message.contains(key)));
    }
}

fn gcode(options: &SliceOptions) -> String {
    String::from_utf8(format_gcode(&rectangular_pipeline(options), options).unwrap()).unwrap()
}

fn total_filament_cost(gcode: &str) -> f64 {
    gcode
        .lines()
        .find_map(|line| line.strip_prefix("; total filament cost = "))
        .unwrap()
        .parse()
        .unwrap()
}

fn command_lines(gcode: &str) -> Vec<String> {
    gcode
        .lines()
        .filter(|line| line.starts_with('G') || line.starts_with('M'))
        .map(str::to_owned)
        .collect()
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    });
    value
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(value).unwrap()
}
