use super::*;
use crate::{PrintPathRole, SliceError};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_type_emits_no_ordinary_ironing() {
    let output = output_for(options(json!({})), 3);

    assert!(role_count(&output, "top_solid_infill") > 0);
    assert_eq!(role_count(&output, "ironing"), 0);
}

#[test]
fn no_ironing_type_emits_no_ordinary_ironing() {
    let output = output_for(options(json!({ "ironing_type": "no ironing" })), 3);

    assert!(role_count(&output, "top_solid_infill") > 0);
    assert_eq!(role_count(&output, "ironing"), 0);
}

#[test]
fn top_ironing_type_duplicates_generated_top_surface_after_source_path() {
    let output = output_for(options(json!({ "ironing_type": "top" })), 3);
    let top_solid_count = role_count(&output, "top_solid_infill");

    assert!(top_solid_count > 0);
    assert_eq!(role_count(&output, "ironing"), top_solid_count);
    assert!(
        output.find(";EXTRUSION:print:top_solid_infill:")
            < output.find(";EXTRUSION:print:ironing:")
    );
    assert!(output.contains(";SPEED:print:ironing:"));
    assert!(output.contains(";PRINT_PATH:ironing:"));
}

#[test]
fn topmost_ironing_type_only_duplicates_final_top_surface_layer() {
    let options = options(json!({
        "ironing_type": "topmost",
        "top_shell_layers": 2
    }));
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 4);
    let output =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer_2_top_count = layer_role_count(&output, 2, "top_solid_infill");
    let layer_3_top_count = layer_role_count(&output, 3, "top_solid_infill");

    assert!(layer_2_top_count > 0);
    assert!(layer_3_top_count > 0);
    assert_eq!(
        role_count(&output, "top_solid_infill"),
        layer_2_top_count + layer_3_top_count
    );
    assert_eq!(role_count(&output, "ironing"), layer_3_top_count);
    assert_eq!(layer_role_count(&output, 2, "ironing"), 0);
    assert_eq!(layer_role_count(&output, 3, "ironing"), layer_3_top_count);
}

#[test]
fn solid_ironing_type_duplicates_current_solid_area_roles() {
    let output = output_for(options(json!({ "ironing_type": "solid" })), 3);
    let bottom_count = role_count(&output, "bottom_surface");
    let solid_count = role_count(&output, "solid_infill");
    let top_count = role_count(&output, "top_solid_infill");

    assert!(bottom_count > 0);
    assert!(solid_count > 0);
    assert!(top_count > 0);
    assert_eq!(
        role_count(&output, "ironing"),
        bottom_count + solid_count + top_count
    );
    assert_eq!(
        layer_role_count(&output, 0, "ironing"),
        layer_role_count(&output, 0, "bottom_surface")
    );
    assert_eq!(
        layer_role_count(&output, 1, "ironing"),
        layer_role_count(&output, 1, "solid_infill")
    );
    assert_eq!(
        layer_role_count(&output, 2, "ironing"),
        layer_role_count(&output, 2, "top_solid_infill")
    );
}

#[test]
fn ordinary_ironing_type_does_not_duplicate_support_interface_paths() {
    let options = options(json!({ "ironing_type": "solid" }));
    let pipeline = crate::pipeline::test_support::single_path_pipeline(
        &options,
        PrintPathRole::SupportMaterialInterface,
        1,
    );
    let output =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(role_count(&output, "support_material_interface"), 1);
    assert_eq!(role_count(&output, "ironing"), 0);
}

#[test]
fn invalid_ironing_type_values_reach_slice_error() {
    for value in [
        json!("TopSurfaces"),
        json!("top surfaces"),
        json!("all"),
        json!(""),
        json!(true),
        json!([]),
        json!({ "value": "top" }),
        Value::Null,
    ] {
        let options = options(json!({ "ironing_type": value }));
        let err = crate::finalize_print_paths(Vec::new(), &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_type"));
    }
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
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
        "ironing_flow": 10,
        "ironing_speed": 20,
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

fn output_for(options: SliceOptions, layer_count: usize) -> String {
    let pipeline =
        crate::pipeline::test_support::rectangular_layers_pipeline(&options, layer_count);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn role_count(output: &str, role: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(&format!(";EXTRUSION:print:{role}:")))
        .count()
}

fn layer_role_count(output: &str, layer_id: usize, role: &str) -> usize {
    let marker = format!(";LAYER:{layer_id}");
    let start = output
        .lines()
        .position(|line| line == marker)
        .unwrap_or_else(|| panic!("missing {marker}"));
    output
        .lines()
        .skip(start + 1)
        .take_while(|line| !line.starts_with(";LAYER:"))
        .filter(|line| line.starts_with(&format!(";EXTRUSION:print:{role}:")))
        .count()
}
