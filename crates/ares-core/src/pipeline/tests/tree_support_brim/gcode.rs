use super::support::{empty_contours, layer, options, support_rect};
use crate::{
    InputFormat, Layer, LayerBrims, LayerExtrusionMoves, LayerGapFills, LayerInfills,
    LayerPerimeters, LayerSkirts, LayerSlice, LayerSpeedMoves, LayerToolpathMoves, Model,
    PrintPathRole, SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::json;

#[test]
fn manual_tree_support_brim_changes_support_gcode_coordinates() {
    let raw = output_for_tree_brim(json!({
        "support_type": "tree(auto)",
        "tree_support_auto_brim": false,
        "tree_support_brim_width": 0.0
    }));
    let brimmed = output_for_tree_brim(json!({
        "support_type": "tree(auto)",
        "tree_support_auto_brim": false,
        "tree_support_brim_width": 1.25
    }));

    assert_ne!(raw, brimmed);
    assert!(raw.contains("X2 Y0"));
    assert!(contains_expanded_g1_move(&brimmed));
    assert!(brimmed.contains(";PRINT_PATH:support_material:"));
    assert!(brimmed.contains(";EXTRUSION:print:support_material:"));
}

#[test]
fn auto_tree_support_brim_changes_support_gcode_coordinates() {
    let raw = output_for_tree_brim(json!({
        "support_type": "normal(auto)",
        "tree_support_auto_brim": true,
        "tree_support_brim_width": 0.0
    }));
    let brimmed = output_for_tree_brim(json!({
        "support_type": "tree(auto)",
        "tree_support_auto_brim": true,
        "tree_support_brim_width": 0.0
    }));

    assert_ne!(raw, brimmed);
    assert!(raw.contains("X2 Y0"));
    assert!(contains_auto_expanded_g1_move(&brimmed));
    assert!(brimmed.contains(";PRINT_PATH:support_material:"));
    assert!(brimmed.contains(";EXTRUSION:print:support_material:"));
}

fn contains_expanded_g1_move(output: &str) -> bool {
    output
        .lines()
        .any(|line| line.starts_with("G1 X-1.25 Y-1.25") || line.starts_with("G1 X3.25 Y-1.25"))
}

fn contains_auto_expanded_g1_move(output: &str) -> bool {
    output
        .lines()
        .any(|line| line.starts_with("G1 X-2 Y-2") || line.starts_with("G1 X4 Y-2"))
}

fn output_for_tree_brim(extra: serde_json::Value) -> String {
    let options = options(extra);
    let layers = vec![Layer::new(0, 0.2, 0.2)];
    let layer_contours = empty_contours(1);
    let paths = crate::finalize_print_paths_with_layer_contours(
        vec![layer(
            0,
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
    layer_contours: Vec<crate::LayerContours>,
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
        layer_slices: vec![LayerSlice::new(0, 0.2, Vec::new())],
        layer_contours: input.layer_contours,
        layer_perimeters: vec![LayerPerimeters::new(0, 0.2, Vec::new())],
        layer_gap_fills: vec![LayerGapFills::new(0, 0.2, Vec::new())],
        layer_infills: vec![LayerInfills::new(0, 0.2, Vec::new())],
        layer_skirts: vec![LayerSkirts::new(0, 0.2, Vec::new())],
        layer_brims: vec![LayerBrims::new(0, 0.2, Vec::new())],
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
