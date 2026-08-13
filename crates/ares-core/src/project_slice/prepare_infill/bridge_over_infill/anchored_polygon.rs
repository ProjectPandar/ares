mod sections;
mod tracing;

#[cfg(test)]
mod tests;

use crate::{
    geometry::{ClipperError, CoordinateScale, Line, Point, Polygon, union_safety_offset_polygons},
    project_slice::perimeters::types::Flow,
};

use sections::build_sections;
use tracing::trace_sections;

pub(in crate::project_slice) fn construct_anchored_polygon(
    bridged_area: &[Polygon],
    anchors: &[Line],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let spacing = scaled_flow_value(bridging_flow.spacing, scale);
    let width = scaled_flow_value(bridging_flow.width, scale);
    let aligning_angle = -bridging_angle + std::f64::consts::PI * 0.5;
    let cosine = aligning_angle.cos();
    let sine = aligning_angle.sin();
    let rotated_area = bridged_area
        .iter()
        .map(|polygon| rotate_polygon(polygon, cosine, sine))
        .collect::<Vec<_>>();
    let rotated_anchors = anchors
        .iter()
        .copied()
        .map(|line| rotate_line(line, cosine, sine))
        .collect::<Vec<_>>();

    let sections = build_sections(&rotated_area, &rotated_anchors, spacing, width);
    let traced = trace_sections(&sections, spacing);
    let mut expanded = union_safety_offset_polygons(&traced)?;
    let inverse_cosine = (-aligning_angle).cos();
    let inverse_sine = (-aligning_angle).sin();
    for polygon in &mut expanded {
        *polygon = rotate_polygon(polygon, inverse_cosine, inverse_sine);
    }
    Ok(expanded)
}

pub(super) fn scaled_flow_value(value: f32, scale: CoordinateScale) -> i64 {
    (f64::from(value) / scale.factor()) as i64
}

fn rotate_polygon(polygon: &Polygon, cosine: f64, sine: f64) -> Polygon {
    Polygon::new(
        polygon
            .points()
            .iter()
            .copied()
            .map(|point| rotate_point(point, cosine, sine))
            .collect(),
    )
}

fn rotate_line(line: Line, cosine: f64, sine: f64) -> Line {
    Line::new(
        rotate_point(line.a, cosine, sine),
        rotate_point(line.b, cosine, sine),
    )
}

fn rotate_point(point: Point, cosine: f64, sine: f64) -> Point {
    let x = point.x() as f64;
    let y = point.y() as f64;
    Point::new(
        (cosine * x - sine * y).round() as i64,
        (cosine * y + sine * x).round() as i64,
    )
}

#[cfg(test)]
pub(super) fn scaled_flow_value_for_test(value: f32, scale: CoordinateScale) -> i64 {
    scaled_flow_value(value, scale)
}

#[cfg(test)]
pub(super) fn rotate_point_for_test(point: Point, cosine: f64, sine: f64) -> Point {
    rotate_point(point, cosine, sine)
}
