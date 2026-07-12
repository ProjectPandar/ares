use super::*;
use crate::{ExtrusionRole, PrintPathRole};
use serde_json::json;

#[test]
fn density_100_emits_bottom_interior_and_top_surface_roles() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_print_paths()[2]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
    assert!(
        pipeline.print().objects()[0].layers()[0].regions()[0]
            .fills()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::BottomSurface)
    );
    assert!(
        pipeline.print().objects()[0].layers()[2].regions()[0]
            .fills()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::TopSolidInfill)
    );
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(gcode.contains(";PRINT_PATH:solid_infill:"));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:"));
    assert!(gcode.contains(";SPEED:print:bottom_surface:"));
    assert!(gcode.contains(";SPEED:print:top_solid_infill:"));
}

#[test]
fn single_layer_density_100_emits_bottom_surface_not_top_surface() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0
    }))
    .unwrap();
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 1);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(!gcode.contains(";PRINT_PATH:top_solid_infill:"));
}

#[test]
fn top_bottom_surface_numeric_options_change_gcode_independently() {
    let base = solid_surface_options(json!({}));
    let wide_top = solid_surface_options(json!({ "top_surface_line_width": 0.8 }));
    let high_top_flow = solid_surface_options(json!({ "top_solid_infill_flow_ratio": 1.6 }));
    let high_bottom_flow = solid_surface_options(json!({ "bottom_solid_infill_flow_ratio": 1.6 }));
    let slow_top = solid_surface_options(json!({ "top_surface_speed": 40 }));
    let top_accel = solid_surface_options(json!({
        "default_acceleration": 500,
        "top_surface_acceleration": 222
    }));
    let top_jerk = solid_surface_options(json!({
        "default_jerk": 8,
        "top_surface_jerk": 3
    }));

    let base_gcode = solid_surface_gcode(&base);
    let wide_top_gcode = solid_surface_gcode(&wide_top);
    let high_top_flow_gcode = solid_surface_gcode(&high_top_flow);
    let high_bottom_flow_gcode = solid_surface_gcode(&high_bottom_flow);
    let slow_top_gcode = solid_surface_gcode(&slow_top);
    let top_accel_gcode = solid_surface_gcode(&top_accel);
    let top_jerk_gcode = solid_surface_gcode(&top_jerk);

    assert_ne!(
        first_role_extrusion_delta(&base_gcode, "top_solid_infill"),
        first_role_extrusion_delta(&wide_top_gcode, "top_solid_infill")
    );
    assert_ne!(
        first_role_extrusion_delta(&base_gcode, "top_solid_infill"),
        first_role_extrusion_delta(&high_top_flow_gcode, "top_solid_infill")
    );
    assert_eq!(
        first_role_extrusion_delta(&base_gcode, "bottom_surface"),
        first_role_extrusion_delta(&high_top_flow_gcode, "bottom_surface")
    );
    assert_ne!(
        first_role_extrusion_delta(&base_gcode, "bottom_surface"),
        first_role_extrusion_delta(&high_bottom_flow_gcode, "bottom_surface")
    );
    assert_eq!(
        first_role_extrusion_delta(&base_gcode, "top_solid_infill"),
        first_role_extrusion_delta(&high_bottom_flow_gcode, "top_solid_infill")
    );
    assert_ne!(
        first_role_feedrate(&base_gcode, "top_solid_infill"),
        first_role_feedrate(&slow_top_gcode, "top_solid_infill")
    );
    assert_eq!(
        first_role_feedrate(&base_gcode, "bottom_surface"),
        first_role_feedrate(&slow_top_gcode, "bottom_surface")
    );
    assert_eq!(first_role_feedrate(&base_gcode, "bottom_surface"), 3600.0);
    assert_role_command(&top_accel_gcode, "top_solid_infill", "M204 S222");
    assert_ne!(
        first_role_command(&base_gcode, "top_solid_infill", "M204 S"),
        first_role_command(&top_accel_gcode, "top_solid_infill", "M204 S")
    );
    assert_eq!(
        first_role_command(&base_gcode, "bottom_surface", "M204 S"),
        first_role_command(&top_accel_gcode, "bottom_surface", "M204 S")
    );
    assert_eq!(
        first_role_command(&base_gcode, "solid_infill", "M204 S"),
        first_role_command(&top_accel_gcode, "solid_infill", "M204 S")
    );
    assert_role_command(&top_jerk_gcode, "top_solid_infill", "M205 X3 Y3");
    assert_ne!(
        first_role_command(&base_gcode, "top_solid_infill", "M205 X"),
        first_role_command(&top_jerk_gcode, "top_solid_infill", "M205 X")
    );
    assert_eq!(
        first_role_command(&base_gcode, "bottom_surface", "M205 X"),
        first_role_command(&top_jerk_gcode, "bottom_surface", "M205 X")
    );
    assert_eq!(
        first_role_command(&base_gcode, "solid_infill", "M205 X"),
        first_role_command(&top_jerk_gcode, "solid_infill", "M205 X")
    );
}

