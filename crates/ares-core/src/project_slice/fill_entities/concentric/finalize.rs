//! Tail-stage polyline handling for concentric fills (`FillConcentric.cpp`):
//! loop start-point selection (`split_at_index` nearest to the previous end),
//! end clipping (`loop_clipping`), validity filtering, and outside-in
//! reordering of thick polylines.

use crate::geometry::{Point, ThickPolyline};

use crate::project_slice::perimeters::classic::shortest_path::reorder_thick_polylines;

pub(super) fn finalize_standard_polylines(
    polylines: Vec<(ThickPolyline, bool)>,
    loop_clipping: f64,
) -> Vec<ThickPolyline> {
    // `FillConcentric.cpp:110-121`: rotate closed rings to the junction
    // nearest the previous end; open rings keep the raw walk start.
    let mut last = Point::new(0, 0);
    let mut polylines = polylines
        .into_iter()
        .map(|(mut polyline, is_closed)| {
            if is_closed {
                polyline.start_at_index(crate::fill::concentric_sparse::nearest_point_index(
                    last,
                    &polyline.points,
                ));
            }
            if let Some(point) = polyline.points.last() {
                last = *point;
            }
            polyline
        })
        .collect::<Vec<_>>();
    for polyline in &mut polylines {
        polyline.clip_end(loop_clipping);
    }
    polylines.retain(|polyline| polyline.points.len() >= 2);
    reorder_thick_polylines(&mut polylines);
    polylines
}

pub(super) fn nearest_to_origin(points: &[Point]) -> usize {
    points
        .iter()
        .enumerate()
        .fold((0, i128::MAX), |nearest, (index, point)| {
            let distance = squared_distance(*point, Point::new(0, 0));
            if distance <= nearest.1 {
                (index, distance)
            } else {
                nearest
            }
        })
        .0
}

fn squared_distance(left: Point, right: Point) -> i128 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    dx * dx + dy * dy
}
