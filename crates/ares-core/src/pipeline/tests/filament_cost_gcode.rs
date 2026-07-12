use crate::{
    LayerSpeedMoves, Point2, SliceError, SliceOptions, ToolpathMoveKind, gcode::format_gcode,
    pipeline::test_support::rectangular_pipeline,
};
use serde_json::{Value, json};

#[test]
fn default_filament_stats_emit_length_and_volume_without_cost() {
    let options = options(json!({}));
    let gcode = gcode(&options);
    let used = pipeline_used_filament_mm(&options);
    let volume = extruded_volume_mm3(&options, used);

    assert!(has_line(
        &gcode,
        format!("; filament used [mm] = {used:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; filament used [cm3] = {:.2}", volume * 0.001)
    ));
    assert!(!gcode.contains("; filament used [g] ="));
    assert!(!gcode.contains("; filament cost ="));
}

#[test]
fn filament_cost_uses_generated_extrusion_density_and_first_diameter() {
    let options = options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0, 3.0]
    }));
    let gcode = gcode(&options);
    let used = pipeline_used_filament_mm(&options);
    let volume = extruded_volume_mm3(&options, used);
    let expected_weight = volume;
    let expected_cost = expected_weight * 2.5;

    assert!(has_line(
        &gcode,
        format!("; filament used [g] = {expected_weight:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; filament cost = {expected_cost:.2}")
    ));
}

#[test]
fn material_cost_emits_matching_total_lines_without_time_cost() {
    let options = options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0]
    }));
    let gcode = gcode(&options);
    let used = pipeline_used_filament_mm(&options);
    let volume = extruded_volume_mm3(&options, used);
    let expected_weight = volume;
    let material_cost = expected_weight * 2.5;

    assert!(has_line(
        &gcode,
        format!("; filament cost = {material_cost:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; total filament used [g] = {expected_weight:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; total filament cost = {material_cost:.2}")
    ));
}

#[test]
fn zero_time_cost_keeps_total_cost_material_only() {
    let options = options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0],
        "time_cost": 0.0
    }));
    let gcode = gcode(&options);
    let used = pipeline_used_filament_mm(&options);
    let volume = extruded_volume_mm3(&options, used);
    let expected_weight = volume;
    let material_cost = expected_weight * 2.5;

    assert!(has_line(
        &gcode,
        format!("; total filament used [g] = {expected_weight:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; total filament cost = {material_cost:.2}")
    ));
}

#[test]
fn time_cost_adds_hourly_component_to_total_filament_cost_only() {
    let options = options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0],
        "time_cost": 3600.0
    }));
    let gcode = gcode(&options);
    let used = pipeline_used_filament_mm(&options);
    let volume = extruded_volume_mm3(&options, used);
    let expected_weight = volume;
    let material_cost = expected_weight * 2.5;
    let time_cost = pipeline_print_time_s(&options);
    let total_cost = material_cost + time_cost;

    assert!(has_line(
        &gcode,
        format!("; filament cost = {material_cost:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; total filament used [g] = {expected_weight:.2}")
    ));
    assert!(has_line(
        &gcode,
        format!("; total filament cost = {total_cost:.2}")
    ));
    assert!(!gcode.contains(&format!("; filament cost = {total_cost:.2}")));
}

#[test]
fn time_cost_emits_total_cost_without_material_cost() {
    let options = options(json!({
        "filament_density": [0.0],
        "filament_cost": [0.0],
        "filament_diameter": [2.0],
        "time_cost": 3600.0
    }));
    let gcode = gcode(&options);
    let total_cost = pipeline_print_time_s(&options);

    assert!(!gcode.contains("; filament used [g] ="));
    assert!(!gcode.contains("; filament cost ="));
    assert!(has_line(
        &gcode,
        format!("; total filament cost = {total_cost:.2}")
    ));
}

#[test]
fn filament_cost_accepts_orca_numeric_vector_forms() {
    for value in [
        json!("2500"),
        json!("2500;3000"),
        json!("2500,3000"),
        json!([2500.0, "3000"]),
    ] {
        let options = options(json!({
            "filament_density": [1000.0],
            "filament_cost": value,
            "filament_diameter": [2.0]
        }));
        let gcode = gcode(&options);

        assert!(
            gcode
                .lines()
                .any(|line| line.starts_with("; filament cost = "))
        );
    }
}

#[test]
fn zero_cost_or_zero_density_suppresses_cost_line() {
    let zero_cost = gcode(&options(json!({
        "filament_density": [1000.0],
        "filament_cost": 0.0,
        "filament_diameter": [2.0]
    })));
    let zero_density = gcode(&options(json!({
        "filament_density": [0.0],
        "filament_cost": 2500.0,
        "filament_diameter": [2.0]
    })));

    assert!(!zero_cost.contains("; filament cost ="));
    assert!(!zero_density.contains("; filament used [g] ="));
    assert!(!zero_density.contains("; filament cost ="));
}

