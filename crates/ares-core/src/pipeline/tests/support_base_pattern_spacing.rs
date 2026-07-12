use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn omitted_spacing_uses_orca_default_pitch_for_closed_support_rectangle() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({}),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[[Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)]],
    );
}

#[test]
fn zero_spacing_uses_support_material_width_as_pitch() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({ "support_base_pattern_spacing": 0.0 }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
}

#[test]
fn larger_spacing_changes_support_material_gcode_coordinates_and_line_count() {
    let dense_output = output_for_support(json!({ "support_base_pattern_spacing": 0.0 }));
    let sparse_output = output_for_support(json!({ "support_base_pattern_spacing": "1.25" }));

    assert_ne!(sparse_output, dense_output);
    assert_eq!(count_role(&dense_output, "support_material"), 3);
    assert_eq!(count_role(&sparse_output, "support_material"), 1);
    assert!(dense_output.contains("X1 Y1.4"));
    assert!(dense_output.contains("X1 Y1.8"));
    assert!(!sparse_output.contains("X1 Y1.4"));
    assert!(!sparse_output.contains("X1 Y1.8"));
}

#[test]
fn generated_lines_preserve_source_metadata_and_extrusion_role() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07)
        .with_closed(true);
    let finalized = finalize(vec![source], json!({}));

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.extrusion_role(), Some(PrintPathRole::SupportMaterial));
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(!path.is_closed());
    }
}

#[test]
fn zero_top_interface_layers_converts_interface_before_base_spacing() {
    let finalized = finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_interface_top_layers": 0,
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
    assert_eq!(finalized[0].paths()[0].extrusion_role(), None);
}

#[test]
fn non_rectangular_non_closed_non_support_and_remaining_interface_paths_are_unchanged() {
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let open_rectangle =
        PrintPath::new(PrintPathRole::SupportMaterial, rectangle_points()).unwrap();
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);
    let finalized = finalize(
        vec![
            triangle.clone(),
            open_rectangle.clone(),
            solid_rectangle.clone(),
        ],
        json!({}),
    );

    assert_eq!(
        finalized[0].paths(),
        [triangle, open_rectangle, solid_rectangle]
    );
}

#[test]
fn remaining_support_material_interface_paths_are_unchanged_by_base_spacing() {
    let interface = support_rectangle(PrintPathRole::SupportMaterialInterface)
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_closed(true);
    let finalized = finalize(
        vec![interface.clone()],
        json!({
            "support_base_pattern_spacing": 0.0,
            "support_ironing": true
        }),
    );

    assert_eq!(finalized[0].paths()[0], interface);
}

#[test]
fn invalid_spacing_values_reach_slice_error() {
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
            vec![LayerPrintPaths::new(
                1,
                0.4,
                vec![support_rectangle(PrintPathRole::SupportMaterial)],
            )],
            &options(json!({ "support_base_pattern_spacing": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_base_pattern_spacing"));
    }
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points())
        .unwrap()
        .with_closed(true)
}

fn rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(1.0, 1.0),
        Point2::new(3.0, 1.0),
        Point2::new(3.0, 2.0),
        Point2::new(1.0, 2.0),
    ]
}

fn assert_support_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.points(), *points);
        assert!(!path.is_closed());
    }
}

fn output_for_support(extra: Value) -> String {
    let options = options(extra);
    let paths = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            1,
            0.4,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options,
    )
    .unwrap();
    let layers = vec![Layer::new(1, 0.2, 0.4)];
    let layer_contours = vec![LayerContours::new(1, 0.4, Vec::new())];
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
        layer_slices: vec![LayerSlice::new(1, 0.4, Vec::new())],
        layer_contours,
        layer_perimeters: vec![LayerPerimeters::new(1, 0.4, Vec::new())],
        layer_gap_fills: vec![LayerGapFills::new(1, 0.4, Vec::new())],
        layer_infills: vec![LayerInfills::new(1, 0.4, Vec::new())],
        layer_skirts: vec![LayerSkirts::new(1, 0.4, Vec::new())],
        layer_brims: vec![LayerBrims::new(1, 0.4, Vec::new())],
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
