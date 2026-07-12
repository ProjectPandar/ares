use crate::Point2;

pub(super) fn filled_area_mm2(contours: &[&[Point2]]) -> f64 {
    let mut areas = contours
        .iter()
        .map(|points| polygon_area_mm2(points).abs())
        .collect::<Vec<_>>();
    areas.sort_by(|a, b| b.total_cmp(a));
    match areas.split_first() {
        Some((outer, holes)) => (outer - holes.iter().sum::<f64>()).max(0.0),
        None => 0.0,
    }
}

fn polygon_area_mm2(points: &[Point2]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a.x() * b.y() - b.x() * a.y())
        .sum::<f64>()
        * 0.5
}
