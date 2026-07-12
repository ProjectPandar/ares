use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn invalid_bottom_spacing_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 0.5 }),
        json!(true),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(1, 0.4, vec![support_rectangle()])],
            &options(json!({ "support_bottom_interface_spacing": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_bottom_interface_spacing"));
    }
}

#[test]
fn valid_bottom_spacing_values_are_accepted_across_current_modes() {
    for extra in [
        json!({ "support_bottom_interface_spacing": 0.0 }),
        json!({ "support_bottom_interface_spacing": "1.25" }),
        json!({
            "support_interface_top_layers": 0,
            "support_interface_bottom_layers": 1,
            "support_bottom_interface_spacing": 0.75
        }),
        json!({
            "support_interface_top_layers": 2,
            "support_interface_bottom_layers": 1,
            "support_bottom_interface_spacing": "0.75"
        }),
        json!({
            "support_ironing": true,
            "support_ironing_spacing": 0.5,
            "support_bottom_interface_spacing": 0.25
        }),
    ] {
        finalize(vec![support_rectangle()], extra);
    }
}

#[test]
fn bottom_only_interface_uses_bottom_spacing_for_paths_and_gcode() {
    let dense = json!({
        "support_interface_top_layers": 0,
        "support_interface_bottom_layers": 1,
        "support_interface_spacing": 0.5,
        "support_bottom_interface_spacing": 0.0
    });
    let sparse = json!({
        "support_interface_top_layers": 0,
        "support_interface_bottom_layers": 1,
        "support_interface_spacing": 0.5,
        "support_bottom_interface_spacing": 1.25
    });

    let dense_paths = finalize(vec![support_rectangle()], dense.clone());
    let sparse_paths = finalize(vec![support_rectangle()], sparse.clone());
    let dense_gcode = output_for_single_path(dense);
    let sparse_gcode = output_for_single_path(sparse);

    assert_ne!(dense_paths, sparse_paths);
    assert_eq!(dense_paths[0].paths().len(), 6);
    assert_eq!(sparse_paths[0].paths().len(), 2);
    assert_ne!(dense_gcode, sparse_gcode);
    assert!(dense_gcode.contains("X1.4 Y1"));
    assert!(!sparse_gcode.contains("X1.4 Y1"));
    assert!(sparse_gcode.contains("X2.65 Y1"));
}

#[test]
fn support_interface_spacing_still_controls_current_generic_interface_geometry() {
    let dense = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_top_layers": 2,
            "support_interface_bottom_layers": 1,
            "support_interface_spacing": 0.0,
            "support_bottom_interface_spacing": 1.25
        }),
    );
    let sparse = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_top_layers": 2,
            "support_interface_bottom_layers": 1,
            "support_interface_spacing": 1.25,
            "support_bottom_interface_spacing": 0.0
        }),
    );

    assert_ne!(dense, sparse);
    assert_eq!(dense[0].paths().len(), 6);
    assert_eq!(sparse[0].paths().len(), 2);
}

#[test]
fn support_ironing_validates_bottom_spacing_before_preserving_solid_interface() {
    let err = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(1, 0.4, vec![support_rectangle()])],
        &options(json!({
            "support_ironing": true,
            "support_ironing_spacing": 0.5,
            "support_bottom_interface_spacing": -0.1
        })),
    )
    .unwrap_err();
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_bottom_interface_spacing"));

    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_ironing": true,
            "support_ironing_spacing": 0.5,
            "support_bottom_interface_spacing": 0.25
        }),
    );

    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert!(finalized[0].paths()[0].is_closed());
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
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

fn output_for_single_path(extra: Value) -> String {
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
    let total_extrusion_mm = layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
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