#[test]
fn top_surface_acceleration_and_jerk_do_not_change_sparse_infill_gcode() {
    let base = sparse_options(json!({}));
    let top_accel = sparse_options(json!({
        "default_acceleration": 500,
        "sparse_infill_acceleration": 333,
        "top_surface_acceleration": 222
    }));
    let top_jerk = sparse_options(json!({
        "default_jerk": 8,
        "infill_jerk": 5,
        "top_surface_jerk": 3
    }));

    let base_gcode = sparse_gcode(&base);
    let top_accel_gcode = sparse_gcode(&top_accel);
    let top_jerk_gcode = sparse_gcode(&top_jerk);

    assert_eq!(
        first_role_command(&base_gcode, "sparse_infill", "M204 S"),
        first_role_command(&top_accel_gcode, "sparse_infill", "M204 S")
    );
    assert_eq!(
        first_role_command(&base_gcode, "sparse_infill", "M205 X"),
        first_role_command(&top_jerk_gcode, "sparse_infill", "M205 X")
    );
}

fn solid_surface_options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "line_width": 0.4,
        "default_acceleration": 500,
        "internal_solid_infill_acceleration": 444,
        "default_jerk": 8,
        "infill_jerk": 5,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    value.as_object_mut().unwrap().extend(
        extra
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}

fn solid_surface_gcode(options: &SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(options, 3);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
}

#[test]
fn shell_layer_counts_change_density_100_surface_roles() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 2,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 5);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(
        surface_roles(&pipeline),
        [
            PrintPathRole::BottomSurface,
            PrintPathRole::BottomSurface,
            PrintPathRole::SolidInfill,
            PrintPathRole::SolidInfill,
            PrintPathRole::TopSolidInfill,
        ]
    );
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(gcode.contains(";PRINT_PATH:solid_infill:"));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:"));
}

fn surface_roles<const N: usize>(pipeline: &crate::SlicingPipeline) -> [PrintPathRole; N] {
    pipeline
        .layer_print_paths()
        .iter()
        .map(|layer| {
            layer
                .paths()
                .iter()
                .find(|path| {
                    matches!(
                        path.role(),
                        PrintPathRole::BottomSurface
                            | PrintPathRole::SolidInfill
                            | PrintPathRole::TopSolidInfill
                    )
                })
                .unwrap()
                .role()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn sparse_options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "line_width": 0.4,
        "default_acceleration": 500,
        "sparse_infill_acceleration": 333,
        "default_jerk": 8,
        "infill_jerk": 5
    });
    value.as_object_mut().unwrap().extend(
        extra
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}

fn sparse_gcode(options: &SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(options, 3);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn first_role_extrusion_delta(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let prefix = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&prefix) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing {role} extrusion");
}

fn first_role_feedrate(gcode: &str, role: &str) -> f64 {
    let prefix = format!(";SPEED:print:{role}:");
    gcode
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|line| line.rsplit_once(':').map(|(_, feedrate)| feedrate))
        .and_then(|feedrate| feedrate.parse::<f64>().ok())
        .unwrap()
}

fn assert_role_command(gcode: &str, role: &str, command: &str) {
    assert_eq!(first_role_command(gcode, role, command), command);
}

fn first_role_command<'a>(gcode: &'a str, role: &str, command_prefix: &str) -> &'a str {
    let target = format!(";MOVE:print:{role}:");
    let lines = gcode.lines().collect::<Vec<_>>();
    let move_index = lines
        .iter()
        .position(|line| line.starts_with(&target))
        .unwrap();
    lines[move_index + 1..]
        .iter()
        .take_while(|line| !line.starts_with(';') || line.starts_with(";MOVE:print:"))
        .find(|line| line.starts_with(command_prefix))
        .copied()
        .unwrap()
}
