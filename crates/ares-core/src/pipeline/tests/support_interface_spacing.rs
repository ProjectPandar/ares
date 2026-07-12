use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn omitted_spacing_uses_orca_default_pitch_for_closed_interface_rectangle() {
    let finalized = finalize(vec![support_rectangle()], json!({}));

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
            [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
        ],
    );
}

#[test]
fn zero_spacing_uses_support_interface_width_as_pitch() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_spacing": 0.0 }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.4, 1.0), Point2::new(1.4, 2.0)],
            [Point2::new(1.8, 1.0), Point2::new(1.8, 2.0)],
            [Point2::new(2.2, 1.0), Point2::new(2.2, 2.0)],
            [Point2::new(2.6, 1.0), Point2::new(2.6, 2.0)],
            [Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)],
        ],
    );
}

#[test]
fn larger_spacing_reduces_line_count_and_changes_gcode_coordinates() {
    let default_output = output_for_support(json!({}));
    let larger_output = output_for_support(json!({ "support_interface_spacing": "1.25" }));

    assert_ne!(larger_output, default_output);
    assert_eq!(count_role(&default_output, "support_material_interface"), 3);
    assert_eq!(count_role(&larger_output, "support_material_interface"), 2);
    assert!(default_output.contains("X1.9 Y1"));
    assert!(!larger_output.contains("X1.9 Y1"));
    assert!(larger_output.contains("X2.65 Y1"));
}

#[test]
fn generated_lines_preserve_source_metadata_and_extrusion_role() {
    let source = support_rectangle()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07)
        .with_closed(true);
    let finalized = finalize(vec![source], json!({}));

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(
            path.extrusion_role(),
            Some(PrintPathRole::SupportMaterialInterface)
        );
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(!path.is_closed());
    }
}

#[test]
fn zero_top_interface_layers_prevents_spacing_conversion() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_top_layers": 0 }),
    );

    assert_eq!(finalized[0].paths().len(), 1);
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterial
    );
    assert_eq!(
        finalized[0].paths()[0].points(),
        [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)]
    );
    assert!(!finalized[0].paths()[0].is_closed());
}

#[test]
fn support_ironing_forces_solid_interface_rectangle() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
    );

    assert_eq!(finalized[0].paths().len(), 4);
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert!(finalized[0].paths()[0].is_closed());
    assert_eq!(
        finalized[0].paths()[0].points(),
        [
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ]
    );
    assert_eq!(support_ironing_count(finalized[0].paths()), 3);
}

#[test]
fn non_rectangular_non_closed_and_non_interface_paths_are_unchanged() {
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let open_rectangle =
        PrintPath::new(PrintPathRole::SupportMaterialInterface, rectangle_points()).unwrap();
    let finalized = finalize(vec![triangle.clone(), open_rectangle.clone()], json!({}));

    assert_eq!(finalized[0].paths(), [triangle, open_rectangle]);
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
            vec![LayerPrintPaths::new(1, 0.4, vec![support_rectangle()])],
            &options(json!({ "support_interface_spacing": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_spacing"));
    }
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle() -> PrintPath {
    PrintPath::new(PrintPathRole::SupportMaterialInterface, rectangle_points())
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

fn assert_interface_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(path.points(), *points);
        assert!(!path.is_closed());
    }
}

fn support_ironing_count(paths: &[PrintPath]) -> usize {
    paths
        .iter()
        .filter(|path| path.role() == PrintPathRole::Ironing)
        .count()
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
