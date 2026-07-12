use crate::{
    SliceError, SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline,
};
use serde_json::{Value, json};

#[test]
fn filament_density_header_emits_default_when_missing() {
    let options = options(json!({}));
    let gcode = gcode(&options);

    assert!(gcode.lines().any(|line| line == "; filament_density = 0"));
}

#[test]
fn filament_density_header_formats_numeric_vector_forms() {
    let options = options(json!({ "filament_density": "1.24;1.27" }));
    let gcode = gcode(&options);

    assert!(
        gcode
            .lines()
            .any(|line| line == "; filament_density = 1.24,1.27")
    );
}

#[test]
fn filament_density_header_accepts_scalar_and_array_forms() {
    let numeric_scalar = gcode(&options(json!({ "filament_density": 1.24 })));
    let string_scalar = gcode(&options(json!({ "filament_density": "1.31" })));
    let array = gcode(&options(json!({ "filament_density": [1.24, "1.27"] })));

    assert!(
        numeric_scalar
            .lines()
            .any(|line| line == "; filament_density = 1.24")
    );
    assert!(
        string_scalar
            .lines()
            .any(|line| line == "; filament_density = 1.31")
    );
    assert!(
        array
            .lines()
            .any(|line| line == "; filament_density = 1.24,1.27")
    );
}

#[test]
fn filament_density_header_rejects_invalid_values() {
    for invalid in [
        json!(-0.01),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(""),
        json!([]),
        json!(["1.24", "NaN"]),
        json!([["1.24"]]),
        json!({"value": 1.24}),
        Value::Null,
    ] {
        let options = options(json!({ "filament_density": invalid }));
        let pipeline = rectangular_pipeline(&options);
        let err = format_gcode(&pipeline, &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn filament_density_header_does_not_change_movement_or_extrusion_commands() {
    let baseline = command_lines(&gcode(&options(json!({}))));
    let configured = command_lines(&gcode(&options(json!({ "filament_density": [1.24] }))));

    assert_eq!(baseline, configured);
}

#[test]
fn filament_density_header_follows_btt_thumbnail_suppression() {
    let options = options(json!({
        "filament_density": [1.24],
        "thumbnails": "64x64/BTT_TFT"
    }));
    let gcode = gcode(&options);

    assert!(!gcode.contains("; filament_density ="));
}

fn gcode(options: &SliceOptions) -> String {
    String::from_utf8(format_gcode(&rectangular_pipeline(options), options).unwrap()).unwrap()
}

fn command_lines(gcode: &str) -> Vec<String> {
    gcode
        .lines()
        .filter(|line| line.starts_with('G') || line.starts_with('M'))
        .map(str::to_owned)
        .collect()
}

fn options(extra: Value) -> SliceOptions {
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
