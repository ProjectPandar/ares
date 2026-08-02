pub(crate) mod annotate;
pub(crate) mod chaining;
pub(crate) mod diagram;
pub(crate) mod validate;

use crate::geometry::{CoordinateScale, ExPolygon, Line, Point, ThickPolyline};

use annotate::VertexCategory;
use validate::{EdgeData, ValidationLimits};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MedialAxisError {
    ConstructionFailed,
}

pub(crate) fn medial_axis(
    expolygon: &ExPolygon,
    min_width: f64,
    max_width: f64,
    scale: CoordinateScale,
) -> Result<Vec<ThickPolyline>, MedialAxisError> {
    let lines = expolygon.lines();
    if lines.is_empty() {
        return Ok(Vec::new());
    }
    let vd = diagram::build(&lines)?;
    let annotations = annotate::annotate(&vd, &lines)?;
    let epsilon = scaled_epsilon(scale);
    let mut edge_data = vec![EdgeData::default(); vd.num_edges() / 2];
    for index in (0..vd.num_edges()).step_by(2) {
        let edge = diagram::edge_index(&vd, index);
        if !vd
            .edge(edge)
            .map_err(|_| MedialAxisError::ConstructionFailed)?
            .is_primary()
            || !vd
                .edge_is_finite(edge)
                .map_err(|_| MedialAxisError::ConstructionFailed)?
        {
            continue;
        }
        if edge_is_eligible(&vd, edge, &annotations)?
            && let Some(data) = validate::validate(
                &vd,
                edge,
                &lines,
                ValidationLimits {
                    min_width,
                    max_width,
                    scaled_epsilon: epsilon,
                },
            )?
        {
            edge_data[index / 2] = data;
        }
    }
    let polylines = chaining::chain(&vd, &mut edge_data)?;
    Ok(postprocess(expolygon, polylines, max_width, epsilon))
}

pub(crate) fn edge_is_eligible(
    vd: &boostvoronoi::prelude::Diagram,
    edge: boostvoronoi::prelude::EdgeIndex,
    annotations: &annotate::Annotations,
) -> Result<bool, MedialAxisError> {
    let vertex0 = vd
        .edge_get_vertex0(edge)
        .map_err(|_| MedialAxisError::ConstructionFailed)?;
    let vertex1 = vd
        .edge_get_vertex1(edge)
        .map_err(|_| MedialAxisError::ConstructionFailed)?;
    Ok([vertex0, vertex1]
        .into_iter()
        .flatten()
        .any(|vertex| annotations.vertices[vertex.usize()] == VertexCategory::Inside))
}

pub(crate) fn scaled_epsilon(scale: CoordinateScale) -> f64 {
    1e-4 / scale.factor()
}

pub(crate) fn postprocess(
    expolygon: &ExPolygon,
    mut polylines: Vec<ThickPolyline>,
    max_width: f64,
    epsilon: f64,
) -> Vec<ThickPolyline> {
    let max_returned_width = polylines
        .iter()
        .flat_map(|polyline| &polyline.width)
        .fold(0.0, |maximum, &width| {
            (maximum as f32).max(width as f32) as f64
        });
    let mut removed = false;
    let mut index = 0;
    while index < polylines.len() {
        extend_endpoints(expolygon, &mut polylines[index], max_width, epsilon);
        if (polylines[index].endpoints.0 || polylines[index].endpoints.1)
            && polylines[index].length() < max_returned_width * 2.0
        {
            polylines.remove(index);
            removed = true;
        } else {
            index += 1;
        }
    }
    if removed {
        reconnect(&mut polylines);
    }
    polylines
}

fn extend_endpoints(
    expolygon: &ExPolygon,
    polyline: &mut ThickPolyline,
    max_width: f64,
    epsilon: f64,
) {
    let mut front = polyline.points[0];
    let mut back = *polyline.points.last().unwrap();
    if polyline.endpoints.0 && !expolygon.on_boundary(front, epsilon) {
        let p1 = float(front);
        let mut p2 = float(polyline.points[1]);
        if polyline.points.len() == 2 {
            p2 = midpoint(p1, p2);
        }
        let direction = normalize(p2.0 - p1.0, p2.1 - p1.1);
        let extended = Point::new(
            (p1.0 - direction.0 * max_width) as i64,
            (p1.1 - direction.1 * max_width) as i64,
        );
        if let Some(intersection) = expolygon
            .contour()
            .intersection(Line::new(extended, Point::new(p2.0 as i64, p2.1 as i64)))
        {
            front = intersection;
        }
    }
    if polyline.endpoints.1 && !expolygon.on_boundary(back, epsilon) {
        let mut p1 = float(polyline.points[polyline.points.len() - 2]);
        let p2 = float(back);
        if polyline.points.len() == 2 {
            p1 = midpoint(p1, p2);
        }
        let direction = normalize(p2.0 - p1.0, p2.1 - p1.1);
        let extended = Point::new(
            (p2.0 + direction.0 * max_width) as i64,
            (p2.1 + direction.1 * max_width) as i64,
        );
        if let Some(intersection) = expolygon
            .contour()
            .intersection(Line::new(Point::new(p1.0 as i64, p1.1 as i64), extended))
        {
            back = intersection;
        }
    }
    polyline.points[0] = front;
    *polyline.points.last_mut().unwrap() = back;
}

fn reconnect(polylines: &mut Vec<ThickPolyline>) {
    let mut index = 0;
    while index < polylines.len() {
        if polylines[index].endpoints.0 && polylines[index].endpoints.1 {
            index += 1;
            continue;
        }
        let mut other_index = index + 1;
        while other_index < polylines.len() {
            {
                let (before, after) = polylines.split_at_mut(other_index);
                orient_pair(&mut before[index], &mut after[0]);
            }
            if polylines[index].points.last() != polylines[other_index].points.first() {
                other_index += 1;
                continue;
            }
            let other = polylines.remove(other_index);
            polylines[index]
                .points
                .extend(other.points.into_iter().skip(1));
            polylines[index].width.extend(other.width);
            polylines[index].endpoints.1 = other.endpoints.1;
            other_index = index + 1;
        }
        index += 1;
    }
}

fn orient_pair(left: &mut ThickPolyline, right: &mut ThickPolyline) {
    if left.points.last() == right.points.last() {
        right.reverse();
    } else if left.points.first() == right.points.last() {
        left.reverse();
        right.reverse();
    } else if left.points.first() == right.points.first() {
        left.reverse();
    }
}

fn float(point: Point) -> (f64, f64) {
    (point.x() as f64, point.y() as f64)
}

fn midpoint(left: (f64, f64), right: (f64, f64)) -> (f64, f64) {
    ((left.0 + right.0) * 0.5, (left.1 + right.1) * 0.5)
}

fn normalize(x: f64, y: f64) -> (f64, f64) {
    let length = (x * x + y * y).sqrt();
    (x / length, y / length)
}
