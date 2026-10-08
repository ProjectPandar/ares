//! `BridgeDetector.cpp:35–218`: anchors, candidate directions and anchored-line coverage.
#[path = "coverage.rs"]
mod coverage;
use crate::geometry::{
    ClipperError, ExPolygon, JoinType, Line, Point, Polygon, Polyline, fixed_gcc_sort_by,
    intersection_open_polylines, intersection_polygons_ex, offset_expolygons_paths,
    union_safety_offset_expolygons,
};
use std::f64::consts::PI;

struct Direction {
    angle: f64,
    coverage: f64,
    max_length: f64,
}

pub(super) fn bridgeable(
    polygon: &ExPolygon,
    support: &[ExPolygon],
    spacing: i64,
    angle_override: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let grown = offset_expolygons_paths(
        std::slice::from_ref(polygon),
        spacing as f32,
        JoinType::Miter,
        3.0,
    )?;
    let contours = support
        .iter()
        .map(|p| p.contour().clone())
        .collect::<Vec<_>>();
    let paths = grown
        .iter()
        .map(Polygon::split_at_first_point)
        .collect::<Vec<_>>();
    let edges = intersection_open_polylines(&paths, &contours)?;
    let anchors = intersection_polygons_ex(&grown, &union_safety_offset_expolygons(support)?)?;
    if edges.is_empty() || anchors.is_empty() {
        return Ok(Vec::new());
    }
    let angles = if angle_override == 0.0 {
        candidates(polygon, &edges)
    } else {
        vec![angle_override]
    };
    let clip = offset_expolygons_paths(
        std::slice::from_ref(polygon),
        0.5_f32 * spacing as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut directions = Vec::with_capacity(angles.len());
    let mut have_coverage = false;
    for angle in angles {
        // MultiPoint.cpp:539–562 rounds the rotated contour bounding box.
        let points = anchors
            .iter()
            .flat_map(|p| p.contour().points())
            .map(|p| rotate_point(*p, -angle))
            .collect::<Vec<_>>();
        let (min, max) = coverage::bounds(&points);
        let mut lines = Vec::new();
        let mut y = min[1];
        while y <= max[1] {
            lines.push(Polyline::new(vec![
                rotate_point(Point::new(min[0], y), angle),
                rotate_point(Point::new(max[0], y), angle),
            ]));
            y += spacing;
        }
        let clipped = intersection_open_polylines(&lines, &clip)?;
        let mut total = 0.0;
        let mut longest: f64 = 0.0;
        for path in clipped {
            let points = path.points();
            let line = Line::new(points[0], points[points.len() - 1]);
            if contains(&anchors, line.a) && contains(&anchors, line.b) {
                let length = line.length();
                total += length;
                longest = longest.max(length);
            }
        }
        have_coverage |= total != 0.0;
        directions.push(Direction {
            angle,
            coverage: total,
            max_length: longest,
        });
    }
    if !have_coverage {
        return Ok(Vec::new());
    }
    fixed_gcc_sort_by(&mut directions, |a, b| a.coverage > b.coverage);
    let mut best = 0;
    for index in 1..directions.len() {
        if directions[best].coverage - directions[index].coverage >= spacing as f64 {
            break;
        }
        if directions[index].max_length < directions[best].max_length {
            best = index;
        }
    }
    let angle = directions[best].angle;
    coverage::coverage(
        polygon,
        &anchors,
        spacing,
        if angle >= PI { angle - PI } else { angle },
    )
}

fn candidates(polygon: &ExPolygon, edges: &[Polyline]) -> Vec<f64> {
    let resolution = PI / 36.0;
    let mut angles = (0..=(PI / resolution) as usize)
        .map(|i| i as f64 * resolution)
        .collect::<Vec<_>>();
    angles.extend(polygon.lines().into_iter().map(direction));
    for edge in edges {
        let points = edge.points();
        if points[0] != points[points.len() - 1] {
            angles.push(direction(Line::new(points[0], points[points.len() - 1])));
        }
    }
    fixed_gcc_sort_by(&mut angles, |a, b| a < b);
    let mut index = 1;
    while index < angles.len() {
        if parallel(angles[index], angles[index - 1]) {
            angles.remove(index);
        } else {
            index += 1;
        }
    }
    if parallel(angles[0], angles[angles.len() - 1]) {
        angles.pop();
    }
    angles
}

fn direction(line: Line) -> f64 {
    let angle = line.orientation();
    if angle >= PI { angle - PI } else { angle }
}
fn parallel(a: f64, b: f64) -> bool {
    let difference = (a - b).abs();
    let tolerance = PI / 180.0;
    difference < tolerance || (difference - PI).abs() < tolerance
}
fn rotate_point(point: Point, angle: f64) -> Point {
    let c = angle.cos();
    let s = angle.sin();
    let x = point.x() as f64;
    let y = point.y() as f64;
    Point::new(
        (c * x - s * y).round() as i64,
        (c * y + s * x).round() as i64,
    )
}
fn contains(polygons: &[ExPolygon], point: Point) -> bool {
    polygons.iter().any(|p| {
        p.contour().contains(&point) && !p.holes().iter().any(|hole| hole.contains(&point))
    })
}
