use crate::{
    Contour, Point2, PrintPathRole, SliceOptions, ToolpathMoveKind,
    extrusions::ExplicitExtrusionSegment, pipeline::test_support::contour_layers_pipeline,
};
use serde_json::json;

#[test]
fn thin_wall_bead_width_override_reaches_paths_moves_and_extrusions() {
    let options = options(json!({
        "detect_thin_wall": true,
        "min_feature_size": 100,
        "initial_layer_min_bead_width": 200,
        "min_bead_width": 175
    }));
    let pipeline = thin_wall_pipeline(&options);
    let extrusion_options = options.extrusion_options().unwrap();

    let first_print_path = thin_wall_print_path(&pipeline, 0);
    let later_print_path = thin_wall_print_path(&pipeline, 1);
    assert_eq!(first_print_path.effective_line_width_mm(), Some(0.8));
    assert_eq!(
        round_6(later_print_path.effective_line_width_mm().unwrap()),
        0.7
    );

    let first_print_move = thin_wall_print_move(&pipeline, 0);
    let later_print_move = thin_wall_print_move(&pipeline, 1);
    assert_eq!(first_print_move.effective_line_width_mm(), Some(0.8));
    assert_eq!(
        round_6(later_print_move.effective_line_width_mm().unwrap()),
        0.7
    );

    let first_extrusion_move = thin_wall_extrusion_move(&pipeline, 0);
    let later_extrusion_move = thin_wall_extrusion_move(&pipeline, 1);
    assert_eq!(first_extrusion_move.effective_line_width_mm(), Some(0.8));
    assert_eq!(
        round_6(later_extrusion_move.effective_line_width_mm().unwrap()),
        0.7
    );
    assert_eq!(
        thin_wall_extrusion_delta(&pipeline, 0),
        round_6(
            extrusion_options
                .extrusion_delta_for_segment_with_width(ExplicitExtrusionSegment {
                    role: PrintPathRole::ExternalPerimeter,
                    layer_height: 0.2,
                    is_first_layer: true,
                    line_width: 0.8,
                    line_length_mm: 2.2,
                })
                .unwrap()
        )
    );
    assert_eq!(
        thin_wall_extrusion_delta(&pipeline, 1),
        round_6(
            extrusion_options
                .extrusion_delta_for_segment_with_width(ExplicitExtrusionSegment {
                    role: PrintPathRole::ExternalPerimeter,
                    layer_height: 0.2,
                    is_first_layer: false,
                    line_width: 0.7,
                    line_length_mm: 2.2,
                })
                .unwrap()
        )
    );
}

fn thin_wall_print_path(pipeline: &crate::SlicingPipeline, layer_id: usize) -> &crate::PrintPath {
    pipeline.layer_print_paths()[layer_id]
        .paths()
        .iter()
        .find(|path| {
            path.role() == PrintPathRole::ExternalPerimeter
                && path.points() == thin_wall_centerline()
        })
        .unwrap()
}

fn thin_wall_print_move(
    pipeline: &crate::SlicingPipeline,
    layer_id: usize,
) -> &crate::ToolpathMove {
    pipeline.layer_toolpath_moves()[layer_id]
        .moves()
        .iter()
        .find(|move_| {
            move_.kind() == ToolpathMoveKind::Print && move_.point() == Point2::new(2.6, 0.35)
        })
        .unwrap()
}

fn thin_wall_extrusion_move(
    pipeline: &crate::SlicingPipeline,
    layer_id: usize,
) -> &crate::ExtrusionMove {
    pipeline.layer_extrusion_moves()[layer_id]
        .moves()
        .iter()
        .find(|move_| {
            move_.kind() == ToolpathMoveKind::Print && move_.point() == Point2::new(2.6, 0.35)
        })
        .unwrap()
}

fn thin_wall_extrusion_delta(pipeline: &crate::SlicingPipeline, layer_id: usize) -> f64 {
    let moves = pipeline.layer_extrusion_moves()[layer_id].moves();
    let index = moves
        .iter()
        .position(|move_| {
            move_.kind() == ToolpathMoveKind::Print && move_.point() == Point2::new(2.6, 0.35)
        })
        .unwrap();
    let previous_e = moves[..index]
        .iter()
        .rev()
        .find_map(crate::ExtrusionMove::e_position)
        .unwrap_or_else(|| {
            layer_id
                .checked_sub(1)
                .and_then(|previous_layer_id| {
                    pipeline.layer_extrusion_moves()[previous_layer_id]
                        .moves()
                        .iter()
                        .rev()
                        .find_map(crate::ExtrusionMove::e_position)
                })
                .unwrap_or(0.0)
        });
    round_6(moves[index].e_position().unwrap() - previous_e)
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
        "wall_loops": 4,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "sparse_infill_density": 0,
        "minimum_sparse_infill_area": 0
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn thin_wall_pipeline(options: &SliceOptions) -> crate::SlicingPipeline {
    contour_layers_pipeline(
        options,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(3.0, 0.7),
            Point2::new(0.0, 0.7),
        ])],
        2,
    )
}

fn thin_wall_centerline() -> [Point2; 2] {
    [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
