use super::*;
use serde_json::json;

#[test]
fn unsupported_second_layer_overhang_reaches_gcode_comments_and_speed() {
    let options = options(json!({
        "bridge_speed": 20,
        "slowdown_for_curled_perimeters": true,
        "overhang_4_4_speed": 30
    }));
    let pipeline = unsupported_second_layer_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::OverhangPerimeter)
    );
    assert!(gcode.contains(";PRINT_PATH:overhang_perimeter:"));
    assert!(gcode.contains(";SPEED:print:overhang_perimeter:"));
    assert!(gcode.contains(";EXTRUSION:print:overhang_perimeter:"));
    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "overhang_perimeter", "print"),
        1800.0
    );
}

#[test]
fn disabled_overhang_speed_preserves_bridge_fallback_feedrate() {
    let options = options(json!({
        "bridge_speed": 20,
        "enable_overhang_speed": false,
        "overhang_4_4_speed": 30
    }));
    let gcode = String::from_utf8(
        crate::gcode::format_gcode(&unsupported_second_layer_pipeline(&options), &options).unwrap(),
    )
    .unwrap();

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "overhang_perimeter", "print"),
        1200.0
    );
}

#[test]
fn disabling_slowdown_for_curled_perimeters_uses_bridge_feedrate_for_full_overhang() {
    let options = options(json!({
        "bridge_speed": 20,
        "slowdown_for_curled_perimeters": false,
        "overhang_4_4_speed": 30
    }));
    let gcode = String::from_utf8(
        crate::gcode::format_gcode(&unsupported_second_layer_pipeline(&options), &options).unwrap(),
    )
    .unwrap();

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "overhang_perimeter", "print"),
        1200.0
    );
}

#[test]
fn unsupported_span_reaches_overhang_extrusion_moves() {
    let options = options(json!({}));
    let pipeline = unsupported_second_layer_pipeline(&options);

    let path_span = pipeline.layer_print_paths()[1]
        .paths()
        .iter()
        .find(|path| path.role() == PrintPathRole::OverhangPerimeter)
        .and_then(|path| path.unsupported_span_mm());
    assert_eq!(path_span, Some(4.0));

    let move_span = pipeline.layer_extrusion_moves()[1]
        .moves()
        .iter()
        .find(|move_| {
            move_.kind() == ToolpathMoveKind::Print
                && move_.role() == PrintPathRole::OverhangPerimeter
        })
        .and_then(|move_| move_.unsupported_span_mm());
    assert_eq!(move_span, Some(4.0));
}

#[test]
fn lower_overhang_speed_band_reaches_generated_gcode() {
    let options = options(json!({
        "bridge_speed": 20,
        "outer_wall_line_width": 20.0,
        "overhang_1_4_speed": 15,
        "overhang_4_4_speed": 30
    }));
    let gcode = String::from_utf8(
        crate::gcode::format_gcode(&unsupported_second_layer_pipeline(&options), &options).unwrap(),
    )
    .unwrap();

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "overhang_perimeter", "print"),
        900.0
    );
}

#[test]
fn overhang_flow_ratio_changes_gcode_extrusion_only_when_gate_is_enabled() {
    let disabled = options(json!({
        "set_other_flow_ratios": false,
        "overhang_flow_ratio": 0.5
    }));
    let enabled = options(json!({
        "set_other_flow_ratios": true,
        "overhang_flow_ratio": 0.5
    }));
    let disabled_gcode = String::from_utf8(
        crate::gcode::format_gcode(&unsupported_second_layer_pipeline(&disabled), &disabled)
            .unwrap(),
    )
    .unwrap();
    let enabled_gcode = String::from_utf8(
        crate::gcode::format_gcode(&unsupported_second_layer_pipeline(&enabled), &enabled).unwrap(),
    )
    .unwrap();

    assert_delta_eq(
        first_layer_extrusion_delta(&enabled_gcode, 1, "overhang_perimeter"),
        first_layer_extrusion_delta(&disabled_gcode, 1, "overhang_perimeter") * 0.5,
    );
}