#[test]
fn filament_cost_stats_are_inserted_before_program_end() {
    let options = options(json!({
        "machine_end_gcode": "G28 X",
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0]
    }));
    let gcode = gcode(&options);

    assert!(
        gcode
            .find("G28 X\n; filament used [mm] =")
            .expect("stats must follow custom end gcode")
            < gcode.find("; filament cost =").unwrap()
    );
    assert!(gcode.find("; filament cost =").unwrap() < gcode.find("\nM2\n").unwrap());
}

#[test]
fn line_numbering_applies_to_filament_statistics() {
    let options = options(json!({
        "gcode_add_line_number": true,
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0]
    }));
    let gcode = gcode(&options);

    assert!(
        gcode
            .lines()
            .any(|line| line.contains(" ; filament cost = "))
    );
    assert!(!gcode.lines().any(|line| line == "; filament cost = 2.06"));
}

#[test]
fn filament_cost_rejects_invalid_values() {
    for invalid in [
        json!(-0.01),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(""),
        json!([]),
        json!(["2500", "NaN"]),
        json!([["2500"]]),
        json!({"value": 2500.0}),
        Value::Null,
    ] {
        let options = options(json!({ "filament_cost": invalid }));
        let pipeline = rectangular_pipeline(&options);
        let err = format_gcode(&pipeline, &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn time_cost_accepts_first_orca_numeric_value_and_rejects_invalid_values() {
    for value in [
        json!(3600.0),
        json!("3600;7200"),
        json!("3600,7200"),
        json!([3600.0]),
    ] {
        let options = options(json!({ "time_cost": value }));
        let gcode = gcode(&options);

        assert!(
            gcode
                .lines()
                .any(|line| line.starts_with("; total filament cost = "))
        );
    }

    let multi_value = options(json!({ "time_cost": "360000;720000" }));
    let gcode = gcode(&multi_value);
    let first_value_total = pipeline_print_time_s(&multi_value) * 100.0;
    let second_value_total = pipeline_print_time_s(&multi_value) * 200.0;

    assert!(has_line(
        &gcode,
        format!("; total filament cost = {first_value_total:.2}")
    ));
    assert!(!has_line(
        &gcode,
        format!("; total filament cost = {second_value_total:.2}")
    ));

    for invalid in [
        json!(-0.01),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(""),
        json!([]),
        json!(["3600", "NaN"]),
        json!([["3600"]]),
        json!({"value": 3600.0}),
        Value::Null,
    ] {
        let options = options(json!({ "time_cost": invalid }));
        let pipeline = rectangular_pipeline(&options);
        let err = format_gcode(&pipeline, &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn filament_and_time_cost_do_not_change_movement_or_extrusion_commands() {
    let baseline = command_lines(&gcode(&options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0],
        "time_cost": 0.0
    }))));
    let configured = command_lines(&gcode(&options(json!({
        "filament_density": [1000.0],
        "filament_cost": [2500.0],
        "filament_diameter": [2.0],
        "time_cost": 3600.0
    }))));

    assert_eq!(baseline, configured);
}

fn gcode(options: &SliceOptions) -> String {
    String::from_utf8(format_gcode(&rectangular_pipeline(options), options).unwrap()).unwrap()
}

fn has_line(gcode: &str, expected: impl AsRef<str>) -> bool {
    let expected = expected.as_ref();
    gcode.lines().any(|line| line == expected)
}

fn pipeline_used_filament_mm(options: &SliceOptions) -> f64 {
    rectangular_pipeline(options)
        .layer_extrusion_moves()
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum()
}

fn extruded_volume_mm3(options: &SliceOptions, used_filament_mm: f64) -> f64 {
    let diameter = options.filament_diameters().unwrap()[0];
    used_filament_mm * std::f64::consts::PI * (diameter * 0.5).powi(2)
}

fn pipeline_print_time_s(options: &SliceOptions) -> f64 {
    rectangular_pipeline(options)
        .layer_speed_moves()
        .iter()
        .map(layer_print_time_s)
        .sum()
}

fn layer_print_time_s(layer: &LayerSpeedMoves) -> f64 {
    let mut last_point = None;
    let mut had_print = false;
    let mut total = 0.0;
    for move_ in layer.moves() {
        let start = last_point.unwrap_or(move_.point());
        if move_.kind() == ToolpathMoveKind::Print {
            had_print = true;
        }
        if had_print {
            let length = distance(start, move_.point());
            if length > 0.0 && move_.speed_mm_s() > 0.0 {
                total += length / move_.speed_mm_s();
            }
        }
        last_point = Some(move_.point());
    }
    total
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
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
