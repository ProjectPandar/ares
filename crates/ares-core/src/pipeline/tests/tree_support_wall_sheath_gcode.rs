use super::tree_support_brim::support::{layer, options, support_rect};
use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerExtrusionMoves, LayerGapFills,
    LayerInfills, LayerPerimeters, LayerSkirts, LayerSlice, LayerSpeedMoves, LayerToolpathMoves,
    Model, PrintPathRole, SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::json;

#[test]
fn tree_support_wall_count_changes_support_material_gcode_coordinates() {
    let raw = output_for_tree_support_wall_count(0);
    let sheathed = output_for_tree_support_wall_count(1);

    assert_ne!(sheathed, raw);
    assert!(raw.contains(";PRINT_PATH:support_material:"));
    assert!(sheathed.contains(";PRINT_PATH:support_material:"));
    assert!(raw.contains("X0 Y0"));
    assert!(raw.contains("X2 Y0"));
    assert!(!raw.contains("X0.16 Y0.16"));
    assert!(sheathed.contains("X0.16 Y0.16"));
}

fn output_for_tree_support_wall_count(wall_count: u32) -> String {
    let options = options(json!({
        "tree_support_wall_count": wall_count,
        "support_base_pattern_spacing": 0.0
    }));
    let layers = vec![Layer::new(1, 0.2, 0.4)];
    let layer_contours = vec![LayerContours::new(1, 0.4, Vec::new())];
    let paths = crate::finalize_print_paths_with_layer_contours(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        &options,
        &layer_contours,
    )
    .unwrap();
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
    let pipeline = pipeline(PipelineInput {
        options: options.clone(),
        layers,
        layer_contours,
        paths,
        print,
        layer_toolpath_moves,
        layer_extrusion_moves,
        layer_speed_moves,
    });

    String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap()
}

struct PipelineInput {
    options: SliceOptions,
    layers: Vec<Layer>,
    layer_contours: Vec<LayerContours>,
    paths: Vec<crate::LayerPrintPaths>,
    print: crate::Print,
    layer_toolpath_moves: Vec<LayerToolpathMoves>,
    layer_extrusion_moves: Vec<LayerExtrusionMoves>,
    layer_speed_moves: Vec<LayerSpeedMoves>,
}

fn pipeline(input: PipelineInput) -> SlicingPipeline {
    SlicingPipeline {
        options: input.options,
        model: Model::new(InputFormat::Stl, Vec::new()),
        layers: input.layers,
        layer_slices: vec![LayerSlice::new(1, 0.4, Vec::new())],
        layer_contours: input.layer_contours,
        layer_perimeters: vec![LayerPerimeters::new(1, 0.4, Vec::new())],
        layer_gap_fills: vec![LayerGapFills::new(1, 0.4, Vec::new())],
        layer_infills: vec![LayerInfills::new(1, 0.4, Vec::new())],
        layer_skirts: vec![LayerSkirts::new(1, 0.4, Vec::new())],
        layer_brims: vec![LayerBrims::new(1, 0.4, Vec::new())],
        layer_print_paths: input.paths,
        print: input.print,
        layer_toolpath_moves: input.layer_toolpath_moves,
        layer_extrusion_moves: input.layer_extrusion_moves,
        layer_speed_moves: input.layer_speed_moves,
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
            total_toolpath_move_count: 0,
            total_extrusion_mm: 0.0,
            total_extrusion_move_count: 0,
            total_speed_move_count: 0,
            empty_layer_count: 0,
            option_count: 0,
        },
    }
}
