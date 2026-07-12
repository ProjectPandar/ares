use crate::{
    SliceError, SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline,
};
use serde_json::{Map, Value, json};

#[test]
fn default_small_perimeter_threshold_preserves_external_perimeter_feedrate() {
    let options = options(json!({}));
    let gcode = String::from_utf8(format_gcode(&rectangular_pipeline(&options), &options).unwrap())
        .unwrap();

    assert_eq!(
        first_speed_feedrate(&gcode, "external_perimeter", "print"),
        6000.0
    );
}

#[test]
fn small_perimeter_threshold_uses_orca_length_conversion_in_gcode() {
    let below = options(json!({
        "small_perimeter_threshold": 2.5,
        "small_perimeter_speed": 20
    }));
    let above = options(json!({
        "small_perimeter_threshold": 2.6,
        "small_perimeter_speed": 20
    }));

    let below_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&below), &below).unwrap()).unwrap();
    let above_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&above), &above).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&below_gcode, "external_perimeter", "print"),
        6000.0
    );
    assert_eq!(
        first_speed_feedrate(&above_gcode, "external_perimeter", "print"),
        1200.0
    );
}

#[test]
fn percent_small_perimeter_speed_resolves_over_outer_wall_speed() {
    let options = options(json!({
        "small_perimeter_threshold": 3.0,
        "small_perimeter_speed": "25%"
    }));
    let gcode = String::from_utf8(format_gcode(&rectangular_pipeline(&options), &options).unwrap())
        .unwrap();

    assert_eq!(
        first_speed_feedrate(&gcode, "external_perimeter", "print"),
        1500.0
    );
}

#[test]
fn zero_small_perimeter_speed_uses_auto_half_outer_wall_speed() {
    let options = options(json!({
        "small_perimeter_threshold": 3.0,
        "small_perimeter_speed": 0
    }));
    let gcode = String::from_utf8(format_gcode(&rectangular_pipeline(&options), &options).unwrap())
        .unwrap();

    assert_eq!(
        first_speed_feedrate(&gcode, "external_perimeter", "print"),
        3000.0
    );
}

#[test]
fn volumetric_cap_still_reduces_selected_small_perimeter_speed() {
    let uncapped = options(json!({
        "small_perimeter_threshold": 3.0,
        "small_perimeter_speed": 80,
        "filament_max_volumetric_speed": 0.0
    }));
    let capped = options(json!({
        "small_perimeter_threshold": 3.0,
        "small_perimeter_speed": 80,
        "filament_max_volumetric_speed": 1.0
    }));

    let uncapped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&uncapped), &uncapped).unwrap())
            .unwrap();
    let capped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&capped), &capped).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print"),
        4800.0
    );
    assert!(
        first_speed_feedrate(&capped_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print")
    );
}

#[test]
fn rejects_invalid_small_perimeter_options() {
    for (key, value) in [
        ("small_perimeter_threshold", json!(-0.1)),
        ("small_perimeter_threshold", json!("wide")),
        ("small_perimeter_speed", json!(-1)),
        ("small_perimeter_speed", json!("fast")),
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

fn first_speed_feedrate(gcode: &str, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    gcode
        .lines()
        .find_map(|line| {
            line.starts_with(&target)
                .then(|| line.rsplit(':').next().unwrap().parse().unwrap())
        })
        .unwrap_or_else(|| panic!("missing {kind} {role} speed"))
}
