use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

const COORD_EPSILON: f64 = 1e-9;

#[test]
fn absent_and_false_preserve_existing_interface_lines() {
    let omitted = finalize(vec![support_rectangle()], json!({}));
    let disabled = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_loop_pattern": false }),
    );

    let expected = [
        [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
        [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
        [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
    ];
    assert_interface_lines(omitted[0].paths(), &expected);
    assert_interface_lines(disabled[0].paths(), &expected);
}

#[test]
fn enabled_loop_pattern_prepends_closed_loop_before_interface_lines() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_loop_pattern": true }),
    );
    let paths = finalized[0].paths();

    assert_eq!(paths.len(), 4);
    assert_loop_path(&paths[0], &closed_rectangle_points());
    assert_interface_lines(
        &paths[1..],
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
            [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
        ],
    );
}

#[test]
fn grid_orders_loop_interface_angle_lines_then_base_angle_lines() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_loop_pattern": true,
            "support_interface_pattern": "grid"
        }),
    );
    let paths = finalized[0].paths();

    assert_eq!(paths.len(), 6);
    assert_loop_path(&paths[0], &closed_rectangle_points());
    assert_interface_lines(
        &paths[1..],
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
            [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.9), Point2::new(3.0, 1.9)],
        ],
    );
}

#[test]
fn loop_preserves_source_metadata_and_extrusion_role() {
    let source = support_rectangle()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize(
        vec![source],
        json!({ "support_interface_loop_pattern": true }),
    );
    let loop_path = &finalized[0].paths()[0];

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    assert_eq!(loop_path.role(), PrintPathRole::SupportMaterialInterface);
    assert_eq!(
        loop_path.extrusion_role(),
        Some(PrintPathRole::SupportMaterialInterface)
    );
    assert_eq!(loop_path.effective_layer_height_mm(), Some(0.13));
    assert_eq!(loop_path.unsupported_span_mm(), Some(2.5));
    assert_eq!(loop_path.seam_gap_mm(), 0.07);
    assert_points(loop_path.points(), &closed_rectangle_points());
    assert!(loop_path.is_closed());
}

#[test]
fn invalid_loop_pattern_values_reach_slice_error() {
    for value in [
        json!("true"),
        json!(1),
        json!(0.0),
        Value::Null,
        json!([]),
        json!({ "value": true }),
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(7, 1.6, vec![support_rectangle()])],
            &options(json!({ "support_interface_loop_pattern": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_loop_pattern"));
    }
}

#[test]
fn support_ironing_validates_loop_pattern_without_adding_loop() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_loop_pattern": true,
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
    );

    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert!(finalized[0].paths()[0].is_closed());
    assert_points(finalized[0].paths()[0].points(), &rectangle_points());
    assert_eq!(support_ironing_count(finalized[0].paths()), 3);

    let err = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![support_rectangle()])],
        &options(json!({
            "support_interface_loop_pattern": "true",
            "support_ironing": true
        })),
    )
    .unwrap_err();
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_interface_loop_pattern"));
}

#[test]
fn zero_top_interface_layers_prevents_loop_pattern_conversion() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_loop_pattern": true,
            "support_interface_top_layers": 0
        }),
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
fn loop_pattern_adds_support_interface_gcode_before_fill_lines() {
    let disabled = output_for_support(json!({ "support_interface_loop_pattern": false }));
    let enabled = output_for_support(json!({ "support_interface_loop_pattern": true }));

    assert_eq!(count_role(&disabled, "support_material_interface"), 3);
    assert_eq!(count_role(&enabled, "support_material_interface"), 7);
    assert_points(
        &support_interface_print_points(&enabled),
        &[
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
            Point2::new(1.0, 1.0),
            Point2::new(1.0, 2.0),
            Point2::new(1.9, 2.0),
            Point2::new(2.8, 2.0),
        ],
    );
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

fn closed_rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(1.0, 1.0),
        Point2::new(3.0, 1.0),
        Point2::new(3.0, 2.0),
        Point2::new(1.0, 2.0),
        Point2::new(1.0, 1.0),
    ]
}

fn assert_loop_path(path: &PrintPath, expected: &[Point2]) {
    assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
    assert_points(path.points(), expected);
    assert!(path.is_closed());
}

fn assert_interface_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_points(path.points(), points);
        assert!(!path.is_closed());
    }
}

fn assert_points(actual: &[Point2], expected: &[Point2]) {
    assert_eq!(actual.len(), expected.len());
    for (actual_point, expected_point) in actual.iter().zip(expected) {
        assert!(
            (actual_point.x() - expected_point.x()).abs() <= COORD_EPSILON,
            "x mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
        assert!(
            (actual_point.y() - expected_point.y()).abs() <= COORD_EPSILON,
            "y mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
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

fn support_interface_print_points(output: &str) -> Vec<Point2> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix(";EXTRUSION:print:support_material_interface:")?;
            let (coords, _) = rest.rsplit_once(':')?;
            let (x, y) = coords.split_once(',')?;
            Some(Point2::new(x.parse().unwrap(), y.parse().unwrap()))
        })
        .collect()
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
