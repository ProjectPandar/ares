use crate::geometry::{ExPolygon, Point, Polygon};

pub(super) fn clean_expolygon(expolygon: ExPolygon) -> ExPolygon {
    let (contour, holes) = expolygon.into_parts();
    let contour = remove_sticks(contour).unwrap_or_else(|| Polygon::new(Vec::new()));
    let holes = holes.into_iter().filter_map(remove_sticks).collect();
    ExPolygon::new(contour, holes)
}

pub(super) fn clean_paths(polygons: Vec<Polygon>, minimum_area: f64) -> Vec<Polygon> {
    polygons
        .into_iter()
        .filter_map(remove_sticks)
        .filter(|polygon| polygon.area().abs() >= minimum_area)
        .collect()
}

fn remove_sticks(polygon: Polygon) -> Option<Polygon> {
    let points = polygon.into_points();
    if points.len() < 3 {
        return None;
    }
    let mut output = Vec::with_capacity(points.len());
    output.push(points[0]);
    for index in 1..points.len() - 1 {
        if !is_stick(*output.last().unwrap(), points[index], points[index + 1]) {
            output.push(points[index]);
        }
    }
    output.push(*points.last().unwrap());
    while output.len() >= 3
        && is_stick(
            output[output.len() - 2],
            output[output.len() - 1],
            output[0],
        )
    {
        output.pop();
    }
    while output.len() >= 3 && is_stick(output[output.len() - 1], output[0], output[1]) {
        output.remove(0);
    }
    (output.len() >= 3).then(|| Polygon::new(output))
}

#[cfg(test)]
pub(crate) fn remove_sticks_from_polygon(polygon: Polygon) -> Option<Polygon> {
    remove_sticks(polygon)
}

fn is_stick(first: Point, middle: Point, last: Point) -> bool {
    let incoming = (middle.x() - first.x(), middle.y() - first.y());
    let outgoing = (last.x() - middle.x(), last.y() - middle.y());
    let direction = incoming.0 * outgoing.0 + incoming.1 * outgoing.1;
    if direction > 0 {
        return false;
    }
    let incoming_length = (incoming.0 as f64).powi(2) + (incoming.1 as f64).powi(2);
    let outgoing_length = (outgoing.0 as f64).powi(2) + (outgoing.1 as f64).powi(2);
    if direction == 0 {
        return incoming_length == 0.0 || outgoing_length == 0.0;
    }
    let cross = incoming.0 as f64 * outgoing.1 as f64 - outgoing.0 as f64 * incoming.1 as f64;
    cross * cross / incoming_length.max(outgoing_length) < 1.0e-8
}
