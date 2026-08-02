// Source-compatible bounding-box prefilter from OrcaSlicer ClipperUtils.cpp.

use super::{BoundingBox, ExPolygon, Point, Polygon};

pub(crate) fn clip_clipper_polygons_with_subject_bbox(
    polygons: &[Polygon],
    bounds: BoundingBox,
) -> Vec<Polygon> {
    polygons
        .iter()
        .filter_map(|polygon| clip_polygon(polygon, bounds, false))
        .collect()
}

pub(crate) fn clip_clipper_expolygons_with_subject_bbox(
    expolygons: &[ExPolygon],
    bounds: BoundingBox,
) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|expolygon| {
            std::iter::once(expolygon.contour())
                .chain(expolygon.holes())
                .filter_map(move |polygon| clip_polygon(polygon, bounds, false))
        })
        .collect()
}

fn clip_polygon(
    polygon: &Polygon,
    bounds: BoundingBox,
    get_entire_polygon: bool,
) -> Option<Polygon> {
    let points = polygon.points();
    if points.len() < 3 {
        return None;
    }

    let mut output = Vec::with_capacity(points.len());
    let mut sides_previous = sides(*points.last().unwrap(), bounds);
    let mut sides_this = sides(points[0], bounds);
    for index in 0..points.len() - 1 {
        let sides_next = sides(points[index + 1], bounds);
        if sides_this == 0 || sides_previous & sides_this & sides_next == 0 {
            output.push(points[index]);
            sides_previous = sides_this;
        }
        sides_this = sides_next;
    }

    if output.is_empty() {
        return None;
    }
    if get_entire_polygon {
        return Some(polygon.clone());
    }
    let sides_next = sides(output[0], bounds);
    if sides_this == 0 || sides_previous & sides_this & sides_next == 0 {
        output.push(*points.last().unwrap());
    }
    Some(Polygon::new(output))
}

fn sides(point: Point, bounds: BoundingBox) -> u8 {
    u8::from(point.x() < bounds.min().x())
        + 2 * u8::from(point.x() > bounds.max().x())
        + 8 * u8::from(point.y() < bounds.min().y())
        + 4 * u8::from(point.y() > bounds.max().y())
}
