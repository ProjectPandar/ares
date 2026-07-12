use super::*;

#[test]
fn role_change_placeholders_receive_solid_infill_role() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 100,
        "sparse_infill_line_width": 0.4,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "brim_width": 0,
        "skirt_loops": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "filament_change_extrusion_role_gcode": ";FILAMENT-ROLE [last_extrusion_role]->[extrusion_role]"
    }))
    .unwrap();

    let output = crate::gcode::format_gcode(
        &crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3),
        &options,
    )
    .map(|bytes| String::from_utf8(bytes).unwrap())
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";FILAMENT-ROLE ") && line.ends_with("->solid_infill"))
    );
    assert!(output.contains(";EXTRUSION:print:solid_infill:"));
}
