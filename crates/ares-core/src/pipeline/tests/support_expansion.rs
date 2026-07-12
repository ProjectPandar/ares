use crate::{
    InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters,
    LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions, SlicingPipeline, build_print_domain, gcode::format_gcode,
    generate_extrusion_moves, generate_speed_moves, generate_toolpath_moves,
};
use serde_json::{Value, json};

#[test]
fn positive_support_expansion_expands_support_material_rectangle() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07)
        .with_closed(true);
    let finalized = finalize(vec![source], json!({ "support_expansion": 0.5 }));

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    assert_eq!(finalized[0].paths().len(), 1);
    let expanded = &finalized[0].paths()[0];
    assert_eq!(expanded.role(), PrintPathRole::SupportMaterial);
    assert_eq!(expanded.extrusion_role(), None);
    assert_eq!(
        expanded.points(),
        [Point2::new(0.5, 0.5), Point2::new(3.5, 0.5)]
    );
    assert_eq!(expanded.effective_layer_height_mm(), Some(0.13));
    assert_eq!(expanded.unsupported_span_mm(), Some(2.5));
    assert_eq!(expanded.seam_gap_mm(), 0.07);
    assert!(!expanded.is_closed());
}

#[test]
fn negative_support_expansion_shrinks_support_interface_before_spacing_conversion() {
    let source = support_rectangle(PrintPathRole::SupportMaterialInterface)
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07)
        .with_closed(true);
    let finalized = finalize(vec![source], json!({ "support_expansion": "-0.25" }));

    assert_eq!(finalized[0].paths().len(), 2);
    for shrunk in finalized[0].paths() {
        assert_eq!(shrunk.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(
            shrunk.extrusion_role(),
            Some(PrintPathRole::SupportMaterialInterface)
        );
        assert_eq!(shrunk.effective_layer_height_mm(), Some(0.13));
        assert_eq!(shrunk.unsupported_span_mm(), Some(2.5));
        assert_eq!(shrunk.seam_gap_mm(), 0.07);
        assert!(!shrunk.is_closed());
    }
    assert_eq!(
        finalized[0].paths()[0].points(),
        [Point2::new(1.25, 1.25), Point2::new(1.25, 1.75)]
    );
    assert_eq!(
        finalized[0].paths()[1].points(),
        [Point2::new(2.15, 1.25), Point2::new(2.15, 1.75)]
    );
}

#[test]
fn collapsing_support_expansion_drops_support_rectangle() {
    let source = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(2.0, 1.0),
            Point2::new(2.0, 1.5),
            Point2::new(1.0, 1.5),
        ],
    )
    .unwrap()
    .with_closed(true);
    let finalized = finalize(vec![source], json!({ "support_expansion": -1.0 }));

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn zero_or_omitted_support_expansion_keeps_material_and_applies_default_interface_spacing() {
    let material = support_rectangle(PrintPathRole::SupportMaterial).with_closed(true);
    let material_line = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
    )
    .unwrap();
    let interface = support_rectangle(PrintPathRole::SupportMaterialInterface)
        .with_closed(true)
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface);
    let interface_lines = [
        PrintPath::new(
            PrintPathRole::SupportMaterialInterface,
            vec![Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
        )
        .unwrap()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        PrintPath::new(
            PrintPathRole::SupportMaterialInterface,
            vec![Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
        )
        .unwrap()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        PrintPath::new(
            PrintPathRole::SupportMaterialInterface,
            vec![Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
        )
        .unwrap()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
    ];

    assert_eq!(
        finalize(vec![material.clone(), interface.clone()], json!({}))[0].paths(),
        [
            material_line.clone(),
            interface_lines[0].clone(),
            interface_lines[1].clone(),
            interface_lines[2].clone()
        ]
    );
    assert_eq!(
        finalize(
            vec![material.clone(), interface.clone()],
            json!({ "support_expansion": 0.0 })
        )[0]
        .paths(),
        [
            material_line,
            interface_lines[0].clone(),
            interface_lines[1].clone(),
            interface_lines[2].clone()
        ]
    );
}

#[test]
fn support_expansion_leaves_non_rectangular_and_non_support_paths_unchanged() {
    let support_triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);
    let finalized = finalize(
        vec![support_triangle.clone(), solid_rectangle.clone()],
        json!({ "support_expansion": 0.5 }),
    );

    assert_eq!(finalized[0].paths(), [support_triangle, solid_rectangle]);
}

#[test]
fn invalid_support_expansion_values_reach_slice_error() {
    for value in [
        json!("NaN"),
        json!("inf"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 1 }),
        json!(true),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                1,
                0.4,
                vec![support_rectangle(PrintPathRole::SupportMaterial)],
            )],
            &options(json!({ "support_expansion": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_expansion"));
    }
}

#[test]
fn support_ironing_uses_expanded_support_interface_geometry() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterialInterface).with_closed(true)],
        json!({
            "support_expansion": 0.5,
            "support_ironing": true,
            "support_ironing_pattern": "concentric",
            "support_ironing_spacing": 1.0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 2);
    for path in finalized[0].paths() {
        assert_eq!(
            path.points(),
            [
                Point2::new(0.5, 0.5),
                Point2::new(3.5, 0.5),
                Point2::new(3.5, 2.5),
                Point2::new(0.5, 2.5),
            ]
        );
    }
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert_eq!(finalized[0].paths()[1].role(), PrintPathRole::Ironing);
}

#[test]
fn support_expansion_changes_emitted_support_gcode_span() {
    let base = output_for_support(json!({ "support_expansion": 0.0 }));
    let expanded = output_for_support(json!({ "support_expansion": 0.5 }));

    assert_ne!(expanded, base);
    assert!(expanded.contains("X0.5 Y0.5"));
    assert!(expanded.contains("X3.5 Y0.5"));
    assert!(!base.contains("X0.5 Y0.5"));
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points()).unwrap()
}

fn rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(1.0, 1.0),
        Point2::new(3.0, 1.0),
        Point2::new(3.0, 2.0),
        Point2::new(1.0, 2.0),
    ]
}

fn output_for_support(extra: Value) -> String {
    let options = options(extra);
    let paths = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterial).with_closed(true)],
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
