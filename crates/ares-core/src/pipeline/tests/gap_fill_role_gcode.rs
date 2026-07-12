use crate::{
    PrintPathRole, SliceOptions,
    gcode::format_gcode,
    pipeline::test_support::{
        contour_layers_pipeline_from_layers_for_tests, narrow_rectangular_gap_fill_pipeline,
        single_path_pipeline,
    },
};
use serde_json::json;

#[test]
fn constructed_gap_fill_path_reaches_gcode_speed_and_extrusion_comments() {
    let low = options(json!({
        "gap_infill_speed": 25,
        "set_other_flow_ratios": true,
        "gap_fill_flow_ratio": 0.5,
        "first_layer_flow_ratio": 0.5
    }));
    let high = options(json!({
        "gap_infill_speed": 45,
        "set_other_flow_ratios": true,
        "gap_fill_flow_ratio": 1.5,
        "first_layer_flow_ratio": 0.5
    }));

    let low_pipeline = single_path_pipeline(&low, PrintPathRole::GapFill, 0);
    let high_pipeline = single_path_pipeline(&high, PrintPathRole::GapFill, 0);
    let low_gcode = String::from_utf8(format_gcode(&low_pipeline, &low).unwrap()).unwrap();
    let high_gcode = String::from_utf8(format_gcode(&high_pipeline, &high).unwrap()).unwrap();

    assert!(high_gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(high_gcode.contains(";EXTRUSION:print:gap_fill:"));
    assert!(high_gcode.contains(";SPEED:print:gap_fill:1,0:2700"));
    assert!(high_gcode.contains(";MOVE:print:gap_fill:"));
    assert_delta_eq(
        first_extrusion_delta(&high_gcode, "gap_fill"),
        first_extrusion_delta(&low_gcode, "gap_fill") * 3.0,
    );
}

#[test]
fn constructed_gap_fill_print_domain_entity_stays_in_extras() {
    let options = options(json!({}));
    let pipeline = single_path_pipeline(&options, PrintPathRole::GapFill, 0);
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert!(region.perimeters().is_empty());
    assert!(region.fills().is_empty());
    assert!(
        region
            .extras()
            .paths()
            .iter()
            .any(|path| path.role() == crate::ExtrusionRole::GapFill)
    );
}

#[test]
fn filter_out_gap_fill_removes_short_constructed_gap_fill_before_gcode() {
    let options = options(json!({ "filter_out_gap_fill": 1.1 }));
    let pipeline = single_path_pipeline(&options, PrintPathRole::GapFill, 0);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert!(!gcode.contains("gap_fill"));
    assert!(pipeline.layer_print_paths()[0].paths().is_empty());
    assert!(pipeline.layer_toolpath_moves()[0].moves().is_empty());
    assert!(pipeline.layer_extrusion_moves()[0].moves().is_empty());
    assert!(pipeline.layer_speed_moves()[0].moves().is_empty());
    assert_eq!(
        pipeline.layer_extrusion_moves()[0].total_extrusion_mm(),
        0.0
    );
    assert!(region.extras().is_empty());
}

#[test]
fn filter_out_gap_fill_keeps_constructed_gap_fill_at_equal_threshold() {
    let options = options(json!({ "filter_out_gap_fill": 1.0 }));
    let pipeline = single_path_pipeline(&options, PrintPathRole::GapFill, 0);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PRINT_PATH:gap_fill:"));
    assert_eq!(pipeline.layer_print_paths()[0].paths().len(), 1);
}

#[test]
fn generated_rectangular_gap_fill_reaches_print_domain_and_gcode() {
    let options = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 45
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::GapFill)
    );
    assert_eq!(region.extras().len(), 1);
    assert_eq!(
        region.extras().paths()[0].role(),
        crate::ExtrusionRole::GapFill
    );
    assert!(gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(gcode.contains(";SPEED:print:gap_fill:2.6,0.35:2700"));
    assert!(pipeline.diagnostics().total_print_path_count() >= 2);
    assert!(pipeline.diagnostics().total_toolpath_move_count() >= 2);
}

#[test]
fn generated_gap_fill_speed_and_flow_ratio_reach_gcode() {
    let low = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 25,
        "set_other_flow_ratios": true,
        "gap_fill_flow_ratio": 0.5,
        "first_layer_flow_ratio": 0.5
    }));
    let high = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 45,
        "set_other_flow_ratios": true,
        "gap_fill_flow_ratio": 1.5,
        "first_layer_flow_ratio": 0.5
    }));

    let low_gcode =
        String::from_utf8(format_gcode(&narrow_rectangular_gap_fill_pipeline(&low), &low).unwrap())
            .unwrap();
    let high_gcode = String::from_utf8(
        format_gcode(&narrow_rectangular_gap_fill_pipeline(&high), &high).unwrap(),
    )
    .unwrap();

    assert!(low_gcode.contains(";SPEED:print:gap_fill:2.6,0.35:1500"));
    assert!(high_gcode.contains(";SPEED:print:gap_fill:2.6,0.35:2700"));
    assert_delta_eq(
        first_extrusion_delta(&high_gcode, "gap_fill"),
        first_extrusion_delta(&low_gcode, "gap_fill") * 3.0,
    );
}

