use crate::{
    PrintPathRole, SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Map, Value, json};

#[test]
fn resonance_avoidance_snaps_external_perimeter_upper_half_to_max_speed() {
    let options = options(json!({
        "outer_wall_speed": 100,
        "resonance_avoidance": true,
        "min_resonance_avoidance_speed": 70,
        "max_resonance_avoidance_speed": 120
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        7200.0
    );
}

#[test]
fn resonance_avoidance_preserves_external_perimeter_lower_half_below_min_speed() {
    let options = options(json!({
        "outer_wall_speed": 60,
        "resonance_avoidance": true,
        "min_resonance_avoidance_speed": 70,
        "max_resonance_avoidance_speed": 120
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        3600.0
    );
}

#[test]
fn resonance_avoidance_skips_non_external_perimeter_roles() {
    let options = options(json!({
        "outer_wall_speed": 100,
        "inner_wall_speed": 100,
        "resonance_avoidance": true,
        "min_resonance_avoidance_speed": 70,
        "max_resonance_avoidance_speed": 120
    }));

    let gcode = layer_gcode(&options, PrintPathRole::InternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "internal_perimeter", "print"),
        6000.0
    );
}

#[test]
fn resonance_avoidance_skips_external_perimeter_above_max_speed() {
    let options = options(json!({
        "outer_wall_speed": 130,
        "resonance_avoidance": true,
        "min_resonance_avoidance_speed": 70,
        "max_resonance_avoidance_speed": 120
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        7800.0
    );
}

#[test]
fn resonance_avoidance_rejects_invalid_values() {
    for (key, value) in [
        ("resonance_avoidance", json!("yes")),
        ("min_resonance_avoidance_speed", json!(-0.1)),
        ("min_resonance_avoidance_speed", json!("slow")),
        ("max_resonance_avoidance_speed", json!(-0.1)),
        ("max_resonance_avoidance_speed", json!("fast")),
    ] {
        let mut values = Map::new();
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

        assert!(matches!(
            options.speed_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_speed": 100,
        "initial_layer_speed": 100,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn layer_gcode(options: &SliceOptions, role: PrintPathRole, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, role, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn layer_speed_feedrate(gcode: &str, layer_id: usize, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    let mut current_layer = None;
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if current_layer == Some(layer_id) && line.starts_with(&target) {
            return line.rsplit(':').next().unwrap().parse().unwrap();
        }
    }
    panic!("missing layer {layer_id} {kind} {role} speed");
}
