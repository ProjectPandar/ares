use crate::{ExtrusionMove, Point2, PrintPathRole, SpeedOptions, ToolpathMoveKind};

pub(super) fn speeds_for_layer(
    moves: &[ExtrusionMove],
    options: &SpeedOptions,
    base_speed: impl Fn(&ExtrusionMove) -> f64,
) -> Vec<f64> {
    let mut speeds = moves.iter().map(base_speed).collect::<Vec<_>>();
    let mut span_start = None;
    for index in 0..moves.len() {
        if moves[index].kind() == ToolpathMoveKind::Travel {
            if let Some(start) = span_start {
                apply_span_speed(moves, &mut speeds, start, index, options);
            }
            span_start = Some(index);
        }
    }
    if let Some(start) = span_start {
        apply_span_speed(moves, &mut speeds, start, moves.len(), options);
    }
    speeds
}

fn apply_span_speed(
    moves: &[ExtrusionMove],
    speeds: &mut [f64],
    start: usize,
    end: usize,
    options: &SpeedOptions,
) {
    let print_range = start + 1..end;
    if print_range.is_empty() {
        return;
    }
    if !print_range.clone().all(|index| {
        moves[index].kind() == ToolpathMoveKind::Print
            && moves[index].role() == PrintPathRole::ExternalPerimeter
    }) {
        return;
    }
    let mut previous = moves[start].point();
    let mut length = 0.0;
    for index in print_range.clone() {
        let point = moves[index].point();
        length += distance(previous, point);
        previous = point;
    }
    if length <= max_small_perimeter_length(options.small_perimeter_threshold_mm()) {
        for index in print_range {
            speeds[index] = options.small_perimeter_speed_mm_s();
        }
    }
}

fn max_small_perimeter_length(threshold_mm: f64) -> f64 {
    threshold_mm * 2.0 * std::f64::consts::PI
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}
