use super::*;
use serde_json::json;

#[test]
fn thick_bridges_changes_external_bridge_extrusion_comment() {
    let thin = bridge_output(false);
    let thick = bridge_output(true);

    let thin_extrusion = bridge_extrusion_line(&thin);
    let thick_extrusion = bridge_extrusion_line(&thick);

    assert_ne!(thin_extrusion, thick_extrusion);
    assert_eq!(thin_extrusion, ";EXTRUSION:print:bridge:1,0:0.022732");
    assert_eq!(thick_extrusion, ";EXTRUSION:print:bridge:1,0:0.04");
}

#[test]
fn thick_internal_bridges_changes_internal_bridge_extrusion_comment() {
    let thin = internal_bridge_output(false);
    let thick = internal_bridge_output(true);

    let thin_extrusion = internal_bridge_extrusion_line(&thin);
    let thick_extrusion = internal_bridge_extrusion_line(&thick);

    assert_ne!(thin_extrusion, thick_extrusion);
    assert_eq!(
        thin_extrusion,
        ";EXTRUSION:print:internal_bridge:1,0:0.022732"
    );
    assert_eq!(thick_extrusion, ";EXTRUSION:print:internal_bridge:1,0:0.04");
}

fn bridge_output(thick_bridges: bool) -> String {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "thick_bridges": thick_bridges
    }))
    .unwrap();
    let pipeline = bridge_pipeline(&options);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn internal_bridge_output(thick_internal_bridges: bool) -> String {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "thick_internal_bridges": thick_internal_bridges
    }))
    .unwrap();
    let pipeline = bridge_role_pipeline(&options, PrintPathRole::InternalBridge);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn bridge_pipeline(options: &SliceOptions) -> SlicingPipeline {
    bridge_role_pipeline(options, PrintPathRole::Bridge)
}

fn bridge_role_pipeline(options: &SliceOptions, role: PrintPathRole) -> SlicingPipeline {
    let layers = vec![crate::Layer::new(0, 0.2, 0.2)];
    let layer_slices = vec![crate::LayerSlice::new(0, 0.2, Vec::new())];
    let layer_contours = vec![crate::LayerContours::new(0, 0.2, Vec::new())];
    let layer_perimeters = vec![crate::LayerPerimeters::new(0, 0.2, Vec::new())];
    let layer_gap_fills = vec![crate::LayerGapFills::new(0, 0.2, Vec::new())];
    let layer_infills = vec![crate::LayerInfills::new(0, 0.2, Vec::new())];
    let layer_skirts = vec![crate::LayerSkirts::new(0, 0.2, Vec::new())];
    let layer_brims = vec![crate::LayerBrims::new(0, 0.2, Vec::new())];
    let layer_print_paths = vec![crate::LayerPrintPaths::new(
        0,
        0.2,
        vec![
            crate::PrintPath::new(role, vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)])
                .unwrap(),
        ],
    )];
    let print = crate::build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = crate::generate_toolpath_moves(&layer_print_paths);
    let layer_extrusion_moves = crate::generate_extrusion_moves(
        &layers,
        &layer_toolpath_moves,
        options.extrusion_options().unwrap(),
    )
    .unwrap();
    let layer_speed_moves =
        crate::generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let total_extrusion_mm = layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum();

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
    }
}

fn bridge_extrusion_line(output: &str) -> &str {
    output
        .lines()
        .find(|line| line.starts_with(";EXTRUSION:print:bridge:"))
        .unwrap()
}

fn internal_bridge_extrusion_line(output: &str) -> &str {
    output
        .lines()
        .find(|line| line.starts_with(";EXTRUSION:print:internal_bridge:"))
        .unwrap()
}
