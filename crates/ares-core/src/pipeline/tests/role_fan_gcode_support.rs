use super::*;

pub(super) fn role_sequence_output(options: &SliceOptions) -> String {
    let pipeline = role_sequence_pipeline(options);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
}

pub(super) fn role_sequence_pipeline(options: &SliceOptions) -> SlicingPipeline {
    role_sequence_pipeline_with_roles(
        options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::InternalBridge,
            PrintPathRole::SparseInfill,
        ],
    )
}

pub(super) fn role_sequence_output_with_roles(
    options: &SliceOptions,
    roles: &[PrintPathRole],
) -> String {
    let pipeline = role_sequence_pipeline_with_roles(options, roles);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn role_sequence_pipeline_with_roles(
    options: &SliceOptions,
    roles: &[PrintPathRole],
) -> SlicingPipeline {
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
        roles
            .iter()
            .enumerate()
            .map(|(index, role)| {
                crate::PrintPath::new(
                    *role,
                    vec![
                        Point2::new(index as f64, 0.0),
                        Point2::new(index as f64 + 1.0, 0.0),
                    ],
                )
                .unwrap()
            })
            .collect(),
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
    let total_toolpath_move_count = layer_toolpath_moves[0].moves().len();
    let total_extrusion_move_count = layer_extrusion_moves[0].moves().len();
    let total_speed_move_count = layer_speed_moves[0].moves().len();
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
            total_print_path_count: roles.len(),
            total_toolpath_move_count,
            total_extrusion_move_count,
            total_speed_move_count,
            total_extrusion_mm,
            empty_layer_count: 1,
            option_count: options.values().len(),
        },
    }
}

pub(super) fn options(extra: serde_json::Value) -> SliceOptions {
    let mut base = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    })
    .as_object()
    .unwrap()
    .clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

pub(super) fn fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 ") || *line == "M126" || *line == "M127")
        .collect()
}

pub(super) fn assert_line_before(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

pub(super) fn assert_line_after(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index > second_index,
        "{first_index} !> {second_index}"
    );
}

pub(super) fn assert_line_before_last(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().rposition(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

pub(super) fn assert_line_before_last_prefix(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().rposition(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .rposition(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

pub(super) fn without_option_count(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("; option_count = "))
        .collect::<Vec<_>>()
        .join("\n")
}
