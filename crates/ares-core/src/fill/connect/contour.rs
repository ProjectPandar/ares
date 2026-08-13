use super::{
    scale::EPSILON_MM,
    types::{BoundaryContour, Intersection},
};
use crate::geometry::Point;

pub(super) fn contour_parameters(points: &[Point]) -> Vec<f64> {
    debug_assert!(!points.is_empty());
    let mut params = vec![0.0; points.len() + 1];
    for index in 1..points.len() {
        params[index] = params[index - 1] + point_distance(points[index - 1], points[index]);
        debug_assert!(params[index] > params[index - 1]);
    }
    params[points.len()] = params[points.len() - 1]
        + point_distance(points[0], *points.last().expect("a contour is nonempty"));
    debug_assert!(params[points.len()] > params[points.len() - 1]);
    params
}

pub(super) fn closed_contour_distance_ccw(param1: f64, param2: f64, contour_length: f64) -> f64 {
    debug_assert!((0.0..=contour_length).contains(&param1));
    debug_assert!((0.0..=contour_length).contains(&param2));
    let distance = param2 - param1;
    if distance < 0.0 {
        distance + contour_length
    } else {
        distance
    }
}

pub(super) fn closed_contour_distance_cw(param1: f64, param2: f64, contour_length: f64) -> f64 {
    closed_contour_distance_ccw(param2, param1, contour_length)
}

pub(super) fn path_length_along_contour_ccw(
    start: &Intersection,
    end: &Intersection,
    contour_length: f64,
) -> f64 {
    debug_assert_eq!(start.contour_index, end.contour_index);
    debug_assert!(!std::ptr::eq(start, end));
    closed_contour_distance_ccw(start.param, end.param, contour_length)
}

pub(super) fn lerp_truncating(start: Point, end: Point, t: f64) -> Point {
    debug_assert!((-EPSILON_MM..=1.0 + EPSILON_MM).contains(&t));
    Point::new(
        ((1.0 - t) * start.x() as f64 + t * end.x() as f64) as i64,
        ((1.0 - t) * start.y() as f64 + t * end.y() as f64) as i64,
    )
}

pub(super) fn append_full(
    output: &mut Vec<Point>,
    contour: &[Point],
    start_index: usize,
    end_index: usize,
    clockwise: bool,
) {
    debug_assert!(!output.is_empty() && output.last() == Some(&contour[start_index]));
    let mut index = adjacent_index(start_index, contour.len(), clockwise);
    while index != end_index {
        output.push(contour[index]);
        index = adjacent_index(index, contour.len(), clockwise);
    }
    output.push(contour[index]);
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source contour interpolation keeps both endpoints, direction, and limit explicit"
)]
pub(super) fn append_limited(
    output: &mut Vec<Point>,
    contour: &BoundaryContour,
    start_index: usize,
    end_index: usize,
    clockwise: bool,
    length_to_take: f64,
    scaled_epsilon: f64,
) -> f64 {
    debug_assert!(output.is_empty() || output.last() == Some(&contour.points[start_index]));
    debug_assert_eq!(contour.points.len() + 1, contour.params.len());
    debug_assert!(length_to_take > scaled_epsilon);
    let contour_length = *contour.params.last().expect("a contour has parameters");
    let start_param = contour.params[start_index];
    let mut index = adjacent_index(start_index, contour.points.len(), clockwise);
    let mut previous_index = start_index;
    let mut previous_length = 0.0;
    loop {
        let length = if clockwise {
            closed_contour_distance_cw(start_param, contour.params[index], contour_length)
        } else {
            closed_contour_distance_ccw(start_param, contour.params[index], contour_length)
        };
        if length >= length_to_take {
            let t = (length_to_take - previous_length) / (length - previous_length);
            output.push(lerp_truncating(
                contour.points[previous_index],
                contour.points[index],
                t,
            ));
            return length_to_take;
        }
        output.push(contour.points[index]);
        if index == end_index {
            return length;
        }
        previous_index = index;
        previous_length = length;
        index = adjacent_index(index, contour.points.len(), clockwise);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source full-arc mutation names both paths and both endpoint records"
)]
pub(super) fn take_full_arc(
    output: &mut Vec<Point>,
    second: &[Point],
    contour: &[Point],
    intersections: &mut [Intersection],
    start: usize,
    end: usize,
    clockwise: bool,
) {
    debug_assert_ne!(start, end);
    debug_assert!(output.len() >= 2 && second.len() >= 2);
    debug_assert!(intersections[start].prev.is_some() && intersections[start].next.is_some());
    debug_assert!(intersections[end].prev.is_some() && intersections[end].next.is_some());
    let start_point_index = intersections[start].point_index;
    let end_point_index = intersections[end].point_index;
    debug_assert_ne!(start_point_index, end_point_index);
    append_full(
        output,
        contour,
        start_point_index,
        end_point_index,
        clockwise,
    );
    output.extend(second.iter().copied().skip(1));

    let (state_start, state_end) = if clockwise {
        (end, start)
    } else {
        (start, end)
    };
    if intersections[state_start].next != Some(state_end) {
        let mut current = intersections[state_start]
            .next
            .expect("a connected intersection has a next link");
        while intersections[current].next != Some(state_end) {
            let next = intersections[current]
                .next
                .expect("a connected intersection has a next link");
            intersections[current].consume_prev();
            intersections[current].consume_next();
            current = next;
        }
    }
    intersections[state_start].consume_next();
    intersections[state_end].consume_prev();
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source limited-hook helper keeps its graph mutation inputs explicit"
)]
pub(super) fn take_limited(
    polyline: &mut Vec<Point>,
    contour: &BoundaryContour,
    intersections: &mut [Intersection],
    start: usize,
    end: usize,
    clockwise: bool,
    take_max_length: f64,
    line_half_width: f64,
    scaled_epsilon: f64,
) {
    debug_assert!(intersections[start].prev.is_some() && intersections[start].next.is_some());
    debug_assert!(intersections[end].prev.is_some() && intersections[end].next.is_some());
    debug_assert!(polyline.len() >= 2);
    debug_assert_eq!(contour.points.len() + 1, contour.params.len());
    let can_take = if clockwise {
        intersections[start].could_take_prev(scaled_epsilon)
    } else {
        intersections[start].could_take_next(scaled_epsilon)
    };
    if !can_take {
        return;
    }

    let start_point_index = intersections[start].point_index;
    debug_assert!(
        polyline.first() == Some(&contour.points[start_point_index])
            || polyline.last() == Some(&contour.points[start_point_index])
    );
    let add_at_start = polyline.first() == Some(&contour.points[start_point_index]);
    let original = if add_at_start {
        std::mem::take(polyline)
    } else {
        Vec::new()
    };

    let contour_length = *contour.params.last().expect("a contour has parameters");
    let mut length_to_go = take_max_length;
    intersections[start].consumed = true;
    if start == end {
        length_to_go = length_to_go.min(contour_length - line_half_width).max(0.0);
        let available = if clockwise {
            intersections[start].not_taken_prev
        } else {
            intersections[start].not_taken_next
        };
        length_to_go = length_to_go.min(available);
        intersections[start].consume_prev();
        intersections[start].consume_next();
        if length_to_go > scaled_epsilon {
            append_limited(
                polyline,
                contour,
                start_point_index,
                start_point_index,
                clockwise,
                length_to_go,
                scaled_epsilon,
            );
        }
    } else {
        take_limited_direction(
            polyline,
            contour,
            intersections,
            start,
            end,
            clockwise,
            length_to_go,
            line_half_width,
            scaled_epsilon,
        );
    }

    if add_at_start {
        polyline.reverse();
        polyline.extend(original);
    }
}

