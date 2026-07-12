use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn snug_merge_changes_support_material_gcode_after_base_spacing() {
    let default = output_for_support(json!({ "support_style": "default" }));
    let grid = output_for_support(json!({ "support_style": "grid" }));
    let snug = output_for_support(json!({ "support_style": "snug" }));

    assert_eq!(grid, default);
    assert_ne!(snug, default);
    assert_eq!(count_role(&default, "support_material"), 2);
    assert_eq!(count_role(&snug, "support_material"), 1);
    assert!(default.contains(";PRINT_PATH:support_material:0,0 -> 2,0"));
    assert!(default.contains(";PRINT_PATH:support_material:5,0 -> 7,0"));
    assert!(snug.contains(";PRINT_PATH:support_material:0,0 -> 7,0"));
}

fn output_for_support(extra: Value) -> String {
    let options = options(extra);
    let paths = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![
                support_rect(0.0, 0.0, 2.0, 2.0),
                support_rect(5.0, 0.0, 7.0, 2.0),
            ],
        )],
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
    let total_print_path_count = paths.iter().map(|layer| layer.paths().len()).sum();
    let total_toolpath_move_count = layer_toolpath_moves
        .iter()
        .map(|layer| layer.moves().len())
        .sum();
    let total_extrusion_move_count = layer_extrusion_moves
        .iter()
        .map(|layer| layer.moves().len())
        .sum();
    let total_speed_move_count = layer_speed_moves
        .iter()
        .map(|layer| layer.moves().len())
        .sum();
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
            total_print_path_count,
            total_toolpath_move_count,
            total_extrusion_move_count,
            total_speed_move_count,
            total_extrusion_mm,
            empty_layer_count: 1,
            option_count: options.values().len(),
        },
    };

    String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn count_role(output: &str, role: &str) -> usize {
    let prefix = format!(";EXTRUSION:print:{role}:");
    output
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

fn support_rect(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "support_base_pattern_spacing": 2.0,
        "support_ironing": true,
        "raft_first_layer_density": 10,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
