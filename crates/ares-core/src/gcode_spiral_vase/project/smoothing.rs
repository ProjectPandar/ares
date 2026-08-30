use super::{Movement, format_axis, format_e, set_word, word_value};
use crate::{
    Point2,
    gcode_spiral_vase::{distance, interpolate, nearest_point_on_polyline},
};

pub(super) struct SmoothRequest<'a> {
    pub(super) normal: String,
    pub(super) movement: Movement,
    pub(super) progress: f64,
    pub(super) previous_layer: &'a [Point2],
    pub(super) maximum_distance: f64,
    pub(super) minimum_segment_length: f64,
}

pub(super) fn smooth_line(
    request: SmoothRequest<'_>,
    last_emitted: &mut Option<Point2>,
    current_layer: &mut Vec<Point2>,
) -> Option<String> {
    let SmoothRequest {
        mut normal,
        movement,
        progress,
        previous_layer,
        maximum_distance,
        minimum_segment_length,
    } = request;
    let original = Point2::new(movement.target_x, movement.target_y);
    current_layer.push(original);
    let Some(nearest) = nearest_point_on_polyline(previous_layer, original) else {
        *last_emitted = Some(original);
        return Some(normal);
    };
    if distance(nearest, original) >= maximum_distance {
        *last_emitted = Some(original);
        return Some(normal);
    }
    let target = interpolate(nearest, original, progress);
    let adjusted_length = distance(last_emitted.unwrap_or(original), target);
    if adjusted_length < minimum_segment_length {
        return None;
    }
    normal = set_word(&normal, 'X', format_axis(target.x()));
    normal = set_word(&normal, 'Y', format_axis(target.y()));
    if let Some(e) = word_value(&normal, 'E') {
        normal = set_word(
            &normal,
            'E',
            format_e(e * adjusted_length / movement.xy_distance),
        );
    }
    *last_emitted = Some(target);
    Some(normal)
}
