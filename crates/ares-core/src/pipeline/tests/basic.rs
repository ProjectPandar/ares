use super::*;

#[test]
fn pipeline_exposes_stage_artifacts_and_diagnostics() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "brim_width": 1.2,
        "infill_direction": 0,
        "is_infill_first": true,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let diagnostics = pipeline.diagnostics();

    assert_eq!(pipeline.model().format(), InputFormat::Stl);
    assert_eq!(pipeline.model().triangles().len(), 4);
    assert_eq!(pipeline.layers().len(), 2);
    assert_eq!(pipeline.layer_slices().len(), 2);
    assert_eq!(pipeline.layer_contours().len(), 2);
    assert_eq!(pipeline.layer_perimeters().len(), 2);
    assert_eq!(pipeline.layer_infills().len(), 2);
    assert_eq!(pipeline.layer_skirts().len(), 2);
    assert_eq!(pipeline.layer_skirts()[0].paths().len(), 1);
    assert_eq!(pipeline.layer_skirts()[1].paths().len(), 0);
    assert_eq!(pipeline.layer_brims().len(), 2);
    assert_eq!(pipeline.layer_brims()[0].paths().len(), 3);
    assert_eq!(pipeline.layer_brims()[1].paths().len(), 0);
    assert_eq!(pipeline.layer_print_paths().len(), 2);
    assert_eq!(pipeline.layer_toolpath_moves().len(), 2);
    assert_eq!(pipeline.layer_extrusion_moves().len(), 2);
    assert_eq!(pipeline.layer_speed_moves().len(), 2);
    assert_eq!(
        diagnostics.completed_stages(),
        &[
            PipelineStage::Model,
            PipelineStage::Layers,
            PipelineStage::Segments,
            PipelineStage::Contours,
            PipelineStage::Perimeters,
            PipelineStage::Infills,
            PipelineStage::Skirts,
            PipelineStage::Brims,
            PipelineStage::PrintPaths,
            PipelineStage::Moves,
            PipelineStage::Extrusions,
            PipelineStage::Speeds,
        ]
    );
    assert_eq!(diagnostics.input_format(), InputFormat::Stl);
    assert_eq!(diagnostics.triangle_count(), 4);
    assert_eq!(diagnostics.layer_count(), 2);
    assert_eq!(diagnostics.total_segment_count(), 8);
    assert_eq!(diagnostics.total_contour_count(), 2);
    assert_eq!(diagnostics.total_perimeter_count(), 2);
    assert_eq!(diagnostics.total_infill_count(), 6);
    assert_eq!(diagnostics.total_skirt_path_count(), 1);
    assert_eq!(diagnostics.total_brim_path_count(), 3);
    assert_eq!(diagnostics.total_print_path_count(), 12);
    assert_eq!(diagnostics.total_toolpath_move_count(), 42);
    assert_eq!(diagnostics.total_extrusion_move_count(), 42);
    assert_eq!(diagnostics.total_speed_move_count(), 42);
    assert!(diagnostics.total_extrusion_mm() > 0.0);
    assert_eq!(diagnostics.empty_layer_count(), 0);
    assert_eq!(diagnostics.option_count(), 10);
    let first_layer_paths = pipeline.layer_print_paths()[0].paths();
    let second_layer_paths = pipeline.layer_print_paths()[1].paths();
    assert_eq!(first_layer_paths[0].role(), PrintPathRole::Skirt);
    assert_eq!(first_layer_paths[1].role(), PrintPathRole::Brim);
    assert_eq!(second_layer_paths[0].role(), PrintPathRole::SparseInfill);
    assert_eq!(
        pipeline.layer_slices()[0].segments()[0],
        Segment2::new(Point2::new(-0.5, 0.0), Point2::new(0.0, -0.5))
    );
}

