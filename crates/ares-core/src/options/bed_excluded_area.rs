use crate::{Model, Point2, SliceError, SliceOptions, model::XyBounds};

impl SliceOptions {
    pub(crate) fn validate_model_bed_excluded_area(
        &self,
        model: &Model,
    ) -> Result<(), SliceError> {
        let Some(polygon) = parse_bed_exclude_area(self.values().get("bed_exclude_area"))? else {
            return Ok(());
        };
        if polygon.len() < 3 {
            return Ok(());
        }
        let Some(bounds) = model.xy_bounds() else {
            return Ok(());
        };
        if polygon_intersects_bounds(&polygon, bounds) {
            return Err(SliceError::InvalidInput(
                "bed_exclude_area intersects model XY bounds".to_owned(),
            ));
        }
        Ok(())
    }
}

fn parse_bed_exclude_area(value: Option<&serde_json::Value>) -> Result<Option<Vec<Point2>>, SliceError> {
    match value {
        None => Ok(None),
        Some(serde_json::Value::String(text)) => parse_bed_exclude_area_text(text),
        Some(serde_json::Value::Array(points)) => parse_bed_exclude_area_array(points),
        Some(_) => Err(bed_exclude_area_error("unsupported value")),
    }
}

fn parse_bed_exclude_area_text(text: &str) -> Result<Option<Vec<Point2>>, SliceError> {
    let text = text.trim();
    if text.is_empty() || text == "0x0" {
        return Ok(None);
    }

    let mut points = Vec::new();
    for token in text.split(',') {
        let token = token.trim();
        let Some((x, y)) = token.split_once('x') else {
            return Err(bed_exclude_area_error("malformed point"));
        };
        if y.contains('x') || x.trim().is_empty() || y.trim().is_empty() {
            return Err(bed_exclude_area_error("malformed point"));
        }
        points.push(point(parse_f64(x)?, parse_f64(y)?)?);
    }
    Ok(Some(points))
}

fn parse_bed_exclude_area_array(
    values: &[serde_json::Value],
) -> Result<Option<Vec<Point2>>, SliceError> {
    if values.is_empty() {
        return Ok(None);
    }
    let mut points = Vec::with_capacity(values.len());
    for value in values {
        let serde_json::Value::Array(coords) = value else {
            return Err(bed_exclude_area_error("malformed JSON point"));
        };
        let [x, y] = coords.as_slice() else {
            return Err(bed_exclude_area_error("malformed JSON point"));
        };
        let Some(x) = x.as_f64() else {
            return Err(bed_exclude_area_error("malformed JSON point"));
        };
        let Some(y) = y.as_f64() else {
            return Err(bed_exclude_area_error("malformed JSON point"));
        };
        points.push(point(x, y)?);
    }
    Ok(Some(points))
}

fn parse_f64(value: &str) -> Result<f64, SliceError> {
    value
        .trim()
        .parse()
        .map_err(|_| bed_exclude_area_error("malformed coordinate"))
}

fn point(x: f64, y: f64) -> Result<Point2, SliceError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(bed_exclude_area_error("non-finite coordinate"));
    }
    Ok(Point2::new(x, y))
}

fn bed_exclude_area_error(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("bed_exclude_area {reason}"))
}

fn polygon_intersects_bounds(polygon: &[Point2], bounds: XyBounds) -> bool {
    polygon.iter().any(|point| point_in_bounds(*point, bounds))
        || bounds_corners(bounds)
            .into_iter()
            .any(|corner| point_in_polygon(corner, polygon))
        || polygon_edges(polygon).any(|(start, end)| {
            bounds_edges(bounds)
                .into_iter()
                .any(|edge| segments_intersect(start, end, edge.0, edge.1))
        })
}

fn point_in_bounds(point: Point2, bounds: XyBounds) -> bool {
    (bounds.min_x..=bounds.max_x).contains(&point.x())
        && (bounds.min_y..=bounds.max_y).contains(&point.y())
}

fn bounds_corners(bounds: XyBounds) -> [Point2; 4] {
    [
        Point2::new(bounds.min_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.max_y),
        Point2::new(bounds.min_x, bounds.max_y),
    ]
}

fn bounds_edges(bounds: XyBounds) -> [(Point2, Point2); 4] {
    let [bottom_left, bottom_right, top_right, top_left] = bounds_corners(bounds);
    [
        (bottom_left, bottom_right),
        (bottom_right, top_right),
        (top_right, top_left),
        (top_left, bottom_left),
    ]
}

fn polygon_edges(points: &[Point2]) -> impl Iterator<Item = (Point2, Point2)> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
}

fn point_in_polygon(point: Point2, polygon: &[Point2]) -> bool {
    if polygon_edges(polygon).any(|(start, end)| point_on_segment(point, start, end)) {
        return true;
    }

    let mut inside = false;
    for (start, end) in polygon_edges(polygon) {
        let crosses_y = (start.y() > point.y()) != (end.y() > point.y());
        if crosses_y {
            let x = (end.x() - start.x()) * (point.y() - start.y()) / (end.y() - start.y())
                + start.x();
            if point.x() < x {
                inside = !inside;
            }
        }
    }
    inside
}

fn segments_intersect(a: Point2, b: Point2, c: Point2, d: Point2) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);

    (ab_c == 0.0 && point_on_segment(c, a, b))
        || (ab_d == 0.0 && point_on_segment(d, a, b))
        || (cd_a == 0.0 && point_on_segment(a, c, d))
        || (cd_b == 0.0 && point_on_segment(b, c, d))
        || ((ab_c > 0.0) != (ab_d > 0.0) && (cd_a > 0.0) != (cd_b > 0.0))
}

fn point_on_segment(point: Point2, start: Point2, end: Point2) -> bool {
    orientation(start, end, point) == 0.0
        && point.x() >= start.x().min(end.x())
        && point.x() <= start.x().max(end.x())
        && point.y() >= start.y().min(end.y())
        && point.y() <= start.y().max(end.y())
}

fn orientation(a: Point2, b: Point2, c: Point2) -> f64 {
    ((b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x()))
        .clamp(-1.0, 1.0)
}
