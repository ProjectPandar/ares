use crate::{
    Contour, InputFormat, Layer, LayerBrims, LayerContours, LayerExtrusionMoves, LayerGapFills,
    LayerInfills, LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice, LayerSpeedMoves,
    LayerToolpathMoves, Model, Point2, PrintPath, PrintPathRole, SliceOptions, SlicingPipeline,
    build_print_domain, gcode::format_gcode, generate_extrusion_moves, generate_speed_moves,
    generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn support_object_first_layer_gap_changes_layer_zero_support_coordinates() {
    let zero = output_for_first_layer_gap(0.0);
    let positive = output_for_first_layer_gap(0.5);

    assert_ne!(positive, zero);
    assert!(zero.contains("X0 Y3"));
    assert!(!zero.contains("X0 Y3.5"));
    assert!(positive.contains("X0 Y3.5"));
}

fn output_for_first_layer_gap(distance: f64) -> String {
    let options = options(json!({
        "support_object_xy_distance": 0.0,
        "support_object_first_layer_gap": distance,
        "support_remove_small_overhang": false
    }));
    let layers = vec![Layer::new(0, 0.2, 0.2)];
    let layer_contours = vec![LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 3.0),
            Point2::new(1.0, 3.0),
        ])],
    )];
    let paths = crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![
                PrintPath::new(
                    PrintPathRole::SupportMaterial,
                    vec![
                        Point2::new(0.0, 0.0),
                        Point2::new(4.0, 0.0),
                        Point2::new(4.0, 4.0),
                        Point2::new(0.0, 4.0),
                    ],
                )
                .unwrap()
                .with_closed(true),
            ],
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
    paths: Vec<LayerPrintPaths>,
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
            total_contour_count: 1,
            total_perimeter_count: 0,
            total_infill_count: 0,
            total_skirt_path_count: 0,
            total_brim_path_count: 0,
            total_print_path_count: 0,
            total_toolpath_move_count: 0,
            total_extrusion_move_count: 0,
            total_speed_move_count: 0,
            total_extrusion_mm: 0.0,
            empty_layer_count: 0,
            option_count: 0,
        },
    }
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "enable_support": true,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().expect("test options must be an object") {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