#[test]
fn gap_infill_speed_zero_disables_generated_wall_gap_fill() {
    let options = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 0
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_gap_fills()[0].paths().is_empty());
    assert!(!gcode.contains(";PRINT_PATH:gap_fill:"));
}

#[test]
fn filter_out_gap_fill_removes_generated_gap_fill_before_outputs() {
    let options = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 45,
        "filter_out_gap_fill": 3.0
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert!(
        !pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::GapFill)
    );
    assert!(!gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(
        !region
            .extras()
            .paths()
            .iter()
            .any(|path| path.role() == crate::ExtrusionRole::GapFill)
    );
}

#[test]
fn gap_fill_target_topbottom_generates_solid_surface_gap_fill() {
    let options = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "topbottom",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert_eq!(region.extras().len(), 1);
    assert!(gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(gcode.contains(";SPEED:print:gap_fill:2.6,0.35:2700"));
}

#[test]
fn gap_fill_target_default_nowhere_does_not_add_solid_surface_gap_fill() {
    let options = options(json!({
        "wall_loops": 0,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_gap_fills()[0].paths().is_empty());
    assert!(!gcode.contains(";PRINT_PATH:gap_fill:"));
}

#[test]
fn gap_fill_target_filter_out_gap_fill_removes_solid_surface_gap_fill_before_outputs() {
    let options = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "topbottom",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45,
        "filter_out_gap_fill": 3.0
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert!(!gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(region.extras().is_empty());
}

#[test]
fn gap_fill_target_everywhere_adds_internal_solid_gap_fill() {
    let topbottom = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "topbottom",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45
    }));
    let everywhere = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "everywhere",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45
    }));
    let topbottom_pipeline = narrow_layers_pipeline(&topbottom, 5);
    let everywhere_pipeline = narrow_layers_pipeline(&everywhere, 5);

    assert!(topbottom_pipeline.layer_gap_fills()[1].paths().is_empty());
    assert_eq!(everywhere_pipeline.layer_gap_fills()[1].paths().len(), 1);
}

#[test]
fn gap_fill_target_bridge_no_support_skips_solid_surface_gap_fill_on_bridge_layer() {
    let options = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "everywhere",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 0,
        "gap_infill_speed": 45,
        "bridge_no_support": true
    }));
    let pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(pipeline.layer_gap_fills()[1].paths().is_empty());
    assert!(layer.contains(";PRINT_PATH:bridge:"));
    assert!(!layer.contains(";PRINT_PATH:gap_fill:"));
}

#[test]
fn counterbore_sacrificial_layer_keeps_solid_surface_gap_fill_on_bridge_layer() {
    let default = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "everywhere",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 0,
        "gap_infill_speed": 45,
        "bridge_no_support": true
    }));
    let sacrificial = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "everywhere",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 0,
        "gap_infill_speed": 45,
        "bridge_no_support": true,
        "counterbore_hole_bridging": "sacrificiallayer"
    }));

    let default_pipeline = unsupported_narrow_second_layer_pipeline(&default);
    let sacrificial_pipeline = unsupported_narrow_second_layer_pipeline(&sacrificial);
    let sacrificial_gcode =
        String::from_utf8(format_gcode(&sacrificial_pipeline, &sacrificial).unwrap()).unwrap();
    let sacrificial_layer = layer_section(&sacrificial_gcode, 1);

    assert!(default_pipeline.layer_gap_fills()[1].paths().is_empty());
    assert_eq!(sacrificial_pipeline.layer_gap_fills()[1].paths().len(), 1);
    assert!(sacrificial_layer.contains(";PRINT_PATH:gap_fill:"));
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_diameter": [2.0],
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn narrow_layers_pipeline(options: &SliceOptions, layer_count: usize) -> crate::SlicingPipeline {
    contour_layers_pipeline_from_layers_for_tests(options, vec![narrow_contour(0.0); layer_count])
}

fn unsupported_narrow_second_layer_pipeline(options: &SliceOptions) -> crate::SlicingPipeline {
    contour_layers_pipeline_from_layers_for_tests(
        options,
        vec![narrow_contour(0.0), narrow_contour(10.0)],
    )
}

fn narrow_contour(x: f64) -> Vec<crate::Contour> {
    vec![crate::Contour::new(vec![
        crate::Point2::new(x, 0.0),
        crate::Point2::new(x + 3.0, 0.0),
        crate::Point2::new(x + 3.0, 0.7),
        crate::Point2::new(x, 0.7),
    ])]
}

fn layer_section(gcode: &str, layer_index: usize) -> &str {
    let marker = format!(";LAYER:{layer_index}");
    let start = gcode.find(&marker).unwrap();
    let next = gcode[start + marker.len()..]
        .find(";LAYER:")
        .map(|offset| start + marker.len() + offset)
        .unwrap_or(gcode.len());
    &gcode[start..next]
}

fn first_extrusion_delta(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let target = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&target) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing {role} extrusion");
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}
