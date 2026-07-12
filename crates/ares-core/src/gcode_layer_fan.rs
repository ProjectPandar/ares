use crate::{LayerSpeedMoves, Point2, SpeedMove, ToolpathMoveKind, options::PartCoolingFanRamp};

pub(crate) fn baseline_speed(
    ramp: PartCoolingFanRamp,
    layer_index: usize,
    layer_speed_moves: &LayerSpeedMoves,
) -> Option<u8> {
    match layer_print_time_s(layer_speed_moves.moves()) {
        Some(layer_time_s) => ramp.speed_for_layer_time(layer_index, Some(layer_time_s)),
        None => ramp.speed_for_layer(layer_index),
    }
}

fn layer_print_time_s(moves: &[SpeedMove]) -> Option<f64> {
    let mut last_point = None;
    let mut had_print = false;
    let mut total = 0.0;
    for move_ in moves {
        let start = last_point.unwrap_or(move_.point());
        if move_.kind() == ToolpathMoveKind::Print {
            had_print = true;
        }
        if had_print {
            let length = distance(start, move_.point());
            if length > 0.0 && move_.speed_mm_s() > 0.0 {
                total += length / move_.speed_mm_s();
            }
        }
        last_point = Some(move_.point());
    }
    had_print.then_some(total)
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}
