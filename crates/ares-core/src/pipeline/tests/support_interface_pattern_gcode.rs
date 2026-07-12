use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn grid_changes_support_interface_gcode_coordinates_and_line_count() {
    let single = output_for_support(json!({}));
    let grid = output_for_support(json!({ "support_interface_pattern": "grid" }));

    assert_ne!(grid, single);
    assert_eq!(count_role(&single, "support_material_interface"), 3);
    assert_eq!(count_role(&grid, "support_material_interface"), 5);
    assert!(!single.contains("X1 Y1.9"));
    assert!(grid.contains("X1 Y1.9"));
}

#[test]
fn concentric_changes_support_interface_gcode_to_closed_loop_coordinates() {
    let single = output_for_support(json!({}));
    let concentric = output_for_support(json!({ "support_interface_pattern": "concentric" }));

    assert_ne!(concentric, single);
    assert_eq!(count_role(&single, "support_material_interface"), 3);
    assert_eq!(count_role(&concentric, "support_material_interface"), 4);
    assert!(!single.contains("X3 Y1"));
    assert!(!single.contains("X3 Y2"));
    assert!(concentric.contains("X3 Y1"));
    assert!(concentric.contains("X3 Y2"));
}

#[test]
fn rectilinear_interlaced_changes_support_interface_gcode_to_diagonal_coordinates() {
    let single = output_for_support(json!({}));
    let interlaced = output_for_support(json!({
        "support_interface_pattern": "rectilinear_interlaced"
    }));

    assert_ne!(interlaced, single);
    assert_eq!(count_role(&single, "support_material_interface"), 3);
    assert_eq!(count_role(&interlaced, "support_material_interface"), 2);
    assert!(!single.contains("X2.727 Y2"));
    assert!(interlaced.contains("X2.727 Y2"));
}

#[test]
fn auto_zero_top_z_distance_changes_support_interface_gcode_to_concentric_coordinates() {
    let default_auto = output_for_support(json!({ "support_interface_pattern": "auto" }));
    let zero_gap_auto = output_for_support(json!({
        "support_interface_pattern": "auto",
        "support_top_z_distance": 0.0
    }));

    assert_ne!(zero_gap_auto, default_auto);
    assert_eq!(count_role(&default_auto, "support_material_interface"), 3);
    assert_eq!(count_role(&zero_gap_auto, "support_material_interface"), 4);
    assert!(!default_auto.contains("X3 Y2"));
    assert!(zero_gap_auto.contains("X3 Y2"));
}

fn output_for_support(extra: Value) -> String {
    let options = options(extra);
    let paths = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![support_rectangle()])],
        &options,
    )
    .unwrap();
    let layers = vec![Layer::new(0, 0.2, 0.2)];
    let layer_contours = vec![LayerContours::new(0, 0.2, Vec::new())];
    let print = build_print_domain(&layers, &layer_contours, &paths).unwrap();
    let layer_toolpath_moves = generate_toolpath_moves(&paths);
    let layer_extrusion_moves = generate_extrusion_moves(
        &layers,
        &layer_toolpath_moves,
        options.extrusion_options().unwrap(),
    )
    .unwrap();
    let layer_speed_moves =
        generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let total_extrusion_mm = layer_extrusion_moves[0].total_extrusion_mm();
    let pipeline = SlicingPipeline {
        options: options.clone(),
        model: Model::new(InputFormat::Stl, Vec::new()),
        layers,
        layer_slices: vec![LayerSlice::new(0, 0.2, Vec::new())],
        layer_contours,
        layer_perimeters: vec![LayerPerimeters::new(0, 0.2, Vec::new())],
        layer_gap_fills: vec![LayerGapFills::new(0, 0.2, Vec::new())],
        layer_infills: vec![LayerInfills::new(0, 0.2, Vec::new())],
        layer_skirts: vec![LayerSkirts::new(0, 0.2, Vec::new())],
        layer_brims: vec![LayerBrims::new(0, 0.2, Vec::new())],
        layer_print_paths: paths,
        print,
        layer_toolpath_moves,
        layer_extrusion_moves,
        layer_speed_moves,
        diagnostics: crate::PipelineDiagnostics {
            completed_stages: vec![
                crate::PipelineStage::Model,
                crate::PipelineStage::Layers,
                crate::PipelineStage::Segments,
                crate::PipelineStage::Contours,
                crate::PipelineStage::Perimeters,
                crate::PipelineStage::Infills,
                crate::PipelineStage::Skirts,
                crate::PipelineStage::Brims,
                crate::PipelineStage::PrintPaths,
                crate::PipelineStage::Moves,
                crate::PipelineStage::Extrusions,
                crate::PipelineStage::Speeds,
            ],
            input_format: InputFormat::Stl,
            triangle_count: 0,
            layer_count: 1,
            total_segment_count: 0,
            total_contour_count: 0,
            total_perimeter_count: 0,
            total_infill_count: 0,
            total_skirt_path_count: 0,
            total_brim_path_count: 0,
            total_print_path_count: 1,
            total_toolpath_move_count: 2,
            total_extrusion_move_count: 2,
            total_speed_move_count: 2,
            total_extrusion_mm,
            empty_layer_count: 1,
            option_count: options.values().len(),
        },
    };

    String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn support_rectangle() -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn count_role(output: &str, role: &str) -> usize {
    let prefix = format!(";EXTRUSION:print:{role}:");
    output
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