fn unsupported_second_layer_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let layers = vec![
        crate::Layer::new(0, 0.2, 0.2),
        crate::Layer::new(1, 0.2, 0.4),
    ];
    let layer_slices = layers
        .iter()
        .map(|layer| crate::LayerSlice::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_contours = vec![
        crate::LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        crate::LayerContours::new(1, 0.4, vec![rectangle(10.0, 0.0, 14.0, 4.0)]),
    ];
    let layer_perimeters =
        crate::generate_perimeters(&layer_contours, options.perimeter_options().unwrap()).unwrap();
    let layer_gap_fills = layer_contours
        .iter()
        .map(|layer| crate::LayerGapFills::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_infills =
        crate::generate_infills(&layers, &layer_contours, options.infill_options().unwrap())
            .unwrap();
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_skirts = layers
        .iter()
        .map(|layer| crate::LayerSkirts::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_brims = layers
        .iter()
        .map(|layer| crate::LayerBrims::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_print_paths = crate::generate_print_paths(
        crate::PrintPathInput::new(
            &layer_skirts,
            &layer_brims,
            &layer_perimeters,
            &layer_gap_fills,
            &layer_infills,
        )
        .with_layer_contours(&layer_contours),
        options.shell_layer_options().unwrap(),
        false,
        options.bridge_options().unwrap().bridge_no_support(),
    )
    .unwrap();
    let print = crate::build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = crate::generate_toolpath_moves(&layer_print_paths);
    let layer_extrusion_moves =
        crate::generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        crate::generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let total_extrusion_mm = layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum();
    let diagnostics = test_diagnostics(
        options,
        &layers,
        &layer_slices,
        &layer_contours,
        &layer_perimeters,
        &layer_infills,
        &layer_skirts,
        &layer_brims,
        &layer_print_paths,
        &layer_toolpath_moves,
        &layer_extrusion_moves,
        &layer_speed_moves,
        total_extrusion_mm,
    );

    SlicingPipeline {
        options: options.clone(),
        model: crate::Model::new(InputFormat::Stl, Vec::new()),
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
        diagnostics,
    }
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> crate::Contour {
    crate::Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}

fn layer_speed_feedrate(gcode: &str, layer_id: usize, role: &str, kind: &str) -> f64 {
    let mut current_layer = None;
    let target = format!(";SPEED:{kind}:{role}:");
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if current_layer == Some(layer_id) && line.starts_with(&target) {
            return line.rsplit(':').next().unwrap().parse().unwrap();
        }
    }
    panic!("missing layer {layer_id} {kind} {role} speed");
}

fn first_layer_extrusion_delta(gcode: &str, layer_id: usize, role: &str) -> f64 {
    let mut current_layer = None;
    let mut previous_e = 0.0;
    let target = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if current_layer == Some(layer_id) && line.starts_with(&target) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing layer {layer_id} {role} extrusion");
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}

#[allow(clippy::too_many_arguments)]
fn test_diagnostics(
    options: &SliceOptions,
    layers: &[crate::Layer],
    layer_slices: &[crate::LayerSlice],
    layer_contours: &[crate::LayerContours],
    layer_perimeters: &[crate::LayerPerimeters],
    layer_infills: &[crate::LayerInfills],
    layer_skirts: &[crate::LayerSkirts],
    layer_brims: &[crate::LayerBrims],
    layer_print_paths: &[crate::LayerPrintPaths],
    layer_toolpath_moves: &[crate::LayerToolpathMoves],
    layer_extrusion_moves: &[crate::LayerExtrusionMoves],
    layer_speed_moves: &[crate::LayerSpeedMoves],
    total_extrusion_mm: f64,
) -> PipelineDiagnostics {
    PipelineDiagnostics {
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
        layer_count: layers.len(),
        total_segment_count: layer_slices
            .iter()
            .map(|layer| layer.segments().len())
            .sum(),
        total_contour_count: layer_contours
            .iter()
            .map(|layer| layer.contours().len())
            .sum(),
        total_perimeter_count: layer_perimeters
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_infill_count: layer_infills.iter().map(|layer| layer.paths().len()).sum(),
        total_skirt_path_count: layer_skirts.iter().map(|layer| layer.paths().len()).sum(),
        total_brim_path_count: layer_brims.iter().map(|layer| layer.paths().len()).sum(),
        total_print_path_count: layer_print_paths
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_toolpath_move_count: layer_toolpath_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_extrusion_move_count: layer_extrusion_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_speed_move_count: layer_speed_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_extrusion_mm,
        empty_layer_count: layer_contours
            .iter()
            .filter(|layer| layer.contours().is_empty())
            .count(),
        option_count: options.values().len(),
    }
}
