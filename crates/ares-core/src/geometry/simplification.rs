use super::clipper::simplify_polygons;
use super::{ClipperError, ExPolygon, FillRule, Point, Polygon, union_ex};

pub(crate) fn append_simplified_expolygon(
    expolygon: ExPolygon,
    tolerance: f64,
    output: &mut Vec<ExPolygon>,
) -> Result<(), ClipperError> {
    let (contour, holes) = expolygon.into_parts();
    let mut paths = Vec::with_capacity(holes.len() + 1);
    paths.push(Polygon::new(simplify_closed_points(
        contour.into_points(),
        tolerance,
    )));
    paths.extend(
        holes
            .into_iter()
            .map(|hole| Polygon::new(simplify_closed_points(hole.into_points(), tolerance))),
    );

    let strict_paths = simplify_polygons(&paths)?;
    output.extend(union_ex(&strict_paths, FillRule::NonZero)?);
    Ok(())
}

pub(crate) fn simplify_expolygon_polygons(
    expolygon: &ExPolygon,
    tolerance: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut paths = Vec::with_capacity(expolygon.holes().len() + 1);
    paths.push(Polygon::new(simplify_closed_points(
        expolygon.contour().points().to_vec(),
        tolerance,
    )));
    paths.extend(
        expolygon
            .holes()
            .iter()
            .map(|hole| Polygon::new(simplify_closed_points(hole.points().to_vec(), tolerance))),
    );
    simplify_polygons(&paths)
}

pub(super) fn distance_to_segment_squared(point: Point, start: Point, end: Point) -> f64 {
    let vector_x = (end.x() - start.x()) as f64;
    let vector_y = (end.y() - start.y()) as f64;
    let point_x = (point.x() - start.x()) as f64;
    let point_y = (point.y() - start.y()) as f64;
    let length_squared = vector_x * vector_x + vector_y * vector_y;
    if length_squared == 0.0 {
        return point_x * point_x + point_y * point_y;
    }

    let projection = (point_x * vector_x + point_y * vector_y) / length_squared;
    if projection <= 0.0 {
        point_x * point_x + point_y * point_y
    } else if projection >= 1.0 {
        let point_x = (point.x() - end.x()) as f64;
        let point_y = (point.y() - end.y()) as f64;
        point_x * point_x + point_y * point_y
    } else {
        let distance_x = projection * vector_x - point_x;
        let distance_y = projection * vector_y - point_y;
        distance_x * distance_x + distance_y * distance_y
    }
}

pub(crate) fn douglas_peucker(points: &[Point], tolerance: f64) -> Vec<Point> {
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    let mut result = Vec::with_capacity(points.len());
    result.push(first);

    let mut anchor = 0;
    let mut floater = points.len() - 1;
    if anchor == floater {
        return result;
    }

    let tolerance_squared = tolerance * tolerance;
    let mut endpoints = Vec::with_capacity(points.len());
    endpoints.push(floater);
    loop {
        let mut maximum = 0.0;
        let mut farthest = anchor;
        for index in anchor + 1..floater {
            let distance =
                distance_to_segment_squared(points[index], points[anchor], points[floater]);
            if distance > maximum {
                maximum = distance;
                farthest = index;
            }
        }
        if maximum <= tolerance_squared {
            result.push(points[floater]);
            anchor = floater;
            endpoints.pop();
            let Some(&next) = endpoints.last() else {
                break;
            };
            floater = next;
        } else {
            floater = farthest;
            endpoints.push(floater);
        }
    }
    result
}

pub(crate) fn simplify_closed_points(mut points: Vec<Point>, tolerance: f64) -> Vec<Point> {
    let Some(&first) = points.first() else {
        return points;
    };
    points.push(first);
    let mut simplified = douglas_peucker(&points, tolerance);
    simplified.pop();
    simplified
}
