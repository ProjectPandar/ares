use crate::{InfillOptions, InfillPattern, Point2};

pub(super) fn mirror_axis_x(
    pattern: InfillPattern,
    options: &InfillOptions,
    contours: &[&[Point2]],
) -> Option<f64> {
    (options.symmetric_infill_y_axis()
        && matches!(
            pattern,
            InfillPattern::ZigZag | InfillPattern::CrossZag | InfillPattern::LockedZag
        ))
    .then(|| contour_center_x(contours))
}

pub(super) fn mirror_contour_x(points: &[Point2], axis_x: f64) -> Vec<Point2> {
    points
        .iter()
        .map(|point| mirror_point_x(*point, axis_x))
        .collect()
}

pub(super) fn mirror_point_x(point: Point2, axis_x: f64) -> Point2 {
    Point2::new(2.0 * axis_x - point.x(), point.y())
}

fn contour_center_x(contours: &[&[Point2]]) -> f64 {
    let (min_x, max_x) = contours.iter().flat_map(|points| points.iter()).fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(min_x, max_x), point| (min_x.min(point.x()), max_x.max(point.x())),
    );
    (min_x + max_x) * 0.5
}
