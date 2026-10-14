use super::PlanePathPattern;
use crate::{
    geometry::{Line, Point, Polyline},
    project_slice::chain_polylines,
};

// `FillPlanePath.cpp:130-160`: every plane-path pattern chains through
// `chain_polylines` (greedy2 + two-exchange flipping,
// `ShortestPath.cpp:1968-1994`), calibrated or not.
pub(super) fn chain(
    polylines: &mut Vec<Polyline>,
    pattern: PlanePathPattern,
    calibration_order: bool,
    scale: crate::geometry::CoordinateScale,
) {
    if calibration_order && pattern == PlanePathPattern::ArchimedeanChords {
        order_calibration_archimedean(polylines, scale);
    } else {
        chain_polylines(polylines, scale);
    }
}

// `FillPlanePath.cpp:132-153`: preserve the first longest segment on ties,
// orient its center line inside-out, and append it after chaining the remainder.
fn order_calibration_archimedean(
    polylines: &mut Vec<Polyline>,
    scale: crate::geometry::CoordinateScale,
) {
    let mut longest_index = 0;
    let mut longest_length = polyline_length(&polylines[0]);
    for (index, polyline) in polylines.iter().enumerate().skip(1) {
        let length = polyline_length(polyline);
        if length > longest_length {
            longest_index = index;
            longest_length = length;
        }
    }
    let mut center = polylines.remove(longest_index);
    if squared_norm(center.front().expect("plane path is valid"))
        > squared_norm(center.back().expect("plane path is valid"))
    {
        center.reverse();
    }
    chain_polylines(polylines, scale);
    polylines.push(center);
}

fn polyline_length(polyline: &Polyline) -> f64 {
    polyline
        .points()
        .windows(2)
        .map(|points| Line::new(points[0], points[1]).length())
        .sum()
}

fn squared_norm(point: Point) -> i128 {
    i128::from(point.x()) * i128::from(point.x()) + i128::from(point.y()) * i128::from(point.y())
}
