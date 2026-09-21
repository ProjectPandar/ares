use super::*;
use crate::{ExtrusionMove, PrintPathRole};

#[test]
fn spiral_starting_flow_ratio_uses_print_xy_only_but_observes_travel_start() {
    let mut state = SpiralVaseLayerState {
        starting_flow_ratio: 0.0,
        total_xy: 10.0,
        printed_xy: 0.0,
        previous_point: None,
        transition_in: true,
        transition_out:
            crate::gcode_spiral_vase_transition::SpiralVaseTransitionOutState::disabled(),
        smooth_xy: false,
        max_xy_smoothing: 0.0,
        previous_layer_points: Vec::new(),
        current_layer_points: Vec::new(),
        last_emitted_point: None,
    };

    assert_eq!(
        state.adjusted_move(command(ToolpathMoveKind::Travel, 100.0, 0.0, 0.0, 0.0)),
        adjusted(100.0, 0.0, 0.0, 0.0)
    );
    assert_eq!(
        state.adjusted_move(command(ToolpathMoveKind::Print, 103.0, 4.0, 0.0, 1.0)),
        adjusted(103.0, 4.0, 0.5, -0.5)
    );
    assert_eq!(
        state.adjusted_move(command(ToolpathMoveKind::Print, 106.0, 8.0, 0.5, 1.5)),
        adjusted(106.0, 8.0, 1.5, 0.0)
    );
}

#[test]
fn xy_smoothing_uses_previous_original_layer_points_and_scales_e() {
    let mut run = SpiralVaseRunState {
        config: SpiralVaseConfig {
            starting_flow_ratio: 0.0,
            finishing_flow_ratio: 0.0,
            transition_layer_index: None,
            smooth_xy: true,
            max_xy_smoothing: 10.0,
        },
        previous_layer_points: Vec::new(),
    };
    let first_layer = layer_moves(0.0);
    let mut first = run.layer_state(0, false, &first_layer);
    first.adjusted_move(command(ToolpathMoveKind::Travel, 0.0, 0.0, 0.0, 0.0));
    first.adjusted_move(command(ToolpathMoveKind::Print, 10.0, 0.0, 0.0, 1.0));
    first.adjusted_move(command(ToolpathMoveKind::Print, 20.0, 0.0, 1.0, 2.0));
    run.previous_layer_points = first.take_current_layer_points();

    let second_layer = layer_moves(4.0);
    let mut second = run.layer_state(1, false, &second_layer);
    second.adjusted_move(command(ToolpathMoveKind::Travel, 0.0, 4.0, 0.0, 0.0));
    let first_smoothed =
        second.adjusted_move(command(ToolpathMoveKind::Print, 10.0, 4.0, 0.0, 1.0));
    let second_smoothed =
        second.adjusted_move(command(ToolpathMoveKind::Print, 20.0, 4.0, 1.0, 2.0));

    assert_eq!(first_smoothed.point, Point2::new(10.0, 2.0));
    assert!((first_smoothed.e_position - 1.0198039027185568).abs() < 1e-12);
    assert_eq!(second_smoothed.point, Point2::new(20.0, 4.0));
    assert!((second_smoothed.e_position - 2.019803902718557).abs() < 1e-12);
    let original_points = vec![Point2::new(10.0, 4.0), Point2::new(20.0, 4.0)];
    assert_eq!(second.take_current_layer_points(), original_points);
}

fn command(
    kind: ToolpathMoveKind,
    x: f64,
    y: f64,
    current_e: f64,
    target_e: f64,
) -> SpiralVaseMoveCommand {
    SpiralVaseMoveCommand {
        kind,
        point: Point2::new(x, y),
        current_e,
        target_e,
    }
}

fn layer_moves(y: f64) -> LayerExtrusionMoves {
    LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, y),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, y),
                Some(1.0),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(20.0, y),
                Some(2.0),
            ),
        ],
        2.0,
    )
}

fn adjusted(x: f64, y: f64, e_position: f64, e_offset_delta: f64) -> SpiralVaseAdjustedMove {
    SpiralVaseAdjustedMove {
        point: Point2::new(x, y),
        e_position,
        e_offset_delta,
    }
}