#[test]
fn pipeline_exposes_libslic3r_print_domain() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "brim_width": 1.2,
        "infill_direction": 0,
        "is_infill_first": true,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let print = pipeline.print();

    assert_eq!(print.objects().len(), 1);
    assert_eq!(print.regions().len(), 1);
    assert_eq!(print.objects()[0].layers().len(), pipeline.layers().len());
    let first_region = &print.objects()[0].layers()[0].regions()[0];
    assert_eq!(
        first_region.slices().len(),
        pipeline.layer_contours()[0].contours().len()
    );
    assert_eq!(first_region.perimeters().len(), 1);
    assert_eq!(first_region.fills().len(), 2);
    assert_eq!(first_region.extras().len(), 4);
    assert_eq!(pipeline.diagnostics().total_print_path_count(), 12);
}

#[test]
fn pipeline_preserves_input_stage_errors() {
    assert!(matches!(
        run_slicing_pipeline([], &SliceOptions::default()),
        Err(SliceError::EmptyInput)
    ));
    assert!(matches!(
        run_slicing_pipeline(b"not a model", &SliceOptions::default()),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn pipeline_preserves_contour_stage_errors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2
    }))
    .unwrap();

    let err = run_slicing_pipeline(single_sloped_triangle_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn rectangular_internal_perimeters_reach_gcode_role_comments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "inner_wall_speed": 35,
        "outer_wall_speed": 60,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let pipeline = rectangular_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PERIMETER:internal:"));
    assert!(gcode.contains(";PRINT_PATH:internal_perimeter:"));
    assert!(gcode.contains(";MOVE:print:internal_perimeter:"));
    assert!(gcode.contains(";EXTRUSION:print:internal_perimeter:"));
    assert!(gcode.contains(";SPEED:print:internal_perimeter:"));
}

#[test]
fn inner_wall_line_width_changes_internal_perimeter_gcode() {
    let narrow_inner: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.2,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let wide_inner: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.6,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let narrow_gcode = String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&narrow_inner), &narrow_inner).unwrap(),
    )
    .unwrap();
    let wide_gcode = String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&wide_inner), &wide_inner).unwrap(),
    )
    .unwrap();

    assert!(narrow_gcode.contains(";PERIMETER:internal:0.3,0.3 -> 3.7,0.3"));
    assert!(wide_gcode.contains(";PERIMETER:internal:0.5,0.5 -> 3.5,0.5"));
    assert_ne!(
        first_internal_extrusion_e(&narrow_gcode),
        first_internal_extrusion_e(&wide_gcode)
    );
}

#[test]
fn brim_flow_ratio_changes_brim_gcode_extrusion_delta() {
    let low_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_width": 0.8,
        "brim_flow_ratio": 0.5,
        "skirt_loops": 0,
        "sparse_infill_density": 0,
        "wall_loops": 0,
        "line_width": 0.4
    }))
    .unwrap();
    let high_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_width": 0.8,
        "brim_flow_ratio": 1.5,
        "skirt_loops": 0,
        "sparse_infill_density": 0,
        "wall_loops": 0,
        "line_width": 0.4
    }))
    .unwrap();
    let low_pipeline = rectangular_pipeline(&low_flow);
    let high_pipeline = rectangular_pipeline(&high_flow);

    assert_eq!(
        low_pipeline.diagnostics().total_brim_path_count(),
        high_pipeline.diagnostics().total_brim_path_count()
    );

    let low_gcode =
        String::from_utf8(crate::gcode::format_gcode(&low_pipeline, &low_flow).unwrap()).unwrap();
    let high_gcode =
        String::from_utf8(crate::gcode::format_gcode(&high_pipeline, &high_flow).unwrap()).unwrap();
    let low_delta = first_brim_extrusion_delta(&low_gcode);
    let high_delta = first_brim_extrusion_delta(&high_gcode);

    assert_eq!(
        (high_delta * 1_000_000.0).round(),
        (low_delta * 3.0 * 1_000_000.0).round()
    );
}

fn first_internal_extrusion_e(gcode: &str) -> &str {
    gcode
        .lines()
        .find_map(|line| line.strip_prefix(";EXTRUSION:print:internal_perimeter:"))
        .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
        .unwrap()
}

fn first_brim_extrusion_delta(gcode: &str) -> f64 {
    let mut previous_e = 0.0;
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(";EXTRUSION:print:brim:") {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing brim extrusion");
}
