use crate::{PerimeterOptions, Point2};

use super::overhang::ExternalRole;

pub(super) fn orient_points(
    points: Vec<Point2>,
    external_role: ExternalRole,
    is_external_path: bool,
    layer_id: usize,
    options: PerimeterOptions,
) -> Vec<Point2> {
    if should_reverse(external_role, is_external_path, layer_id, options) {
        let mut points = points;
        points.reverse();
        points
    } else {
        points
    }
}

fn should_reverse(
    external_role: ExternalRole,
    is_external_path: bool,
    layer_id: usize,
    options: PerimeterOptions,
) -> bool {
    options.overhang_reverse()
        && layer_id % 2 == 1
        && !(options.overhang_reverse_internal_only() && is_external_path)
        && (!options.detect_overhang_wall()
            || external_role
                .unsupported_span_mm()
                .is_some_and(|span| options.overhang_reverse_threshold_mm() <= span))
}
