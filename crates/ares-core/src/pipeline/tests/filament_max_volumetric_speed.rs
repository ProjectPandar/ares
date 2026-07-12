use crate::{
    ExtrusionMove, InputFormat, Layer, LayerBrims, LayerContours, LayerExtrusionMoves,
    LayerGapFills, LayerInfills, LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice,
    LayerToolpathMoves, Model, PipelineDiagnostics, PipelineStage, Point2, PrintPath,
    PrintPathRole, SliceOptions, SlicingPipeline, ToolpathMove, ToolpathMoveKind,
    build_print_domain, gcode::format_gcode, generate_speed_moves,
    pipeline::test_support::rectangular_pipeline,
};
use serde_json::json;

#[test]
fn filament_max_volumetric_speed_caps_print_feedrate() {
    let uncapped = options(json!({ "filament_max_volumetric_speed": 0.0 }));
    let capped = options(json!({ "filament_max_volumetric_speed": 1.0 }));

    let uncapped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&uncapped), &uncapped).unwrap())
            .unwrap();
    let capped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&capped), &capped).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print"),
        6000.0
    );
    assert!(
        first_speed_feedrate(&capped_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print")
    );
}

#[test]
fn filament_max_volumetric_speed_does_not_cap_travel_feedrate() {
    let capped = options(json!({
        "filament_max_volumetric_speed": 0.1,
        "travel_speed": 120
    }));
    let gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&capped), &capped).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&gcode, "external_perimeter", "travel"),
        7200.0
    );
}

#[test]
fn filament_flow_ratio_lowers_volumetric_capped_feedrate() {
    let base = options(json!({ "filament_max_volumetric_speed": 1.0 }));
    let higher_flow = options(json!({
        "filament_max_volumetric_speed": 1.0,
        "filament_flow_ratio": 2.0
    }));

    let base_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&base), &base).unwrap()).unwrap();
    let higher_flow_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&higher_flow), &higher_flow).unwrap())
            .unwrap();

    assert!(
        first_speed_feedrate(&higher_flow_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&base_gcode, "external_perimeter", "print")
    );
}

#[test]
fn adaptive_volumetric_speed_lowers_print_feedrate_from_coefficients() {
    let disabled = options(json!({
        "filament_max_volumetric_speed": 10.0,
        "filament_adaptive_volumetric_speed": false,
        "volumetric_speed_coefficients": ["0 0 0 0 0 1"]
    }));
    let enabled = options(json!({
        "filament_max_volumetric_speed": 10.0,
        "filament_adaptive_volumetric_speed": true,
        "volumetric_speed_coefficients": ["0 0 0 0 0 1"]
    }));

    let disabled_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&disabled), &disabled).unwrap())
            .unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&enabled), &enabled).unwrap())
            .unwrap();

    assert!(
        first_speed_feedrate(&enabled_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&disabled_gcode, "external_perimeter", "print")
    );
}

#[test]
fn max_volumetric_extrusion_rate_slope_lowers_later_print_feedrate() {
    let disabled = options(json!({
        "filament_max_volumetric_speed": 0.0,
        "max_volumetric_extrusion_rate_slope": 0.0
    }));
    let enabled = options(json!({
        "filament_max_volumetric_speed": 0.0,
        "max_volumetric_extrusion_rate_slope": 1.0
    }));

    let disabled_gcode = String::from_utf8(
        format_gcode(
            &external_perimeter_rate_smoothing_pipeline(&disabled),
            &disabled,
        )
        .unwrap(),
    )
    .unwrap();
    let enabled_gcode = String::from_utf8(
        format_gcode(
            &external_perimeter_rate_smoothing_pipeline(&enabled),
            &enabled,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(
        last_speed_feedrate(&enabled_gcode, "external_perimeter", "print")
            < last_speed_feedrate(&disabled_gcode, "external_perimeter", "print")
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_speed": 100,
        "initial_layer_speed": 100,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn external_perimeter_rate_smoothing_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let layers = vec![Layer::new(0, 0.2, 0.2)];
    let layer_slices = vec![LayerSlice::new(0, 0.2, Vec::new())];
    let layer_contours = vec![LayerContours::new(0, 0.2, Vec::new())];
    let layer_perimeters = vec![LayerPerimeters::new(0, 0.2, Vec::new())];
    let layer_gap_fills = vec![LayerGapFills::new(0, 0.2, Vec::new())];
    let layer_infills = vec![LayerInfills::new(0, 0.2, Vec::new())];
    let layer_skirts = vec![LayerSkirts::new(0, 0.2, Vec::new())];
    let layer_brims = vec![LayerBrims::new(0, 0.2, Vec::new())];
    let layer_print_paths = vec![LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::ExternalPerimeter,
                vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(10.0, 0.0),
                    Point2::new(20.0, 0.0),
                ],
            )
            .unwrap(),
        ],
    )];
    let print = build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = vec![LayerToolpathMoves::new(
        0,
        0.2,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(20.0, 0.0),
            ),
        ],
    )];
    let layer_extrusion_moves = vec![LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(20.0, 0.0),
                Some(1.1),
            ),
        ],
        1.1,
    )];
    let layer_speed_moves =
        generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());

    SlicingPipeline {
        options: options.clone(),
        model: Model::new(InputFormat::Stl, Vec::new()),
        layers,
        layer_slices,
        layer_contours,
        layer_perimeters,
        layer_gap_fills,
        layer_infills,
        layer_skirts,
        layer_brims,
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
            total_toolpath_move_count: 3,
            total_extrusion_move_count: 3,
            total_speed_move_count: 3,
            total_extrusion_mm: 1.1,
            empty_layer_count: 1,
            option_count: options.values().len(),
        },
    }
}

fn first_speed_feedrate(gcode: &str, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    gcode
        .lines()
        .find_map(|line| {
            line.starts_with(&target)
                .then(|| line.rsplit(':').next().unwrap().parse().unwrap())
        })
        .unwrap_or_else(|| panic!("missing {kind} {role} speed"))
}

fn last_speed_feedrate(gcode: &str, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    gcode
        .lines()
        .filter(|line| line.starts_with(&target))
        .map(|line| line.rsplit(':').next().unwrap().parse().unwrap())
        .next_back()
        .unwrap_or_else(|| panic!("missing {kind} {role} speed"))
}