pub(super) fn skip_sorted_multiline_arc(
    multiline: i32,
    arc_length: f64,
    scaled_spacing: f64,
) -> bool {
    multiline > 1 && arc_length < scaled_spacing * f64::from(multiline)
}

pub(super) fn sorted_arc_takes_full(arc_length: f64, anchor_length_max: f64) -> bool {
    arc_length < anchor_length_max
}

pub(super) fn remaining_arc_is_eligible(length: f64, anchor_length_max: f64) -> bool {
    length != f64::MAX && length <= anchor_length_max
}

pub(super) fn complete_arc_attempts(previous: f64, next: f64) -> [(f64, bool); 2] {
    let shorter = previous.min(next);
    let longer = previous.max(next);
    [(shorter, shorter == previous), (longer, longer == previous)]
}

pub(super) fn limited_hook_is_clockwise(not_taken_previous: f64, not_taken_next: f64) -> bool {
    not_taken_previous > not_taken_next
}

#[expect(
    clippy::too_many_arguments,
    clippy::excessive_nesting,
    reason = "the source directional hook walk preserves ordered trim and append branches"
)]
fn take_limited_direction(
    output: &mut Vec<Point>,
    contour: &BoundaryContour,
    intersections: &mut [Intersection],
    start: usize,
    end: usize,
    clockwise: bool,
    mut length_to_go: f64,
    line_half_width: f64,
    scaled_epsilon: f64,
) {
    let contour_length = *contour.params.last().expect("a contour has parameters");
    let mut current = start;
    while current != end {
        let adjacent = if clockwise {
            intersections[current].prev
        } else {
            intersections[current].next
        }
        .expect("a connected intersection has a contour link");
        let length = if clockwise {
            closed_contour_distance_cw(
                intersections[current].param,
                intersections[adjacent].param,
                contour_length,
            )
        } else {
            closed_contour_distance_ccw(
                intersections[current].param,
                intersections[adjacent].param,
                contour_length,
            )
        };
        let available = if clockwise {
            intersections[current].not_taken_prev
        } else {
            intersections[current].not_taken_next
        };
        length_to_go = length_to_go.min(available);
        length_to_go = length_to_go.min(length - line_half_width).max(0.0);
        if clockwise {
            intersections[current].consume_prev();
        } else {
            intersections[current].consume_next();
        }
        if length >= length_to_go {
            if length_to_go > scaled_epsilon {
                if clockwise {
                    intersections[adjacent].trim_next(length - length_to_go);
                } else {
                    intersections[adjacent].trim_prev(length - length_to_go);
                }
                append_limited(
                    output,
                    contour,
                    intersections[current].point_index,
                    intersections[adjacent].point_index,
                    clockwise,
                    length_to_go,
                    scaled_epsilon,
                );
            }
            break;
        }
        if clockwise {
            intersections[adjacent].trim_next(0.0);
        } else {
            intersections[adjacent].trim_prev(0.0);
        }
        append_full(
            output,
            &contour.points,
            intersections[current].point_index,
            intersections[adjacent].point_index,
            clockwise,
        );
        length_to_go -= length;
        current = adjacent;
    }
}

fn adjacent_index(index: usize, length: usize, clockwise: bool) -> usize {
    if clockwise {
        if index == 0 { length - 1 } else { index - 1 }
    } else if index + 1 == length {
        0
    } else {
        index + 1
    }
}

fn point_distance(start: Point, end: Point) -> f64 {
    let dx = end.x() as f64 - start.x() as f64;
    let dy = end.y() as f64 - start.y() as f64;
    (dx * dx + dy * dy).sqrt()
}
