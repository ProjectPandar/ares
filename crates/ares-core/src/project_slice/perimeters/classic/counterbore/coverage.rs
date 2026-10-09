//! `BridgeDetector.cpp:235–418`, precise coverage only (`coverage(-1,true)`).
use crate::geometry::{
    ClipperError, ExPolygon, JoinType, Point, Polygon, intersection_polygons_paths,
    offset_expolygon, union_polygons_paths,
};

pub(super) fn coverage(
    bridge: &ExPolygon,
    anchors: &[ExPolygon],
    spacing: i64,
    angle: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let rotation = std::f64::consts::FRAC_PI_2 - angle;
    let mut anchors = anchors
        .iter()
        .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
        .cloned()
        .collect::<Vec<_>>();
    for polygon in &mut anchors {
        polygon.rotate(rotation.cos(), rotation.sin());
    }
    let rotated = rotate_expolygon(bridge, rotation);
    let mut covered = Vec::new();
    for expanded in offset_expolygon(&rotated, 0.5_f32 * spacing as f32, JoinType::Miter, 3.0)? {
        cover_expanded_bridge(&expanded, &anchors, spacing, &mut covered)?;
    }
    let mut covered = union_polygons_paths(&covered)?;
    for polygon in &mut covered {
        polygon.rotate((-rotation).cos(), (-rotation).sin());
    }
    Ok(covered)
}

fn cover_expanded_bridge(
    expanded: &ExPolygon,
    anchors: &[Polygon],
    spacing: i64,
    covered: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    let points = std::iter::once(expanded.contour())
        .chain(expanded.holes())
        .flat_map(Polygon::points)
        .copied()
        .collect::<Vec<_>>();
    let (min, max) = bounds(&points);
    let step = spacing as f32 as i64;
    let mut xs = Vec::new();
    let mut x = min[0];
    while x < max[0] - (spacing as f32 / 2.0) as i64 {
        xs.push(x);
        x += step;
    }
    xs.push(max[0]);
    let area = std::iter::once(expanded.contour())
        .chain(expanded.holes())
        .cloned()
        .collect::<Vec<_>>();
    for pair in xs.windows(2) {
        if pair[0] == pair[1] {
            continue;
        }
        let quarter = step / 4;
        let rectangle = Polygon::new(vec![
            Point::new(pair[0] + quarter, min[1]),
            Point::new(pair[1] - quarter, min[1]),
            Point::new(pair[1] - quarter, max[1]),
            Point::new(pair[0] + quarter, max[1]),
        ]);
        cover_strip(rectangle, &area, anchors, spacing, covered)?;
    }
    Ok(())
}

fn cover_strip(
    rectangle: Polygon,
    area: &[Polygon],
    anchors: &[Polygon],
    spacing: i64,
    covered: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    for trapezoid in intersection_polygons_paths(&[rectangle], area)? {
        let Some(polygon) = supported_trapezoid(trapezoid, anchors, spacing)? else {
            continue;
        };
        covered.push(polygon);
    }
    Ok(())
}

fn supported_trapezoid(
    trapezoid: Polygon,
    anchors: &[Polygon],
    spacing: i64,
) -> Result<Option<Polygon>, ClipperError> {
    let intersects = intersection_polygons_paths(std::slice::from_ref(&trapezoid), anchors)?;
    if intersects.len() < 2 {
        return Ok(None);
    }
    let mut min_y = i64::MAX;
    let mut max_y = i64::MIN;
    for polygon in &intersects {
        let (lo, hi) = bounds(polygon.points());
        let center = (lo[1] + hi[1]) / 2;
        min_y = min_y.min(center);
        max_y = max_y.max(center);
    }
    let (lo, hi) = bounds(trapezoid.points());
    let min_x = lo[0] - (spacing / 4 + 1);
    let max_x = hi[0] + (spacing / 4 + 1);
    let mid_x = (min_x + max_x) / 2;
    let points = trapezoid
        .into_points()
        .into_iter()
        .map(|point| {
            let mut x = point.x();
            if x > min_x && x < mid_x {
                x = min_x;
            }
            if x < max_x && x > mid_x {
                x = max_x;
            }
            Point::new(x, point.y().clamp(min_y, max_y))
        })
        .collect();
    Ok(Some(Polygon::new(points)))
}

pub(super) fn bounds(points: &[Point]) -> ([i64; 2], [i64; 2]) {
    let mut min = [points[0].x(), points[0].y()];
    let mut max = min;
    for point in points {
        min[0] = min[0].min(point.x());
        min[1] = min[1].min(point.y());
        max[0] = max[0].max(point.x());
        max[1] = max[1].max(point.y());
    }
    (min, max)
}

fn rotate_expolygon(polygon: &ExPolygon, angle: f64) -> ExPolygon {
    let mut contour = polygon.contour().clone();
    contour.rotate(angle.cos(), angle.sin());
    let mut holes = polygon.holes().to_vec();
    for hole in &mut holes {
        hole.rotate(angle.cos(), angle.sin());
    }
    ExPolygon::new(contour, holes)
}
