use super::*;
use serde_json::json;

#[test]
fn constructed_internal_bridge_path_reaches_gcode_comments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "bridge_speed": 20,
        "internal_bridge_speed": "150%",
        "internal_bridge_flow": 0.5,
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let layers = vec![crate::Layer::new(0, 0.2, 0.2)];
    let layer_contours = vec![crate::LayerContours::new(0, 0.2, Vec::new())];
    let layer_print_paths = vec![crate::LayerPrintPaths::new(
        0,
        0.2,
        vec![
            crate::PrintPath::new(
                PrintPathRole::InternalBridge,
                vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
            )
            .unwrap(),
        ],
    )];
    let layer_toolpath_moves = crate::generate_toolpath_moves(&layer_print_paths);
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_extrusion_moves =
        crate::generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        crate::generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let total_extrusion_mm = layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum();
    let print = crate::build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let pipeline = SlicingPipeline {
        options: options.clone(),
        model: crate::Model::new(InputFormat::Stl, Vec::new()),
        layers,
        layer_slices: vec![crate::LayerSlice::new(0, 0.2, Vec::new())],
        layer_contours,
        layer_perimeters: vec![crate::LayerPerimeters::new(0, 0.2, Vec::new())],
        layer_gap_fills: vec![crate::LayerGapFills::new(0, 0.2, Vec::new())],
        layer_infills: vec![crate::LayerInfills::new(0, 0.2, Vec::new())],
        layer_skirts: vec![crate::LayerSkirts::new(0, 0.2, Vec::new())],
        layer_brims: vec![crate::LayerBrims::new(0, 0.2, Vec::new())],
        layer_print_paths,
        print,
        layer_toolpath_moves,
        layer_extrusion_moves,
        layer_speed_moves,
        diagnostics: PipelineDiagnostics {
            completed_stages: vec![
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
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PRINT_PATH:internal_bridge:"));
    assert!(gcode.contains(";EXTRUSION:print:internal_bridge:"));
    assert!(gcode.contains(";SPEED:print:internal_bridge:1,0:1800"));
    assert!(gcode.contains(";MOVE:print:internal_bridge:"));
}

#[test]
fn ordinary_rectangular_pipeline_keeps_generated_infill_sparse() {
    let pipeline = rectangular_pipeline(&SliceOptions::default());

    assert_eq!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .filter(|path| path.role() == PrintPathRole::InternalBridge)
            .count(),
        0
    );
}
