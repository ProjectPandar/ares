use crate::{LayerContours, Point2};

type Bounds = (f64, f64, f64, f64);

pub(crate) fn fully_unsupported_layer(
    layer_contours: &[LayerContours],
    layer_index: usize,
) -> bool {
    let Some(previous_index) = layer_index.checked_sub(1) else {
        return false;
    };
    let Some(current) = layer_contours.get(layer_index) else {
        return false;
    };
    let Some(previous) = layer_contours.get(previous_index) else {
        return false;
    };

    let previous_bounds = previous
        .contours()
        .iter()
        .filter_map(|contour| contour_bounds(contour.points()))
        .collect::<Vec<_>>();

    !current.contours().is_empty()
        && current.contours().iter().all(|current_contour| {
            contour_bounds(current_contour.points())
                .is_some_and(|bounds| !overlaps_any_previous(bounds, &previous_bounds))
        })
}

fn overlaps_any_previous(current: Bounds, previous_bounds: &[Bounds]) -> bool {
    previous_bounds
        .iter()
        .any(|previous| has_positive_area_overlap(current, *previous))
}

fn contour_bounds(points: &[Point2]) -> Option<Bounds> {
    let first = points.first()?;
    let mut min_x = first.x();
    let mut max_x = first.x();
    let mut min_y = first.y();
    let mut max_y = first.y();
    for point in &points[1..] {
        min_x = min_x.min(point.x());
        max_x = max_x.max(point.x());
        min_y = min_y.min(point.y());
        max_y = max_y.max(point.y());
    }
    Some((min_x, min_y, max_x, max_y))
}

fn has_positive_area_overlap(current: Bounds, previous: Bounds) -> bool {
    current.0.max(previous.0) < current.2.min(previous.2)
        && current.1.max(previous.1) < current.3.min(previous.3)
}
